use super::ManagedChild;
use std::ffi::OsStr;
use std::time::Duration;

#[test]
fn parent_death_marker_requires_complete_pid() {
    assert_eq!(completed_pid(""), None);
    assert_eq!(completed_pid("12"), None);
    assert_eq!(completed_pid("12x\n"), None);
    assert_eq!(completed_pid("0\n"), None);
    assert_eq!(completed_pid("123\n"), Some(123));
}

fn completed_pid(value: &str) -> Option<u32> {
    let digits = value.strip_suffix('\n')?;
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok().filter(|pid| *pid != 0)
}

fn start(case: &str, memory: u64) -> std::io::Result<ManagedChild> {
    let executable = std::env::current_exe()?;
    ManagedChild::spawn(
        &executable,
        &[
            OsStr::new("--ignored"),
            OsStr::new("--exact"),
            OsStr::new(case),
        ],
        memory,
    )
}

fn start_piped(case: &str) -> std::io::Result<(ManagedChild, std::fs::File, std::fs::File)> {
    ManagedChild::spawn_piped(
        &std::env::current_exe()?,
        &[
            OsStr::new("--ignored"),
            OsStr::new("--exact"),
            OsStr::new(case),
        ],
        256 * 1024 * 1024,
    )
}

#[test]
fn native_job_transports_framed_parser_request_without_unrelated_handle_inheritance() {
    use rrd_attunement::parse::ParseLimits;
    use rrd_attunement::worker::{read_response, write_request, WorkerRequest, WorkerResponse};
    use std::io::{Read, Write};
    use windows_sys::Win32::Foundation::WAIT_TIMEOUT;
    use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
    use windows_sys::Win32::System::Threading::{CreateEventW, WaitForSingleObject};

    let security = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: std::ptr::null_mut(),
        bInheritHandle: 1,
    };
    // SAFETY: the initialized attributes create an unnamed inheritable event.
    let event = unsafe { CreateEventW(&security, 1, 0, std::ptr::null()) };
    assert!(!event.is_null());
    let event = super::Handle(event);
    let (mut child, mut input, mut output) =
        start_piped("engine::attunement::windows_supervisor::tests::child_runs_framed_parser")
            .unwrap();
    input
        .write_all(&(event.0 as usize as u64).to_le_bytes())
        .unwrap();
    let source = b"pub fn supervised_symbol() {}\n";
    let request = WorkerRequest {
        format_version: 1,
        source_id: "fixture/source.rs".into(),
        content_sha256: rrd_core::digest::sha256_hex(source),
        language: "rust".into(),
        source_bytes: source.len() as u64,
        limits: ParseLimits {
            input_bytes: 1024,
            nodes: 512,
            depth: 64,
            output_bytes: 65536,
            elapsed_micros: 5_000_000,
        },
    };
    write_request(&mut input, &request, source).unwrap();
    drop(input);
    let status = child.wait(Duration::from_secs(15)).unwrap();
    if status.is_none() {
        child.terminate().unwrap();
        let _ = child.wait(Duration::from_secs(15));
        panic!("supervised parser child exceeded its test deadline");
    }
    assert_eq!(status, Some(0));
    // libtest prints its own preamble; the child fixture writes this delimiter
    // immediately before the actual binary protocol and exits without a footer.
    let mut prefix = Vec::new();
    let mut byte = [0];
    while !prefix.ends_with(b"RR-PARSER-FRAME\n") {
        output.read_exact(&mut byte).unwrap();
        prefix.push(byte[0]);
        assert!(prefix.len() <= 1024, "unexpected child test preamble");
    }
    let WorkerResponse::Parsed { proposal } = read_response(&mut output, &request).unwrap() else {
        panic!("supervised child did not return parsed evidence");
    };
    assert!(proposal.nodes.iter().any(|node| {
        &source[node.start_byte as usize..node.end_byte as usize] == b"supervised_symbol"
    }));
    // SAFETY: event is still the live parent handle. A listed child handle is
    // not this event, so the child must not have been able to signal it.
    let observed = unsafe { WaitForSingleObject(event.0, 0) };
    assert_eq!(observed, WAIT_TIMEOUT, "unrelated inheritable event leaked");
}

