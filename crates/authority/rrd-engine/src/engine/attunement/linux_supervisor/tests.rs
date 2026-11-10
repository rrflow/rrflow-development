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
    ManagedChild::spawn(
        &std::env::current_exe()?,
        &[
            OsStr::new("--ignored"),
            OsStr::new("--exact"),
            OsStr::new(case),
        ],
        memory,
    )
}

#[test]
fn native_linux_supervisor_enforces_address_limit() {
    let mut child = start(
        "engine::attunement::linux_supervisor::tests::child_checks_limit",
        256 * 1024 * 1024,
    )
    .unwrap();
    assert_eq!(child.wait(Duration::from_secs(15)).unwrap(), Some(37));
}

#[test]
fn native_linux_supervisor_kills_child_when_supervisor_process_dies() {
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
    use std::time::Instant;

    let mut supervisor = start(
        "engine::attunement::linux_supervisor::tests::child_supervises_a_child",
        512 * 1024 * 1024,
    )
    .unwrap();
    let parent_pid = supervisor.child.id();
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
    // The fixture creates this file exclusively. Verify its PID belongs to the
    // spawned supervisor before opening a process handle or deleting the file.
    let status = std::fs::read_to_string(format!("/proc/{child_pid}/status")).unwrap();
    let parent_line = status
        .lines()
        .find(|line| line.starts_with("PPid:"))
        .unwrap();
    assert_eq!(parent_line[5..].trim().parse::<u32>().unwrap(), parent_pid);
    // SAFETY: pidfd_open creates a handle to the verified child; ownership is
    // transferred to OwnedFd only when the syscall succeeds.
    let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, child_pid, 0) } as i32;
    assert!(
        fd >= 0,
        "pidfd_open is required for this test: {}",
        std::io::Error::last_os_error()
    );
    let observed = unsafe { OwnedFd::from_raw_fd(fd) };
    supervisor.terminate().unwrap();
    assert!(supervisor.wait(Duration::from_secs(15)).unwrap().is_some());
    let mut wait = libc::pollfd {
        fd: observed.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    // SAFETY: wait points to one initialized pollfd with a live pidfd.
    assert_eq!(unsafe { libc::poll(&mut wait, 1, 15_000) }, 1);
    assert_ne!(
        wait.revents & (libc::POLLIN | libc::POLLHUP),
        0,
        "worker survived supervisor process death"
    );
    std::fs::remove_file(marker).unwrap();
}

fn marker_for(pid: u32) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("rrflow-linux-supervisor-{pid}.pid"))
}

#[test]
#[ignore = "subprocess fixture; run by native_linux_supervisor_kills_child_when_supervisor_process_dies"]
fn child_supervises_a_child() {
    use std::io::Write;
    let worker = start(
        "engine::attunement::linux_supervisor::tests::child_waits",
        256 * 1024 * 1024,
    )
    .unwrap();
    let mut marker = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(marker_for(std::process::id()))
        .unwrap();
    writeln!(marker, "{}", worker.child.id()).unwrap();
    marker.flush().unwrap();
    std::thread::sleep(Duration::from_secs(30));
    drop(worker);
}

#[test]
fn native_linux_supervisor_terminates_and_reaps_child() {
    let mut child = start(
        "engine::attunement::linux_supervisor::tests::child_waits",
        256 * 1024 * 1024,
    )
    .unwrap();
    assert_eq!(child.wait(Duration::from_millis(50)).unwrap(), None);
    child.terminate().unwrap();
    assert!(child.wait(Duration::from_secs(15)).unwrap().is_some());
}

#[test]
fn native_linux_supervisor_drop_terminates_owned_child() {
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
    let child = start(
        "engine::attunement::linux_supervisor::tests::child_waits",
        256 * 1024 * 1024,
    )
    .unwrap();
    // SAFETY: pidfd_open creates a descriptor for this live owned child.
    let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, child.child.id(), 0) } as i32;
    assert!(
        fd >= 0,
        "pidfd_open failed: {}",
        std::io::Error::last_os_error()
    );
    // SAFETY: the successful syscall returned an owned descriptor.
    let observed = unsafe { OwnedFd::from_raw_fd(fd) };
    drop(child);
    let mut wait = libc::pollfd {
        fd: observed.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    // SAFETY: wait describes a live pidfd and a finite timeout.
    assert_eq!(unsafe { libc::poll(&mut wait, 1, 15_000) }, 1);
    assert_ne!(wait.revents & (libc::POLLIN | libc::POLLHUP), 0);
}

#[test]
fn native_linux_supervisor_denies_zero_budget() {
    assert!(start(
        "engine::attunement::linux_supervisor::tests::child_waits",
        0
    )
    .is_err());
}

#[test]
fn native_linux_worker_survives_requesting_thread_exit() {
    let mut child = std::thread::spawn(|| {
        start(
            "engine::attunement::linux_supervisor::tests::child_waits",
            256 * 1024 * 1024,
        )
        .unwrap()
    })
    .join()
    .unwrap();
    assert_eq!(child.wait(Duration::from_millis(250)).unwrap(), None);
    child.terminate().unwrap();
    assert_eq!(child.wait(Duration::from_secs(15)).unwrap(), Some(137));
}

#[test]
#[ignore = "subprocess fixture; run by native_linux_supervisor_enforces_address_limit"]
fn child_checks_limit() {
    let mut limit = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    // SAFETY: limit points to writable storage for getrlimit.
    if unsafe { libc::getrlimit(libc::RLIMIT_AS, &mut limit) } != 0 {
        std::process::exit(41);
    }
    if limit.rlim_cur != 256 * 1024 * 1024 || limit.rlim_max != 256 * 1024 * 1024 {
        std::process::exit(42);
    }
    // SAFETY: an anonymous mapping is requested and its failure is checked.
    let allocation = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            1024 * 1024 * 1024,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        )
    };
    if allocation != libc::MAP_FAILED {
        // SAFETY: this is the successful mapping returned immediately above.
        unsafe { libc::munmap(allocation, 1024 * 1024 * 1024) };
        std::process::exit(45);
    }
    std::process::exit(37);
}

#[test]
#[ignore = "subprocess fixture; run by native_linux_supervisor_terminates_and_reaps_child"]
fn child_waits() {
    std::thread::sleep(Duration::from_secs(30));
}
