//! Landlock filesystem sandboxing for agent VM processes.
//!
//! Uses raw syscalls — no `landlock` crate dependency. The Landlock LSM
//! restricts filesystem access to explicitly declared paths from the
//! agent manifest.

use crate::krun::VmConfig;
use crate::manifest::AgentManifest;
use std::os::unix::io::RawFd;
use std::path::PathBuf;

// ── Landlock syscall numbers (x86_64) ───────────────────────────────────────

const SYS_LANDLOCK_CREATE_RULESET: libc::c_long = 444;
const SYS_LANDLOCK_ADD_RULE: libc::c_long = 445;
const SYS_LANDLOCK_RESTRICT_SELF: libc::c_long = 446;

// ── Landlock ABI constants ──────────────────────────────────────────────────

const LANDLOCK_CREATE_RULESET_VERSION: u32 = 1 << 0;

// Access rights for files under Landlock ABI v1.
const LANDLOCK_ACCESS_FS_EXECUTE: u64 = 1 << 0;
const LANDLOCK_ACCESS_FS_WRITE_FILE: u64 = 1 << 1;
const LANDLOCK_ACCESS_FS_READ_FILE: u64 = 1 << 2;
const LANDLOCK_ACCESS_FS_READ_DIR: u64 = 1 << 3;
const LANDLOCK_ACCESS_FS_REMOVE_DIR: u64 = 1 << 4;
const LANDLOCK_ACCESS_FS_REMOVE_FILE: u64 = 1 << 5;
const LANDLOCK_ACCESS_FS_MAKE_CHAR: u64 = 1 << 6;
const LANDLOCK_ACCESS_FS_MAKE_DIR: u64 = 1 << 7;
const LANDLOCK_ACCESS_FS_MAKE_REG: u64 = 1 << 8;
const LANDLOCK_ACCESS_FS_MAKE_SOCK: u64 = 1 << 9;
const LANDLOCK_ACCESS_FS_MAKE_FIFO: u64 = 1 << 10;
const LANDLOCK_ACCESS_FS_MAKE_BLOCK: u64 = 1 << 11;
const LANDLOCK_ACCESS_FS_MAKE_SYM: u64 = 1 << 12;

/// All filesystem access rights handled by Landlock ABI v1.
const LANDLOCK_ACCESS_FS_ALL: u64 = LANDLOCK_ACCESS_FS_EXECUTE
    | LANDLOCK_ACCESS_FS_WRITE_FILE
    | LANDLOCK_ACCESS_FS_READ_FILE
    | LANDLOCK_ACCESS_FS_READ_DIR
    | LANDLOCK_ACCESS_FS_REMOVE_DIR
    | LANDLOCK_ACCESS_FS_REMOVE_FILE
    | LANDLOCK_ACCESS_FS_MAKE_CHAR
    | LANDLOCK_ACCESS_FS_MAKE_DIR
    | LANDLOCK_ACCESS_FS_MAKE_REG
    | LANDLOCK_ACCESS_FS_MAKE_SOCK
    | LANDLOCK_ACCESS_FS_MAKE_FIFO
    | LANDLOCK_ACCESS_FS_MAKE_BLOCK
    | LANDLOCK_ACCESS_FS_MAKE_SYM;

/// Read-only access mask.
const ACCESS_READ: u64 = LANDLOCK_ACCESS_FS_READ_FILE | LANDLOCK_ACCESS_FS_READ_DIR;

/// Read + execute access mask.
const ACCESS_READ_EXEC: u64 = ACCESS_READ | LANDLOCK_ACCESS_FS_EXECUTE;

/// Read + write access mask (includes creation and removal).
const ACCESS_READ_WRITE: u64 = LANDLOCK_ACCESS_FS_READ_FILE
    | LANDLOCK_ACCESS_FS_READ_DIR
    | LANDLOCK_ACCESS_FS_WRITE_FILE
    | LANDLOCK_ACCESS_FS_REMOVE_DIR
    | LANDLOCK_ACCESS_FS_REMOVE_FILE
    | LANDLOCK_ACCESS_FS_MAKE_DIR
    | LANDLOCK_ACCESS_FS_MAKE_REG
    | LANDLOCK_ACCESS_FS_MAKE_SOCK
    | LANDLOCK_ACCESS_FS_MAKE_FIFO
    | LANDLOCK_ACCESS_FS_MAKE_SYM;