#[test]
fn native_job_termination_releases_writer_blocked_on_child_input() {
    use std::io::Write;
    let (mut child, mut input, _output) =
        start_piped("engine::attunement::windows_supervisor::tests::child_waits").unwrap();
    let (send, receive) = std::sync::mpsc::sync_channel(1);
    let (started, ready) = std::sync::mpsc::sync_channel(1);
    let writer = std::thread::spawn(move || {
        let payload = vec![0; 16 * 1024 * 1024];
        let _ = started.send(());
        let result = input.write_all(&payload);
        let _ = send.send(result);
    });
    ready.recv_timeout(Duration::from_secs(15)).unwrap();
    assert!(receive.recv_timeout(Duration::from_millis(100)).is_err());
    child.terminate().unwrap();
    assert!(child.wait(Duration::from_secs(15)).unwrap().is_some());
    assert!(receive
        .recv_timeout(Duration::from_secs(15))
        .unwrap()
        .is_err());
    writer.join().unwrap();
}

#[test]
#[ignore = "subprocess fixture; run by native_job_transports_framed_parser_request_without_unrelated_handle_inheritance"]
fn child_runs_framed_parser() {
    use std::io::{Read, Write};
    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    let mut raw = [0; 8];
    stdin.read_exact(&mut raw).unwrap();
    let event = u64::from_le_bytes(raw) as usize as windows_sys::Win32::Foundation::HANDLE;
    // SAFETY: deliberately probe the parent's numeric handle; without
    // inheritance Windows must reject it or it must refer to a different object.
    unsafe { windows_sys::Win32::System::Threading::SetEvent(event) };
    stdout.write_all(b"RR-PARSER-FRAME\n").unwrap();
    stdout.flush().unwrap();
    rrd_attunement::worker::run_worker(&mut stdin, &mut stdout).unwrap();
    std::process::exit(0);
}

#[test]
#[ignore = "requires an explicitly selected source-built rrflow executable"]
fn native_job_runs_primary_binary_parser_worker() {
    use rrd_attunement::parse::ParseLimits;
    use rrd_attunement::worker::{read_response, write_request, WorkerRequest, WorkerResponse};
    let executable = std::path::PathBuf::from(
        std::env::var_os("RRFLOW_SUPERVISOR_TEST_EXECUTABLE")
            .expect("select the exact source-built primary executable"),
    );
    let (mut child, mut input, mut output) = ManagedChild::spawn_piped(
        &executable,
        &[OsStr::new("--rrflow-internal-parse-worker-v1")],
        512 * 1024 * 1024,
    )
    .unwrap();
    let mut assigned = 0;
    // SAFETY: both handles are owned and assigned is writable BOOL storage.
    assert_ne!(
        unsafe {
            windows_sys::Win32::System::JobObjects::IsProcessInJob(
                child.process.0,
                child.job.0,
                &mut assigned,
            )
        },
        0
    );
    assert_ne!(assigned, 0);
    let source = b"pub fn actual_binary_symbol() {}\n";
    let request = WorkerRequest {
        format_version: 1,
        source_id: "fixture/source.rs".into(),
        content_sha256: rrd_core::digest::sha256_hex(source),
        language: "rust".into(),
        source_bytes: source.len() as u64,
        limits: ParseLimits {
            input_bytes: 1024,
            nodes: 512,
            depth: 64,
            output_bytes: 65536,
            elapsed_micros: 5_000_000,
        },
    };
    write_request(&mut input, &request, source).unwrap();
    drop(input);
    let status = child.wait(Duration::from_secs(15)).unwrap();
    if status.is_none() {
        child.terminate().unwrap();
        let _ = child.wait(Duration::from_secs(15));
        panic!("primary parser worker exceeded its test deadline");
    }
    assert_eq!(status, Some(0));
    let WorkerResponse::Parsed { proposal } = read_response(&mut output, &request).unwrap() else {
        panic!("primary parser worker did not return parsed evidence");
    };
    assert!(proposal.nodes.iter().any(|node| {
        &source[node.start_byte as usize..node.end_byte as usize] == b"actual_binary_symbol"
    }));
}

