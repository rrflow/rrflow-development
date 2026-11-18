#[cfg(test)]
use super::Path;
use super::{denied, File, Metadata};
use std::io;
use std::os::unix::fs::{MetadataExt, PermissionsExt};

pub(super) fn open_empty_relative(
    parent: &File,
    name: &std::ffi::OsStr,
    create: bool,
) -> io::Result<File> {
    open_relative(parent, name, create, false)
}

pub(super) fn open_relative(
    parent: &File,
    name: &std::ffi::OsStr,
    create: bool,
    directory: bool,
) -> io::Result<File> {
    use std::os::fd::{AsRawFd, FromRawFd};
    use std::os::unix::ffi::OsStrExt;
    let name =
        std::ffi::CString::new(name.as_bytes()).map_err(|_| denied("private name contains NUL"))?;
    if create
        && directory
        && unsafe { libc::mkdirat(parent.as_raw_fd(), name.as_ptr(), 0o700) } != 0
    {
        return Err(io::Error::last_os_error());
    }
    let mut flags = libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC;
    if directory {
        flags |= libc::O_DIRECTORY | libc::O_RDONLY;
    } else if create {
        flags |= libc::O_RDWR | libc::O_CREAT | libc::O_EXCL;
    } else {
        flags |= libc::O_RDONLY;
    }
    let descriptor = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), flags, 0o600) };
    if descriptor < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: successful openat transfers this new descriptor to the File.
    Ok(unsafe { File::from_raw_fd(descriptor) })
}

#[cfg(test)]
pub(super) fn open_directory(path: &Path, create: bool) -> io::Result<File> {
    let path = rrd_store::filesystem::AnchoredPath::from_path(path)?;
    path.with_parent(|parent, name| open_relative(parent, name, create, true))
}

pub(super) fn validate_directory(file: &File) -> io::Result<()> {
    let metadata = file.metadata()?;
    // SAFETY: geteuid has no pointer arguments or preconditions.
    if !metadata.is_dir()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.permissions().mode() & 0o7777 != 0o700
    {
        return Err(denied(
            "engram directory requires effective-user ownership and mode 0700",
        ));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub(super) fn directory_security_sha256(file: &File) -> io::Result<String> {
    use std::os::fd::AsRawFd;
    fn acl(file: &File, name: &std::ffi::CStr) -> io::Result<Option<Vec<u8>>> {
        // A retained directory handle may be an O_PATH descriptor, which the
        // f*xattr calls refuse (EBADF). The kernel's per-descriptor link names
        // the same inode without resolving any project path.
        let link = std::ffi::CString::new(format!("/proc/self/fd/{}", file.as_raw_fd()))
            .map_err(io::Error::other)?;
        // SAFETY: link and name are NUL-terminated; the first call requests the length only.
        let size = unsafe { libc::getxattr(link.as_ptr(), name.as_ptr(), std::ptr::null_mut(), 0) };
        if size < 0 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::ENODATA) {
                return Ok(None);
            }
            return Err(error);
        }
        let size = usize::try_from(size).map_err(io::Error::other)?;
        // Linux bounds xattr values to XATTR_SIZE_MAX (64 KiB).
        if size > 65_536 {
            return Err(denied("directory ACL exceeds native bound"));
        }
        let mut bytes = vec![0u8; size];
        // SAFETY: bytes owns size writable bytes; the native function reports actual bytes.
        let actual = unsafe {
            libc::getxattr(
                link.as_ptr(),
                name.as_ptr(),
                bytes.as_mut_ptr().cast(),
                size,
            )
        };
        if actual < 0 {
            return Err(io::Error::last_os_error());
        }
        if usize::try_from(actual).map_err(io::Error::other)? != size {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "directory ACL changed while observed",
            ));
        }
        Ok(Some(bytes))
    }
    let metadata = file.metadata()?;
    let access = acl(file, c"system.posix_acl_access")?;
    let default = acl(file, c"system.posix_acl_default")?;
    let bytes = serde_json::to_vec(&(
        "rrflow-linux-directory-security-v1",
        metadata.uid(),
        metadata.gid(),
        metadata.mode(),
        access,
        default,
    ))
    .map_err(io::Error::other)?;
    Ok(rrd_core::digest::sha256_hex(&bytes))
}

/// The four `acl(3)` calls the retained-directory digest needs on macOS. The
/// `libc` crate does not bind them for Apple targets; libSystem provides them.
#[cfg(target_os = "macos")]
mod macos_acl {
    use std::ffi::{c_int, c_void};

