//! Per-agent seccomp-BPF syscall filtering.
//!
//! Builds a BPF whitelist from the agent manifest and applies it via
//! `prctl(PR_SET_SECCOMP, SECCOMP_MODE_FILTER)`. No external crate —
//! raw BPF bytecode assembled from `libc` constants.

use crate::manifest::AgentManifest;
use std::collections::BTreeSet;

/// A set of allowed syscall numbers (x86_64).
#[derive(Debug, Clone)]
pub struct SeccompProfile {
    pub allowed: BTreeSet<i64>,
}

/// Errors from seccomp operations.
#[derive(Debug, thiserror::Error)]
pub enum SeccompError {
    #[error("prctl(PR_SET_NO_NEW_PRIVS) failed: {0}")]
    NoNewPrivs(i32),
    #[error("prctl(PR_SET_SECCOMP) failed: {0}")]
    SetSeccomp(i32),
    #[error("too many allowed syscalls for BPF program ({0}, max 255)")]
    TooManySyscalls(usize),
}

// ── BPF instruction encoding ────────────────────────────────────────────────

/// A single BPF instruction (struct sock_filter).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct SockFilter {
    code: u16,
    jt: u8,
    jf: u8,
    k: u32,
}

/// A BPF program header (struct sock_fprog).
#[repr(C)]
struct SockFprog {
    len: u16,
    filter: *const SockFilter,
}

// BPF opcodes
const BPF_LD: u16 = 0x00;
const BPF_W: u16 = 0x00;
const BPF_ABS: u16 = 0x20;
const BPF_JMP: u16 = 0x05;
const BPF_JEQ: u16 = 0x10;
const BPF_RET: u16 = 0x06;
const BPF_K: u16 = 0x00;

// seccomp constants
const SECCOMP_RET_ALLOW: u32 = 0x7fff_0000;
const SECCOMP_RET_LOG: u32 = 0x7ffc_0000;
const SECCOMP_RET_KILL_PROCESS: u32 = 0x8000_0000;

// seccomp_data layout: nr is at offset 0
const SECCOMP_DATA_NR_OFFSET: u32 = 0;

// prctl constants (may not be in libc on all targets)
const PR_SET_NO_NEW_PRIVS: libc::c_int = 38;
const PR_SET_SECCOMP: libc::c_int = 22;
const SECCOMP_MODE_FILTER: libc::c_ulong = 2;

fn bpf_stmt(code: u16, k: u32) -> SockFilter {
    SockFilter {
        code,
        jt: 0,
        jf: 0,
        k,
    }
}

fn bpf_jump(code: u16, k: u32, jt: u8, jf: u8) -> SockFilter {
    SockFilter { code, jt, jf, k }
}

// ── VMM base syscalls ───────────────────────────────────────────────────────

/// Core syscalls every agent VM process needs.
const VMM_BASE: &[i64] = &[
    libc::SYS_read,
    libc::SYS_write,
    libc::SYS_close,
    libc::SYS_mmap,
    libc::SYS_mprotect,
    libc::SYS_munmap,
    libc::SYS_brk,
    libc::SYS_ioctl,
    libc::SYS_openat,
    libc::SYS_newfstatat,
    libc::SYS_lseek,
    libc::SYS_pread64,
    libc::SYS_pwrite64,
    libc::SYS_readv,
    libc::SYS_writev,
    libc::SYS_access,
    libc::SYS_pipe2,
    libc::SYS_select,
    libc::SYS_poll,
    libc::SYS_epoll_create1,
    libc::SYS_epoll_ctl,
    libc::SYS_epoll_wait,
    libc::SYS_clock_gettime,
    libc::SYS_clock_getres,
    libc::SYS_nanosleep,
    libc::SYS_futex,
    libc::SYS_exit_group,
    libc::SYS_exit,
    libc::SYS_rt_sigaction,
    libc::SYS_rt_sigprocmask,
    libc::SYS_rt_sigreturn,
    libc::SYS_sigaltstack,
    libc::SYS_arch_prctl,
    libc::SYS_set_tid_address,
    libc::SYS_set_robust_list,
    libc::SYS_prlimit64,
    libc::SYS_getrandom,
    libc::SYS_eventfd2,
    libc::SYS_timerfd_create,
    libc::SYS_timerfd_settime,
    libc::SYS_sched_yield,
    libc::SYS_sched_getaffinity,
    libc::SYS_clone3,
    libc::SYS_wait4,
    libc::SYS_fcntl,
    libc::SYS_getpid,
    libc::SYS_gettid,
    libc::SYS_tgkill,
    libc::SYS_madvise,
    libc::SYS_mremap,
    libc::SYS_ftruncate,
    libc::SYS_dup,
    libc::SYS_dup2,
    libc::SYS_dup3,
    libc::SYS_prctl,
    libc::SYS_rseq,
    libc::SYS_capget,
    libc::SYS_capset,
    libc::SYS_statx,
    libc::SYS_getuid,
    libc::SYS_getgid,
    libc::SYS_geteuid,
    libc::SYS_getegid,
    libc::SYS_setsid,
    libc::SYS_umask,
    libc::SYS_getcwd,
    libc::SYS_sched_setaffinity,
    libc::SYS_clock_nanosleep,
    libc::SYS_ppoll,
    libc::SYS_epoll_pwait,
    libc::SYS_fstatfs,
    libc::SYS_unlinkat,
    libc::SYS_mkdirat,
    libc::SYS_renameat2,
    libc::SYS_memfd_create,
    libc::SYS_copy_file_range,
    libc::SYS_preadv,
    libc::SYS_pwritev,
    libc::SYS_preadv2,
    libc::SYS_pwritev2,
    libc::SYS_fgetxattr,
    libc::SYS_fsetxattr,
    libc::SYS_flistxattr,
    libc::SYS_fallocate,
    libc::SYS_fchmod,
    libc::SYS_fchmodat,
    libc::SYS_fchown,
    libc::SYS_fchownat,
    libc::SYS_linkat,
    libc::SYS_symlinkat,
    libc::SYS_readlinkat,
    libc::SYS_utimensat,
];