const LANDLOCK_RULE_PATH_BENEATH: u32 = 1;

// ── Kernel ABI structs ──────────────────────────────────────────────────────

#[repr(C)]
struct LandlockRulesetAttr {
    handled_access_fs: u64,
}

#[repr(C)]
struct LandlockPathBeneathAttr {
    allowed_access: u64,
    parent_fd: RawFd,
}

// ── Public types ────────────────────────────────────────────────────────────

/// A single path rule: which path and what access to grant.
#[derive(Debug, Clone)]
pub struct PathRule {
    pub path: PathBuf,
    pub access: u64,
}

/// A collection of Landlock path rules to enforce.
#[derive(Debug, Clone)]
pub struct Sandbox {
    pub rules: Vec<PathRule>,
}

/// Errors from Landlock operations.
#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("Landlock not supported by kernel (ENOSYS)")]
    NotSupported,
    #[error("landlock_create_ruleset failed: {0}")]
    CreateRuleset(i64),
    #[error("landlock_add_rule failed for {path}: {errno}")]
    AddRule { path: String, errno: i64 },
    #[error("failed to open path for Landlock rule: {path}: {source}")]
    OpenPath {
        path: String,
        source: std::io::Error,
    },
    #[error("prctl(PR_SET_NO_NEW_PRIVS) failed: {0}")]
    NoNewPrivs(i32),
    #[error("landlock_restrict_self failed: {0}")]
    RestrictSelf(i64),
}

// ── Sandbox construction ────────────────────────────────────────────────────

/// Build a filesystem sandbox from the VM config and agent manifest.
pub fn build_sandbox(config: &VmConfig, manifest: &AgentManifest) -> Sandbox {
    let mut rules = Vec::new();

    // Root filesystem: read + execute
    rules.push(PathRule {
        path: PathBuf::from(&config.root_path),
        access: ACCESS_READ_EXEC,
    });

    // Virtiofs mounts: _ro suffix → read, _rw suffix → read+write
    for mount in &config.virtiofs_mounts {
        let access = if mount.tag.ends_with("_ro") {
            ACCESS_READ
        } else if mount.tag.ends_with("_rw") {
            ACCESS_READ_WRITE
        } else {
            ACCESS_READ
        };
        rules.push(PathRule {
            path: PathBuf::from(&mount.host_path),
            access,
        });
    }

    // Device nodes
    rules.push(PathRule {
        path: PathBuf::from("/dev/kvm"),
        access: LANDLOCK_ACCESS_FS_READ_FILE | LANDLOCK_ACCESS_FS_WRITE_FILE,
    });
    for dev in &["/dev/null", "/dev/zero", "/dev/urandom"] {
        rules.push(PathRule {
            path: PathBuf::from(dev),
            access: LANDLOCK_ACCESS_FS_READ_FILE,
        });
    }

    // Agent log directory
    rules.push(PathRule {
        path: PathBuf::from("/var/log/cage/"),
        access: ACCESS_READ_WRITE,
    });

    // Shared libraries (host linker paths)
    rules.push(PathRule {
        path: PathBuf::from("/usr/lib/"),
        access: ACCESS_READ,
    });
    rules.push(PathRule {
        path: PathBuf::from("/lib64/"),
        access: ACCESS_READ,
    });

    // Proc and sys — libkrun VMM reads CPU topology, NUMA info, etc.
    rules.push(PathRule {
        path: PathBuf::from("/proc/"),
        access: ACCESS_READ,
    });
    rules.push(PathRule {
        path: PathBuf::from("/sys/"),
        access: ACCESS_READ,
    });

    // Tmp — libkrun may create temporary files
    rules.push(PathRule {
        path: PathBuf::from("/tmp/"),
        access: ACCESS_READ_WRITE,
    });

    // Run — for sockets and runtime state
    rules.push(PathRule {
        path: PathBuf::from("/run/"),
        access: ACCESS_READ_WRITE,
    });

    // Additional read paths from manifest
    for path in &manifest.capabilities.filesystem.read {
        rules.push(PathRule {
            path: PathBuf::from(path),
            access: ACCESS_READ,
        });
    }

    // Additional write paths from manifest
    for path in &manifest.capabilities.filesystem.write {
        rules.push(PathRule {
            path: PathBuf::from(path),
            access: ACCESS_READ_WRITE,
        });
    }

    Sandbox { rules }
}