    /// `ACL_TYPE_EXTENDED` from `<sys/acl.h>`: the one ACL type macOS stores.
    /// Its entries carry their own inheritance flags, so there is no separate
    /// default ACL to read as there is on Linux.
    pub(super) const ACL_TYPE_EXTENDED: c_int = 0x0000_0100;

    unsafe extern "C" {
        /// Returns an `acl_t` the caller frees, or null with `ENOENT` when the
        /// object has no ACL.
        pub(super) fn acl_get_fd_np(fd: c_int, kind: c_int) -> *mut c_void;
        /// Bytes `acl_copy_ext` needs for this ACL.
        pub(super) fn acl_size(acl: *mut c_void) -> libc::ssize_t;
        /// Writes the ACL's portable external form: entries, flags and
        /// principal identifiers, with no names and no directory lookup.
        pub(super) fn acl_copy_ext(
            buffer: *mut c_void,
            acl: *mut c_void,
            size: libc::ssize_t,
        ) -> libc::ssize_t;
        pub(super) fn acl_free(object: *mut c_void) -> c_int;
    }
}

/// Hash the owner, group, mode, file flags and extended ACL read from this
/// exact descriptor. Nothing is resolved by path and no principal is named.
#[cfg(target_os = "macos")]
pub(super) fn directory_security_sha256(file: &File) -> io::Result<String> {
    use std::os::fd::AsRawFd;
    use std::os::macos::fs::MetadataExt as _;

    /// Frees the native ACL on every path out of the read.
    struct NativeAcl(*mut std::ffi::c_void);
    impl Drop for NativeAcl {
        fn drop(&mut self) {
            // SAFETY: the pointer came from acl_get_fd_np and is freed once.
            unsafe { macos_acl::acl_free(self.0) };
        }
    }

    fn extended_acl(file: &File) -> io::Result<Option<Vec<u8>>> {
        // SAFETY: the descriptor is live for this borrow; the call takes no pointers.
        let native =
            unsafe { macos_acl::acl_get_fd_np(file.as_raw_fd(), macos_acl::ACL_TYPE_EXTENDED) };
        if native.is_null() {
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::ENOENT) {
                return Ok(None);
            }
            return Err(error);
        }
        let native = NativeAcl(native);
        // SAFETY: native.0 is a live acl_t.
        let size = unsafe { macos_acl::acl_size(native.0) };
        if size < 0 {
            return Err(io::Error::last_os_error());
        }
        let length = usize::try_from(size).map_err(io::Error::other)?;
        // The kernel bounds an ACL to 128 entries; this is the same bound the
        // Linux read applies to its attribute value.
        if length > 65_536 {
            return Err(denied("directory ACL exceeds native bound"));
        }
        let mut bytes = vec![0u8; length];
        // SAFETY: bytes owns `size` writable bytes and native.0 is a live acl_t.
        let written = unsafe { macos_acl::acl_copy_ext(bytes.as_mut_ptr().cast(), native.0, size) };
        if written < 0 {
            return Err(io::Error::last_os_error());
        }
        bytes.truncate(usize::try_from(written).map_err(io::Error::other)?);
        Ok(Some(bytes))
    }

    let metadata = file.metadata()?;
    let acl = extended_acl(file)?;
    let bytes = serde_json::to_vec(&(
        "rrflow-macos-directory-security-v1",
        metadata.uid(),
        metadata.gid(),
        metadata.mode(),
        metadata.st_flags(),
        acl,
    ))
    .map_err(io::Error::other)?;
    Ok(rrd_core::digest::sha256_hex(&bytes))
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub(super) fn directory_security_sha256(_: &File) -> io::Result<String> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "retained directory ACL inspection is not yet supported on this platform",
    ))
}

#[cfg(test)]
pub(super) fn open(path: &Path, create: bool) -> io::Result<File> {
    let path = rrd_store::filesystem::AnchoredPath::from_path(path)?;
    path.open_file_with(|parent, name| open_relative(parent, name, create, false))
}

pub(super) fn validate(file: &File) -> io::Result<Metadata> {
    let metadata = file.metadata()?;
    // SAFETY: geteuid has no pointer arguments or preconditions.
    let owner = unsafe { libc::geteuid() };
    if !metadata.is_file() || metadata.nlink() != 1 {
        return Err(denied(
            "secret must be a regular file with exactly one link",
        ));
    }
    if metadata.uid() != owner || metadata.permissions().mode() & 0o077 != 0 {
        return Err(denied(
            "secret must belong exclusively to the effective user",
        ));
    }
    Ok(metadata)
}