/// Networking syscalls — included only when network.allow is non-empty.
const NETWORK_SYSCALLS: &[i64] = &[
    libc::SYS_socket,
    libc::SYS_connect,
    libc::SYS_bind,
    libc::SYS_listen,
    libc::SYS_accept4,
    libc::SYS_sendto,
    libc::SYS_recvfrom,
    libc::SYS_sendmsg,
    libc::SYS_recvmsg,
    libc::SYS_getsockopt,
    libc::SYS_setsockopt,
    libc::SYS_getpeername,
    libc::SYS_getsockname,
    libc::SYS_shutdown,
];

/// Shell/exec syscalls — included only when capabilities.shell is true.
const SHELL_SYSCALLS: &[i64] = &[
    libc::SYS_execve,
    libc::SYS_execveat,
    libc::SYS_fork,
    libc::SYS_vfork,
    libc::SYS_clone,
];

// ── Profile construction ────────────────────────────────────────────────────

/// Build a seccomp whitelist profile from the agent manifest.
pub fn build_profile(manifest: &AgentManifest) -> SeccompProfile {
    let mut allowed: BTreeSet<i64> = VMM_BASE.iter().copied().collect();

    if manifest.capabilities.shell {
        allowed.extend(SHELL_SYSCALLS.iter().copied());
    }

    if !manifest.capabilities.network.allow.is_empty() {
        allowed.extend(NETWORK_SYSCALLS.iter().copied());
    }

    SeccompProfile { allowed }
}

// ── BPF assembly ────────────────────────────────────────────────────────────

/// Assemble a BPF filter program from the profile.
///
/// Structure:
///   [0]  LD  seccomp_data.nr
///   [1]  JEQ syscall_0 → ALLOW
///   [2]  JEQ syscall_1 → ALLOW
///   ...
///   [N]  RET default_action
///   [N+1] RET ALLOW
fn assemble_bpf(profile: &SeccompProfile, kill: bool) -> Result<Vec<SockFilter>, SeccompError> {
    let syscalls: Vec<u32> = profile.allowed.iter().map(|&n| n as u32).collect();
    let count = syscalls.len();

    // BPF jump offsets are u8, so max 255 rules before the default RET.
    if count > 255 {
        return Err(SeccompError::TooManySyscalls(count));
    }

    let mut prog = Vec::with_capacity(count + 3);

    // Load syscall number
    prog.push(bpf_stmt(BPF_LD | BPF_W | BPF_ABS, SECCOMP_DATA_NR_OFFSET));

    // For each allowed syscall: if match, jump to ALLOW (which sits after
    // the default RET). jt = distance to ALLOW, jf = 0 (fall through).
    for (i, &nr) in syscalls.iter().enumerate() {
        let remaining = count - i - 1; // instructions after this JEQ (not counting RET pair)
        // jt: skip remaining JEQs + 1 default RET = remaining + 1
        let jt = (remaining + 1) as u8;
        prog.push(bpf_jump(BPF_JMP | BPF_JEQ | BPF_K, nr, jt, 0));
    }

    // Default: deny
    let default_action = if kill {
        SECCOMP_RET_KILL_PROCESS
    } else {
        SECCOMP_RET_LOG
    };
    prog.push(bpf_stmt(BPF_RET | BPF_K, default_action));

    // ALLOW
    prog.push(bpf_stmt(BPF_RET | BPF_K, SECCOMP_RET_ALLOW));

    Ok(prog)
}