// ── Landlock syscall wrappers ───────────────────────────────────────────────

/// Detect the Landlock ABI version. Returns 0 if unsupported.
fn landlock_abi_version() -> i64 {
    unsafe {
        libc::syscall(
            SYS_LANDLOCK_CREATE_RULESET,
            std::ptr::null::<LandlockRulesetAttr>(),
            0usize,
            LANDLOCK_CREATE_RULESET_VERSION,
        )
    }
}

fn landlock_create_ruleset(handled_access_fs: u64) -> Result<RawFd, SandboxError> {
    let attr = LandlockRulesetAttr { handled_access_fs };
    let fd = unsafe {
        libc::syscall(
            SYS_LANDLOCK_CREATE_RULESET,
            &attr as *const LandlockRulesetAttr,
            std::mem::size_of::<LandlockRulesetAttr>(),
            0u32,
        )
    };
    if fd < 0 {
        if fd == -(libc::ENOSYS as i64) {
            return Err(SandboxError::NotSupported);
        }
        return Err(SandboxError::CreateRuleset(fd));
    }
    Ok(fd as RawFd)
}

fn landlock_add_rule(ruleset_fd: RawFd, path: &PathRule) -> Result<(), SandboxError> {
    use std::os::unix::io::AsRawFd;

    let file = std::fs::File::open(&path.path).map_err(|e| SandboxError::OpenPath {
        path: path.path.display().to_string(),
        source: e,
    })?;

    let attr = LandlockPathBeneathAttr {
        allowed_access: path.access,
        parent_fd: file.as_raw_fd(),
    };

    let ret = unsafe {
        libc::syscall(
            SYS_LANDLOCK_ADD_RULE,
            ruleset_fd,
            LANDLOCK_RULE_PATH_BENEATH,
            &attr as *const LandlockPathBeneathAttr,
            0u32,
        )
    };

    if ret < 0 {
        return Err(SandboxError::AddRule {
            path: path.path.display().to_string(),
            errno: ret,
        });
    }

    Ok(())
}

fn landlock_restrict_self(ruleset_fd: RawFd) -> Result<(), SandboxError> {
    let ret =
        unsafe { libc::syscall(SYS_LANDLOCK_RESTRICT_SELF, ruleset_fd, 0u32) };
    if ret < 0 {
        return Err(SandboxError::RestrictSelf(ret));
    }
    Ok(())
}

// ── Apply ───────────────────────────────────────────────────────────────────