#[cfg(test)]
pub(super) fn make_directory_permissive_for_test(path: &Path) {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[cfg(test)]
pub(super) fn restore_private_directory_for_test(path: &Path) {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
}

#[cfg(test)]
mod tests {
    use super::super::{read_private_bounded, write_private_new};
    use super::*;

    #[test]
    #[ignore = "required native CI explicitly runs the privileged ownership fixture"]
    fn foreign_owner_is_denied_without_repair() {
        use std::os::fd::AsRawFd;
        assert_eq!(
            std::env::var("RRFLOW_PRIVATE_OWNER_FIXTURE").as_deref(),
            Ok("1")
        );
        // SAFETY: geteuid has no preconditions. Missing privilege is a failure.
        assert_eq!(unsafe { libc::geteuid() }, 0, "fixture requires root");
        let root = tempfile::tempdir().unwrap();
        let directory_path = root.path().join("private");
        let directory = open_directory(&directory_path, true).unwrap();
        let path = directory_path.join("secret");
        write_private_new(&path, b"unchanged ownership fixture").unwrap();
        let file = open(&path, false).unwrap();
        validate_directory(&directory).unwrap();
        validate(&file).unwrap();
        struct Restore<'a>(&'a File);
        impl Drop for Restore<'_> {
            fn drop(&mut self) {
                // SAFETY: this guard borrows only its retained test-owned handle.
                unsafe { libc::fchown(self.0.as_raw_fd(), 0, !0) };
            }
        }
        let _restore_directory = Restore(&directory);
        let _restore_file = Restore(&file);
        for object in [&directory, &file] {
            // SAFETY: live test-owned descriptors; UID 1 is deliberately not root.
            assert_eq!(unsafe { libc::fchown(object.as_raw_fd(), 1, !0) }, 0);
        }
        let directory_mode = directory.metadata().unwrap().mode();
        let file_mode = file.metadata().unwrap().mode();
        for _ in 0..2 {
            assert!(validate_directory(&directory).is_err());
            assert!(validate(&file).is_err());
            assert!(read_private_bounded(&path, 64).is_err());
            assert_eq!(directory.metadata().unwrap().uid(), 1);
            assert_eq!(file.metadata().unwrap().uid(), 1);
            assert_eq!(directory.metadata().unwrap().mode(), directory_mode);
            assert_eq!(file.metadata().unwrap().mode(), file_mode);
            assert_eq!(
                std::fs::read(&path).unwrap(),
                b"unchanged ownership fixture"
            );
        }
    }

    #[test]
    fn directory_modes_and_symlink_leaves_are_denied_without_repair() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("directory");
        let directory = open_directory(&path, true).unwrap();
        validate_directory(&directory).unwrap();
        for mode in [0o750, 0o755, 0o777, 0o1700, 0o2700] {
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).unwrap();
            assert!(validate_directory(&directory).is_err());
            assert_eq!(std::fs::metadata(&path).unwrap().mode() & 0o7777, mode);
        }
        restore_private_directory_for_test(&path);
        let alias = root.path().join("alias");
        std::os::unix::fs::symlink(&path, &alias).unwrap();
        assert!(open_directory(&alias, false).is_err());
        validate_directory(&directory).unwrap();
    }

    #[test]
    fn permissive_modes_and_symlinks_are_denied_without_repair() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("secret");
        write_private_new(&path, b"secret").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o640)).unwrap();
        assert!(read_private_bounded(&path, 6).is_err());
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o640
        );
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        let alias = root.path().join("alias");
        std::os::unix::fs::symlink(&path, &alias).unwrap();
        assert!(read_private_bounded(&alias, 6).is_err());
        assert_eq!(read_private_bounded(&path, 6).unwrap(), b"secret");
    }

    #[test]
    fn fifo_is_rejected_without_waiting_for_a_writer() {
        use std::os::unix::ffi::OsStrExt;
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("fifo");
        let name = std::ffi::CString::new(path.as_os_str().as_bytes()).unwrap();
        // SAFETY: name is a live NUL-terminated path in a test-owned directory.
        assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
        assert!(read_private_bounded(&path, 32).is_err());
    }
}