#[test]
fn native_job_enforces_memory_before_child_runs() {
    let mut child = start(
        "engine::attunement::windows_supervisor::tests::child_checks_job_memory",
        128 * 1024 * 1024,
    )
    .unwrap();
    // The child exits with a distinct code after its native assertions, so a
    // successful test harness with zero matching tests cannot pass this oracle.
    assert_eq!(child.wait(Duration::from_secs(15)).unwrap(), Some(37));
}

#[test]
fn native_job_terminates_and_reaps_child() {
    let mut child = start(
        "engine::attunement::windows_supervisor::tests::child_waits",
        128 * 1024 * 1024,
    )
    .unwrap();
    assert_eq!(child.wait(Duration::from_millis(50)).unwrap(), None);
    let mut assigned = 0;
    // SAFETY: both handles are live and assigned is writable BOOL storage.
    assert_ne!(
        unsafe {
            windows_sys::Win32::System::JobObjects::IsProcessInJob(
                child.process.0,
                child.job.0,
                &mut assigned,
            )
        },
        0
    );
    assert_ne!(assigned, 0, "child was not associated with this job");
    child.terminate().unwrap();
    assert!(child.wait(Duration::from_secs(15)).unwrap().is_some());
}

#[test]
fn native_job_rejects_zero_limit_before_process_creation() {
    assert!(start(
        "engine::attunement::windows_supervisor::tests::child_waits",
        0
    )
    .is_err());
}

#[test]
fn native_job_drop_terminates_the_owned_child() {
    use windows_sys::Win32::Foundation::{
        CloseHandle, DuplicateHandle, DUPLICATE_SAME_ACCESS, WAIT_OBJECT_0,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, WaitForSingleObject};
    let child = start(
        "engine::attunement::windows_supervisor::tests::child_waits",
        128 * 1024 * 1024,
    )
    .unwrap();
    let mut observed = std::ptr::null_mut();
    // SAFETY: duplicate the live owned process handle into this process; the
    // duplicate remains valid after ManagedChild drops its own handle.
    assert_ne!(
        unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                child.process.0,
                GetCurrentProcess(),
                &mut observed,
                0,
                0,
                DUPLICATE_SAME_ACCESS,
            )
        },
        0
    );
    drop(child);
    // SAFETY: observed is a successful duplicate and is closed once below.
    let result = unsafe { WaitForSingleObject(observed, 15_000) };
    unsafe { CloseHandle(observed) };
    assert_eq!(
        result, WAIT_OBJECT_0,
        "owned child survived supervisor drop"
    );
}

#[test]
fn native_job_kills_child_when_its_supervisor_process_dies() {
    use std::time::Instant;
    use windows_sys::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0};
    use windows_sys::Win32::System::Threading::{
        GetProcessId, OpenProcess, TerminateProcess, WaitForSingleObject,
        PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
    };

    let supervisor = start(
        "engine::attunement::windows_supervisor::tests::child_supervises_a_child",
        512 * 1024 * 1024,
    )
    .unwrap();
    // SAFETY: supervisor owns a live process handle.
    let parent_pid = unsafe { GetProcessId(supervisor.process.0) };
    assert_ne!(parent_pid, 0);
    let marker = marker_for(parent_pid);
    let started = Instant::now();
    let child_pid = loop {
        if let Ok(contents) = std::fs::read_to_string(&marker) {
            if let Some(pid) = completed_pid(&contents) {
                break pid;
            }
        }
        assert!(
            started.elapsed() < Duration::from_secs(15),
            "child supervisor did not report its worker"
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    // SAFETY: open a separate wait handle before terminating the supervisor.
    let observed = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            0,
            child_pid,
        )
    };
    assert!(
        !observed.is_null(),
        "worker disappeared before parent-death test"
    );
    assert_ne!(child_pid, parent_pid);
    let mut in_outer_job = 0;
    // SAFETY: both handles are live; this verifies the reported PID belongs
    // to the outer job rather than accepting a stale marker or foreign process.
    assert_ne!(
        unsafe {
            windows_sys::Win32::System::JobObjects::IsProcessInJob(
                observed,
                supervisor.job.0,
                &mut in_outer_job,
            )
        },
        0
    );
    assert_ne!(in_outer_job, 0);
    // Terminate only the middle process. Keep the outer job alive: closing the
    // inner job on process death, not dropping the outer job, must kill worker.
    // SAFETY: the process handle is live and owned by supervisor.
    assert_ne!(unsafe { TerminateProcess(supervisor.process.0, 42) }, 0);
    // SAFETY: observed remains a valid process handle until closed below.
    let result = unsafe { WaitForSingleObject(observed, 15_000) };
    unsafe { CloseHandle(observed) };
    std::fs::remove_file(&marker).unwrap();
    assert_eq!(
        result, WAIT_OBJECT_0,
        "worker survived supervisor process death"
    );
    drop(supervisor);
}