/// Apply the seccomp profile to the current process.
///
/// When `kill` is true, unallowed syscalls trigger `SECCOMP_RET_KILL_PROCESS`.
/// When false, they are logged via `SECCOMP_RET_LOG` (audit mode for v0.1).
pub fn apply(profile: &SeccompProfile, kill: bool) -> Result<(), SeccompError> {
    let prog = assemble_bpf(profile, kill)?;

    let fprog = SockFprog {
        len: prog.len() as u16,
        filter: prog.as_ptr(),
    };

    unsafe {
        // Required before installing a seccomp filter as non-root.
        let ret = libc::prctl(PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0);
        if ret != 0 {
            return Err(SeccompError::NoNewPrivs(ret));
        }

        let ret = libc::prctl(
            PR_SET_SECCOMP,
            SECCOMP_MODE_FILTER as libc::c_ulong,
            &fprog as *const SockFprog as libc::c_ulong,
        );
        if ret != 0 {
            return Err(SeccompError::SetSeccomp(ret));
        }
    }

    Ok(())
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest;

    fn parse_manifest(toml: &str) -> AgentManifest {
        manifest::parse(toml).unwrap()
    }

    #[test]
    fn test_base_profile_has_core_syscalls() {
        let m = parse_manifest(
            r#"
            [agent]
            name = "basic"
            "#,
        );
        let profile = build_profile(&m);

        // Core syscalls must be present
        assert!(profile.allowed.contains(&libc::SYS_read));
        assert!(profile.allowed.contains(&libc::SYS_write));
        assert!(profile.allowed.contains(&libc::SYS_close));
        assert!(profile.allowed.contains(&libc::SYS_mmap));
        assert!(profile.allowed.contains(&libc::SYS_exit_group));
        assert!(profile.allowed.contains(&libc::SYS_futex));
        assert!(profile.allowed.contains(&libc::SYS_getrandom));
    }

    #[test]
    fn test_no_shell_excludes_execve() {
        let m = parse_manifest(
            r#"
            [agent]
            name = "no-shell"

            [capabilities]
            shell = false
            "#,
        );
        let profile = build_profile(&m);

        assert!(!profile.allowed.contains(&libc::SYS_execve));
        assert!(!profile.allowed.contains(&libc::SYS_execveat));
        assert!(!profile.allowed.contains(&libc::SYS_fork));
        assert!(!profile.allowed.contains(&libc::SYS_vfork));
        assert!(!profile.allowed.contains(&libc::SYS_clone));
    }

    #[test]
    fn test_no_network_excludes_socket() {
        let m = parse_manifest(
            r#"
            [agent]
            name = "isolated"
            "#,
        );
        let profile = build_profile(&m);

        assert!(!profile.allowed.contains(&libc::SYS_socket));
        assert!(!profile.allowed.contains(&libc::SYS_connect));
        assert!(!profile.allowed.contains(&libc::SYS_bind));
        assert!(!profile.allowed.contains(&libc::SYS_listen));
        assert!(!profile.allowed.contains(&libc::SYS_accept4));
        assert!(!profile.allowed.contains(&libc::SYS_sendto));
        assert!(!profile.allowed.contains(&libc::SYS_recvfrom));
    }

    #[test]
    fn test_full_capabilities() {
        let m = parse_manifest(
            r#"
            [agent]
            name = "full"

            [capabilities]
            shell = true

            [capabilities.network]
            allow = ["api.openai.com"]

            [capabilities.filesystem]
            read = ["/data"]
            write = ["/output"]
            "#,
        );
        let profile = build_profile(&m);

        // Shell syscalls present
        assert!(profile.allowed.contains(&libc::SYS_execve));
        assert!(profile.allowed.contains(&libc::SYS_fork));

        // Network syscalls present
        assert!(profile.allowed.contains(&libc::SYS_socket));
        assert!(profile.allowed.contains(&libc::SYS_connect));

        // Base syscalls still present
        assert!(profile.allowed.contains(&libc::SYS_read));
        assert!(profile.allowed.contains(&libc::SYS_write));
    }

    #[test]
    fn test_bpf_assembly_structure() {
        let m = parse_manifest(
            r#"
            [agent]
            name = "test"
            "#,
        );
        let profile = build_profile(&m);
        let prog = assemble_bpf(&profile, true).unwrap();

        // First instruction: load syscall number
        assert_eq!(prog[0].code, BPF_LD | BPF_W | BPF_ABS);
        assert_eq!(prog[0].k, SECCOMP_DATA_NR_OFFSET);

        // Last instruction: RET ALLOW
        let last = prog.last().unwrap();
        assert_eq!(last.code, BPF_RET | BPF_K);
        assert_eq!(last.k, SECCOMP_RET_ALLOW);

        // Second-to-last: RET KILL_PROCESS (default deny)
        let default_ret = &prog[prog.len() - 2];
        assert_eq!(default_ret.code, BPF_RET | BPF_K);
        assert_eq!(default_ret.k, SECCOMP_RET_KILL_PROCESS);

        // Total length: 1 (load) + N (JEQ per syscall) + 2 (RET pair)
        assert_eq!(prog.len(), 1 + profile.allowed.len() + 2);
    }

    #[test]
    fn test_bpf_audit_mode() {
        let m = parse_manifest(
            r#"
            [agent]
            name = "audit"
            "#,
        );
        let profile = build_profile(&m);
        let prog = assemble_bpf(&profile, false).unwrap();

        // Default action should be LOG, not KILL
        let default_ret = &prog[prog.len() - 2];
        assert_eq!(default_ret.k, SECCOMP_RET_LOG);
    }
}
