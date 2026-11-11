//! Native containment for the Parse worker: a job object caps memory and kills
//! on close, the launched file is pinned and matched to the running engine, and
//! the child sees only its two pipes. The inheritable duplicates exist only
//! while `child_launch_lock` is held, which every process launch in the engine
//! takes, so no sibling launch can inherit them. A host embedding the engine
//! that launches children without that lock is outside this guarantee.

use super::WORKER_FLAG;
use std::ffi::{c_void, OsStr};
use std::io;
use std::mem::{size_of, zeroed};
use std::os::windows::ffi::OsStrExt;
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::{AsRawHandle, FromRawHandle};
use std::path::Path;
use std::ptr::{null, null_mut};
use std::sync::OnceLock;
use std::time::Duration;
use windows_sys::Win32::Foundation::{
    CloseHandle, DuplicateHandle, DUPLICATE_SAME_ACCESS, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT,
};
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::Storage::FileSystem::{
    GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION, FILE_SHARE_READ,
};
use windows_sys::Win32::System::JobObjects::{
    CreateJobObjectW, JobObjectExtendedLimitInformation, SetInformationJobObject,
    TerminateJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_JOB_MEMORY,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOB_OBJECT_LIMIT_PROCESS_MEMORY,
};
use windows_sys::Win32::System::Pipes::CreatePipe;
use windows_sys::Win32::System::Threading::{
    CreateProcessW, DeleteProcThreadAttributeList, GetCurrentProcess, GetExitCodeProcess,
    InitializeProcThreadAttributeList, ResumeThread, UpdateProcThreadAttribute,
    WaitForSingleObject, CREATE_NO_WINDOW, CREATE_SUSPENDED, CREATE_UNICODE_ENVIRONMENT,
    EXTENDED_STARTUPINFO_PRESENT, LPPROC_THREAD_ATTRIBUTE_LIST, PROCESS_INFORMATION,
    PROC_THREAD_ATTRIBUTE_HANDLE_LIST, PROC_THREAD_ATTRIBUTE_JOB_LIST, STARTF_USESTDHANDLES,
    STARTUPINFOEXW,
};

struct Handle(HANDLE);

impl Handle {
    fn into_file(self) -> std::fs::File {
        let raw = self.0;
        std::mem::forget(self);
        // SAFETY: this owned, live handle is transferred exactly once to File.
        unsafe { std::fs::File::from_raw_handle(raw) }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        // SAFETY: Handle owns a non-null Windows handle and closes it exactly once.
        unsafe { CloseHandle(self.0) };
    }
}

struct Attributes {
    // The native API requires aligned backing memory to outlive its list.
    _storage: Vec<usize>,
    list: LPPROC_THREAD_ATTRIBUTE_LIST,
}

impl Attributes {
    fn new(count: u32) -> io::Result<Self> {
        let mut size = 0usize;
        // SAFETY: the documented first call asks for the required buffer size.
        unsafe { InitializeProcThreadAttributeList(null_mut(), count, 0, &mut size) };
        if size == 0 {
            return Err(io::Error::last_os_error());
        }
        let words = size
            .checked_add(size_of::<usize>() - 1)
            .ok_or_else(|| io::Error::other("attribute list size overflow"))?
            / size_of::<usize>();
        let mut storage = vec![0usize; words];
        let list = storage.as_mut_ptr().cast::<c_void>();
        // SAFETY: the aligned buffer has at least the requested size and lives
        // until after DeleteProcThreadAttributeList in Drop.
        if unsafe { InitializeProcThreadAttributeList(list, count, 0, &mut size) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self {
            _storage: storage,
            list,
        })
    }
}

impl Drop for Attributes {
    fn drop(&mut self) {
        // SAFETY: initialization succeeded and the backing storage is still live.
        unsafe { DeleteProcThreadAttributeList(self.list) };
    }
}

struct Pipes {
    child_input: Handle,
    parent_input: Handle,
    parent_output: Handle,
    child_output: Handle,
    child_error: Handle,
}

