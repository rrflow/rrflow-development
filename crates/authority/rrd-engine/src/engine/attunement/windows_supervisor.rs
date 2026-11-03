//! Native containment foundation. This module is not a Parse executor: it has
//! no admitted executable identity, source pipes, deadline or publication.
//! Do not connect it to engine work until those boundaries are implemented.

use std::ffi::{c_void, OsStr};
use std::io;
use std::mem::{size_of, zeroed};
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::ptr::{null, null_mut};
use std::time::Duration;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows_sys::Win32::System::JobObjects::{
    CreateJobObjectW, JobObjectExtendedLimitInformation, SetInformationJobObject,
    TerminateJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_JOB_MEMORY,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOB_OBJECT_LIMIT_PROCESS_MEMORY,
};
use windows_sys::Win32::System::Threading::{
    CreateProcessW, DeleteProcThreadAttributeList, GetExitCodeProcess,
    InitializeProcThreadAttributeList, ResumeThread, UpdateProcThreadAttribute,
    WaitForSingleObject, CREATE_NO_WINDOW, CREATE_SUSPENDED, EXTENDED_STARTUPINFO_PRESENT,
    LPPROC_THREAD_ATTRIBUTE_LIST, PROCESS_INFORMATION, PROC_THREAD_ATTRIBUTE_JOB_LIST,
    STARTUPINFOEXW,
};

struct Handle(HANDLE);

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
    fn new() -> io::Result<Self> {
        let mut size = 0usize;
        // SAFETY: the documented first call asks for the required buffer size.
        unsafe { InitializeProcThreadAttributeList(null_mut(), 1, 0, &mut size) };
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
        if unsafe { InitializeProcThreadAttributeList(list, 1, 0, &mut size) } == 0 {
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

/// One owned native process and its pre-associated job. No handle is inherited.
#[allow(dead_code)] // Parse activation is intentionally blocked until full admission exists.
pub(super) struct ManagedChild {
    job: Handle,
    process: Handle,
}

#[allow(dead_code)]
impl ManagedChild {
    pub(super) fn spawn(executable: &Path, args: &[&OsStr], limit: u64) -> io::Result<Self> {
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
        let attributes = Attributes::new()?;
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
        // SAFETY: these native structures require zero initialization.
        let mut startup: STARTUPINFOEXW = unsafe { zeroed() };
        startup.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
        startup.lpAttributeList = attributes.list;
        // SAFETY: PROCESS_INFORMATION is populated only after a successful call.
        let mut info: PROCESS_INFORMATION = unsafe { zeroed() };
        // SAFETY: application and mutable command are terminated UTF-16, the
        // attribute list and its job handle remain live, and no handles inherit.
        let created = unsafe {
            CreateProcessW(
                application.as_ptr(),
                command.as_mut_ptr(),
                null(),
                null(),
                0,
                CREATE_SUSPENDED | CREATE_NO_WINDOW | EXTENDED_STARTUPINFO_PRESENT,
                null(),
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