/// Apply the Landlock sandbox to the current process.
///
/// If the kernel does not support Landlock (`ENOSYS`), logs a warning
/// and returns `Ok(())` — graceful degradation for development kernels.
pub fn apply(sandbox: &Sandbox) -> Result<(), SandboxError> {
    // Check Landlock support
    let abi = landlock_abi_version();
    if abi <= 0 {
        log::warn!("Landlock not supported (ABI version check returned {abi}), skipping");
        return Ok(());
    }
    log::info!("Landlock ABI version {abi} detected");

    // Create ruleset covering all filesystem access types
    let ruleset_fd = landlock_create_ruleset(LANDLOCK_ACCESS_FS_ALL)?;

    // Add each path rule, skipping paths that don't exist on the host
    for rule in &sandbox.rules {
        if rule.path.exists() {
            landlock_add_rule(ruleset_fd, rule)?;
        } else {
            log::debug!(
                "Landlock: skipping non-existent path {}",
                rule.path.display()
            );
        }
    }

    // PR_SET_NO_NEW_PRIVS is required before enforcing Landlock
    let ret = unsafe { libc::prctl(38 /* PR_SET_NO_NEW_PRIVS */, 1, 0, 0, 0) };
    if ret != 0 {
        unsafe { libc::close(ruleset_fd) };
        return Err(SandboxError::NoNewPrivs(ret));
    }

    // Enforce
    let result = landlock_restrict_self(ruleset_fd);
    unsafe { libc::close(ruleset_fd) };
    result
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::krun::{VirtiofsMount, VmConfig};
    use crate::manifest;

    fn test_config() -> VmConfig {
        VmConfig {
            name: "test-agent".to_string(),
            vcpus: 2,
            ram_mib: 512,
            root_path: "/system/rootfs/test-agent".to_string(),
            exec_path: "/agent/bin/test-agent".to_string(),
            args: vec![],
            env: vec![],
            port_map: vec![],
            workdir: "/".to_string(),
            virtiofs_mounts: vec![
                VirtiofsMount {
                    tag: "data_corpus_ro".to_string(),
                    host_path: "/var/lib/cage/test-agent/data/corpus".to_string(),
                },
                VirtiofsMount {
                    tag: "data_output_rw".to_string(),
                    host_path: "/var/lib/cage/test-agent/data/output".to_string(),
                },
            ],
            rlimits: vec![],
            console_output: None,
            manifest_toml: None,
        }
    }

    fn test_manifest() -> AgentManifest {
        manifest::parse(
            r#"
            [agent]
            name = "test-agent"

            [capabilities.filesystem]
            read = ["/data/corpus"]
            write = ["/data/output"]
            "#,
        )
        .unwrap()
    }

    #[test]
    fn test_sandbox_includes_rootfs() {
        let config = test_config();
        let manifest = test_manifest();
        let sandbox = build_sandbox(&config, &manifest);

        let rootfs_rule = sandbox
            .rules
            .iter()
            .find(|r| r.path == PathBuf::from("/system/rootfs/test-agent"));
        assert!(rootfs_rule.is_some(), "sandbox must include rootfs path");

        let rule = rootfs_rule.unwrap();
        assert!(
            rule.access & LANDLOCK_ACCESS_FS_READ_FILE != 0,
            "rootfs must have read access"
        );
        assert!(
            rule.access & LANDLOCK_ACCESS_FS_EXECUTE != 0,
            "rootfs must have execute access"
        );
    }

    #[test]
    fn test_read_paths_no_write() {
        let config = test_config();
        let manifest = test_manifest();
        let sandbox = build_sandbox(&config, &manifest);

        // Find the _ro virtiofs mount rule
        let ro_rule = sandbox
            .rules
            .iter()
            .find(|r| r.path == PathBuf::from("/var/lib/cage/test-agent/data/corpus"));
        assert!(ro_rule.is_some(), "sandbox must include read-only mount");

        let rule = ro_rule.unwrap();
        assert!(
            rule.access & LANDLOCK_ACCESS_FS_READ_FILE != 0,
            "ro mount must have read access"
        );
        assert_eq!(
            rule.access & LANDLOCK_ACCESS_FS_WRITE_FILE,
            0,
            "ro mount must NOT have write access"
        );
    }

    #[test]
    fn test_kvm_always_included() {
        let config = test_config();
        let manifest = test_manifest();
        let sandbox = build_sandbox(&config, &manifest);

        let kvm_rule = sandbox
            .rules
            .iter()
            .find(|r| r.path == PathBuf::from("/dev/kvm"));
        assert!(kvm_rule.is_some(), "sandbox must always include /dev/kvm");

        let rule = kvm_rule.unwrap();
        assert!(rule.access & LANDLOCK_ACCESS_FS_READ_FILE != 0);
        assert!(rule.access & LANDLOCK_ACCESS_FS_WRITE_FILE != 0);
    }

    #[test]
    fn test_rw_mount_has_write() {
        let config = test_config();
        let manifest = test_manifest();
        let sandbox = build_sandbox(&config, &manifest);

        let rw_rule = sandbox
            .rules
            .iter()
            .find(|r| r.path == PathBuf::from("/var/lib/cage/test-agent/data/output"));
        assert!(rw_rule.is_some(), "sandbox must include read-write mount");

        let rule = rw_rule.unwrap();
        assert!(
            rule.access & LANDLOCK_ACCESS_FS_WRITE_FILE != 0,
            "rw mount must have write access"
        );
        assert!(
            rule.access & LANDLOCK_ACCESS_FS_READ_FILE != 0,
            "rw mount must have read access"
        );
    }

    #[test]
    fn test_dev_nodes_read_only() {
        let config = test_config();
        let manifest = test_manifest();
        let sandbox = build_sandbox(&config, &manifest);

        for dev in ["/dev/null", "/dev/zero", "/dev/urandom"] {
            let rule = sandbox
                .rules
                .iter()
                .find(|r| r.path == PathBuf::from(dev));
            assert!(rule.is_some(), "{dev} must be in sandbox");
            let rule = rule.unwrap();
            assert!(rule.access & LANDLOCK_ACCESS_FS_READ_FILE != 0);
            assert_eq!(
                rule.access & LANDLOCK_ACCESS_FS_WRITE_FILE,
                0,
                "{dev} must be read-only"
            );
        }
    }
}