impl Pipes {
    fn new() -> io::Result<Self> {
        fn pipe() -> io::Result<(Handle, Handle)> {
            let attributes = SECURITY_ATTRIBUTES {
                nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
                lpSecurityDescriptor: null_mut(),
                bInheritHandle: 0,
            };
            let mut read = null_mut();
            let mut write = null_mut();
            // SAFETY: the output pointers and attributes are valid and writable.
            if unsafe { CreatePipe(&mut read, &mut write, &attributes, 0) } == 0 {
                return Err(io::Error::last_os_error());
            }
            Ok((Handle(read), Handle(write)))
        }
        fn inherited_duplicate(source: HANDLE) -> io::Result<Handle> {
            let mut duplicated = null_mut();
            // SAFETY: both process pseudo-handles are valid, and the caller
            // retains the source until DuplicateHandle returns.
            if unsafe {
                DuplicateHandle(
                    GetCurrentProcess(),
                    source,
                    GetCurrentProcess(),
                    &mut duplicated,
                    0,
                    1,
                    DUPLICATE_SAME_ACCESS,
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
            Ok(Handle(duplicated))
        }
        let (child_input_source, parent_input) = pipe()?;
        let (parent_output, child_output_source) = pipe()?;
        // Create parent ends as non-inheritable from the outset. Only short-
        // lived duplicates of the selected child ends are inheritable.
        let child_input = inherited_duplicate(child_input_source.0)?;
        let child_output = inherited_duplicate(child_output_source.0)?;
        // Keep diagnostics out of the framed stdout stream. Duplicate the
        // null sink as inheritable without changing the original handle.
        let null_file = std::fs::OpenOptions::new().write(true).open("NUL")?;
        let child_error = inherited_duplicate(null_file.as_raw_handle())?;
        Ok(Self {
            child_input,
            parent_input,
            parent_output,
            child_output,
            child_error,
        })
    }
}

/// One owned native process and its pre-associated job.
#[allow(dead_code)] // Parse activation is intentionally blocked until full admission exists.
pub(super) struct ManagedChild {
    job: Handle,
    process: Handle,
}

#[cfg_attr(test, allow(dead_code))]
#[derive(Clone, Copy)]
enum Environment {
    Inherited,
    Empty,
}

const EMPTY_ENVIRONMENT: [u16; 2] = [0, 0];

static PRIMARY_EXECUTABLE: OnceLock<(u32, u64)> = OnceLock::new();

fn file_identity(file: &std::fs::File) -> io::Result<(u32, u64)> {
    // SAFETY: zeroed POD; the handle is live for the duration of the call.
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { zeroed() };
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok((
        info.dwVolumeSerialNumber,
        (u64::from(info.nFileIndexHigh) << 32) | u64::from(info.nFileIndexLow),
    ))
}

/// Records which file this engine was started from, before any Parse can run.
pub(in crate::engine) fn capture_primary_executable() {
    if let Ok(identity) = std::env::current_exe()
        .and_then(std::fs::File::open)
        .and_then(|file| file_identity(&file))
    {
        let _ = PRIMARY_EXECUTABLE.set(identity);
    }
}

#[allow(dead_code)] // The Parse executor launches the worker.
/// Starts the private parser worker from the exact file this engine runs from.
/// The file stays open without write or delete sharing through the launch, so
/// the path cannot be replaced between the check and the process start.
pub(in crate::engine::attunement) fn spawn_worker(
    limit: u64,
) -> io::Result<(ManagedChild, std::fs::File, std::fs::File)> {
    let path = std::env::current_exe()?;
    let pinned = std::fs::File::options()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(&path)?;
    if PRIMARY_EXECUTABLE.get().copied() != Some(file_identity(&pinned)?) {
        return Err(io::Error::other(
            "the engine executable no longer matches the file it started from",
        ));
    }
    let spawned = ManagedChild::piped(&path, &[OsStr::new(WORKER_FLAG)], limit, Environment::Empty);
    drop(pinned);
    spawned
}

#[allow(dead_code)]
impl ManagedChild {
    pub(super) fn spawn(executable: &Path, args: &[&OsStr], limit: u64) -> io::Result<Self> {
        Self::spawn_inner(executable, args, limit, None, Environment::Inherited)
    }

    pub(super) fn spawn_piped(
        executable: &Path,
        args: &[&OsStr],
        limit: u64,
    ) -> io::Result<(Self, std::fs::File, std::fs::File)> {
        Self::piped(executable, args, limit, Environment::Inherited)
    }

    fn piped(
        executable: &Path,
        args: &[&OsStr],
        limit: u64,
        environment: Environment,
    ) -> io::Result<(Self, std::fs::File, std::fs::File)> {
        let _launch = rrd_core::child_launch_lock();
        let pipes = Pipes::new()?;
        let child = Self::spawn_inner(executable, args, limit, Some(&pipes), environment)?;
        let Pipes {
            parent_input,
            parent_output,
            ..
        } = pipes;
        Ok((child, parent_input.into_file(), parent_output.into_file()))
    }

    fn spawn_inner(
        executable: &Path,
        args: &[&OsStr],
        limit: u64,
        pipes: Option<&Pipes>,
        environment: Environment,
    ) -> io::Result<Self> {
        let memory = usize::try_from(limit)
            .ok()
            .filter(|value| *value > 0)
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "invalid job memory limit")
            })?;
        if !executable.is_absolute() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "absolute executable required",
            ));
        }
        let application = terminated(executable.as_os_str())?;
        let mut command = quoted(executable.as_os_str())?;
        for arg in args {
            command.push(u16::from(b' '));
            command.extend(quoted(arg)?);
        }
        command.push(0);

        // SAFETY: no security descriptor or job name is requested.
        let job = unsafe { CreateJobObjectW(null(), null()) };
        if job.is_null() {
            return Err(io::Error::last_os_error());
        }
        let job = Handle(job);
        // SAFETY: this POD structure is specified by Windows as zero-initializable.
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
            | JOB_OBJECT_LIMIT_PROCESS_MEMORY
            | JOB_OBJECT_LIMIT_JOB_MEMORY;
        limits.ProcessMemoryLimit = memory;
        limits.JobMemoryLimit = memory;
        // SAFETY: the information class matches the initialized structure and size.
        if unsafe {
            SetInformationJobObject(
                job.0,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }

        // The job handle array must outlive the attribute list and CreateProcessW.
        let jobs = [job.0];
        let inherited = pipes.map(|pipes| {
            [
                pipes.child_input.0,
                pipes.child_output.0,
                pipes.child_error.0,
            ]
        });
        let attributes = Attributes::new(if pipes.is_some() { 2 } else { 1 })?;
        // SAFETY: the list is initialized, and jobs remains live until it is deleted.
        if unsafe {
            UpdateProcThreadAttribute(
                attributes.list,
                0,
                PROC_THREAD_ATTRIBUTE_JOB_LIST as usize,
                jobs.as_ptr().cast(),
                size_of::<HANDLE>(),
                null_mut(),
                null(),
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        if let Some(handles) = &inherited {
            // SAFETY: all listed handles are inheritable and the array outlives
            // the attribute list and CreateProcessW.
            if unsafe {
                UpdateProcThreadAttribute(
                    attributes.list,
                    0,
                    PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                    handles.as_ptr().cast(),
                    std::mem::size_of_val(handles),
                    null_mut(),
                    null(),
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
        }
        // SAFETY: these native structures require zero initialization.
        let mut startup: STARTUPINFOEXW = unsafe { zeroed() };
        startup.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
        startup.lpAttributeList = attributes.list;
        if let Some(pipes) = pipes {
            startup.StartupInfo.dwFlags |= STARTF_USESTDHANDLES;
            startup.StartupInfo.hStdInput = pipes.child_input.0;
            startup.StartupInfo.hStdOutput = pipes.child_output.0;
            startup.StartupInfo.hStdError = pipes.child_error.0;
        }
        // SAFETY: PROCESS_INFORMATION is populated only after a successful call.
        let mut info: PROCESS_INFORMATION = unsafe { zeroed() };
        // SAFETY: application and mutable command are terminated UTF-16, the
        // attribute list and its handles remain live. In piped mode only the
        // explicit handle list is inherited.
        let created = unsafe {
            CreateProcessW(
                application.as_ptr(),
                command.as_mut_ptr(),
                null(),
                null(),
                i32::from(pipes.is_some()),
                CREATE_SUSPENDED
                    | CREATE_NO_WINDOW
                    | CREATE_UNICODE_ENVIRONMENT
                    | EXTENDED_STARTUPINFO_PRESENT,
                match environment {
                    Environment::Inherited => null(),
                    Environment::Empty => EMPTY_ENVIRONMENT.as_ptr().cast(),
                },
                null(),
                &startup.StartupInfo,
                &mut info,
            )
        };
        if created == 0 {
            return Err(io::Error::last_os_error());
        }
        let process = Handle(info.hProcess);
        let thread = Handle(info.hThread);
        let child = Self { job, process };
        // SAFETY: the primary thread handle is valid and exclusively owned.
        if unsafe { ResumeThread(thread.0) } == u32::MAX {
            return Err(io::Error::last_os_error());
        }
        Ok(child)
    }

    pub(super) fn wait(&mut self, timeout: Duration) -> io::Result<Option<u32>> {
        let millis = u32::try_from(timeout.as_millis())
            .unwrap_or(u32::MAX - 1)
            .min(u32::MAX - 1);
        // SAFETY: process is a live owned process handle; timeout is finite.
        match unsafe { WaitForSingleObject(self.process.0, millis) } {
            WAIT_TIMEOUT => Ok(None),
            WAIT_OBJECT_0 => {
                let mut code = 0u32;
                // SAFETY: code points to initialized writable storage.
                if unsafe { GetExitCodeProcess(self.process.0, &mut code) } == 0 {
                    Err(io::Error::last_os_error())
                } else {
                    Ok(Some(code))
                }
            }
            _ => Err(io::Error::last_os_error()),
        }
    }

    pub(super) fn terminate(&mut self) -> io::Result<()> {
        // SAFETY: job is an owned job handle; this affects only its processes.
        if unsafe { TerminateJobObject(self.job.0, 1) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

impl Drop for ManagedChild {
    fn drop(&mut self) {
        // A bounded wait avoids indefinitely blocking an engine drop. Closing
        // the job still triggers kill-on-close if termination is delayed.
        let _ = self.terminate();
        let _ = self.wait(Duration::from_secs(5));
    }
}

fn terminated(value: &OsStr) -> io::Result<Vec<u16>> {
    let mut wide: Vec<u16> = value.encode_wide().collect();
    if wide.contains(&0) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "embedded NUL"));
    }
    wide.push(0);
    Ok(wide)
}

// Quote according to the Windows C runtime argument decoding rules, including
// backslashes before a quote and at the end of an argument.
fn quoted(value: &OsStr) -> io::Result<Vec<u16>> {
    let units: Vec<u16> = value.encode_wide().collect();
    if units.contains(&0) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "embedded NUL"));
    }
    let mut output = vec![u16::from(b'"')];
    let mut slashes = 0usize;
    for unit in units {
        if unit == u16::from(b'\\') {
            slashes += 1;
        } else {
            let count = if unit == u16::from(b'"') {
                slashes * 2 + 1
            } else {
                slashes
            };
            output.extend(std::iter::repeat_n(u16::from(b'\\'), count));
            output.push(unit);
            slashes = 0;
        }
    }
    output.extend(std::iter::repeat_n(u16::from(b'\\'), slashes * 2));
    output.push(u16::from(b'"'));
    Ok(output)
}

#[cfg(test)]
mod tests;