fn marker_for(pid: u32) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("rrflow-native-supervisor-{pid}.pid"))
}

#[test]
#[ignore = "subprocess fixture; run by native_job_kills_child_when_its_supervisor_process_dies"]
fn child_supervises_a_child() {
    use std::io::Write;
    use windows_sys::Win32::System::Threading::GetProcessId;
    let worker = start(
        "engine::attunement::windows_supervisor::tests::child_waits",
        128 * 1024 * 1024,
    )
    .unwrap();
    // SAFETY: worker owns a live process handle.
    let pid = unsafe { GetProcessId(worker.process.0) };
    assert_ne!(pid, 0);
    let mut marker = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(marker_for(std::process::id()))
        .unwrap();
    writeln!(marker, "{pid}").unwrap();
    marker.flush().unwrap();
    std::thread::sleep(Duration::from_secs(30));
    drop(worker);
}

#[test]
#[ignore = "subprocess fixture; run by native_job_enforces_memory_before_child_runs"]
fn child_checks_job_memory() {
    use windows_sys::Win32::System::JobObjects::{
        IsProcessInJob, JobObjectExtendedLimitInformation, QueryInformationJobObject,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_JOB_MEMORY,
        JOB_OBJECT_LIMIT_PROCESS_MEMORY,
    };
    use windows_sys::Win32::System::Memory::{
        VirtualAlloc, MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE,
    };
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    let mut assigned = 0;
    // SAFETY: the pseudo-handle is valid, null asks whether this process is in
    // any job, and assigned points to writable BOOL storage.
    assert_ne!(
        unsafe { IsProcessInJob(GetCurrentProcess(), std::ptr::null_mut(), &mut assigned) },
        0
    );
    assert_ne!(assigned, 0);
    // Do not attempt a large allocation unless this process's immediate job
    // actually has the test's committed-memory limits. This also makes an
    // accidental direct run of the ignored fixture fail before allocation.
    let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    // SAFETY: a null job selects this process's immediate job; limits points
    // to correctly sized writable storage for the requested information class.
    assert_ne!(
        unsafe {
            QueryInformationJobObject(
                std::ptr::null_mut(),
                JobObjectExtendedLimitInformation,
                (&mut limits as *mut JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                std::ptr::null_mut(),
            )
        },
        0
    );
    assert_eq!(limits.ProcessMemoryLimit, 128 * 1024 * 1024);
    assert_eq!(limits.JobMemoryLimit, 128 * 1024 * 1024);
    assert_eq!(
        limits.BasicLimitInformation.LimitFlags
            & (JOB_OBJECT_LIMIT_PROCESS_MEMORY | JOB_OBJECT_LIMIT_JOB_MEMORY),
        JOB_OBJECT_LIMIT_PROCESS_MEMORY | JOB_OBJECT_LIMIT_JOB_MEMORY,
    );
    // SAFETY: request a fresh allocation and check failure; no pointer is dereferenced.
    let allocation = unsafe {
        VirtualAlloc(
            std::ptr::null(),
            1024 * 1024 * 1024,
            MEM_RESERVE | MEM_COMMIT,
            PAGE_READWRITE,
        )
    };
    assert!(
        allocation.is_null(),
        "job memory limit did not deny commitment"
    );
    std::process::exit(37);
}

#[test]
#[ignore = "subprocess fixture; run by native_job_terminates_and_reaps_child"]
fn child_waits() {
    std::thread::sleep(Duration::from_secs(30));
}
