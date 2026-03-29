//! Minimal FFI bindings to libkrun.
//!
//! Only the functions needed for NullBox agent VM lifecycle.
//! Full API: https://github.com/containers/libkrun/blob/main/include/libkrun.h

use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

#[link(name = "krun")]
unsafe extern "C" {
    fn krun_set_log_level(level: u32) -> i32;
    fn krun_create_ctx() -> i32;
    fn krun_free_ctx(ctx_id: u32) -> i32;
    fn krun_set_vm_config(ctx_id: u32, num_vcpus: u8, ram_mib: u32) -> i32;
    fn krun_set_root(ctx_id: u32, root_path: *const c_char) -> i32;
    fn krun_set_workdir(ctx_id: u32, workdir_path: *const c_char) -> i32;
    fn krun_set_exec(
        ctx_id: u32,
        exec_path: *const c_char,
        argv: *const *const c_char,
        envp: *const *const c_char,
    ) -> i32;
    fn krun_set_port_map(ctx_id: u32, port_map: *const *const c_char) -> i32;
    fn krun_start_enter(ctx_id: u32) -> i32;
}

/// Configuration for a single agent microVM.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VmConfig {
    pub name: String,
    pub vcpus: u8,
    pub ram_mib: u32,
    pub root_path: String,
    pub exec_path: String,
    pub args: Vec<String>,
    pub env: Vec<String>,
    pub port_map: Vec<String>,
    pub workdir: String,
}

/// Errors from libkrun operations.
#[derive(Debug, thiserror::Error)]
pub enum KrunError {
    #[error("krun_create_ctx failed: {0}")]
    CreateCtx(i32),
    #[error("krun_set_vm_config failed: {0}")]
    SetVmConfig(i32),
    #[error("krun_set_root failed: {0}")]
    SetRoot(i32),
    #[error("krun_set_exec failed: {0}")]
    SetExec(i32),
    #[error("krun_start_enter failed: {0}")]
    StartEnter(i32),
    #[error("nul byte in string: {0}")]
    NulError(#[from] std::ffi::NulError),
    #[error("libkrun call failed: {0}")]
    Other(String),
}

/// Set libkrun log level. Call once before creating any context.
/// 0=Off, 1=Error, 2=Warn, 3=Info, 4=Debug, 5=Trace
pub fn set_log_level(level: u32) {
    unsafe {
        krun_set_log_level(level);
    }
}

/// Create a VM context, configure it, and start it.
///
/// **WARNING**: `krun_start_enter` never returns on success.
/// This function should only be called from a forked child process.
pub fn run_vm(config: &VmConfig) -> Result<(), KrunError> {
    unsafe {
        // Create context
        let ctx = krun_create_ctx();
        if ctx < 0 {
            return Err(KrunError::CreateCtx(ctx));
        }
        let ctx = ctx as u32;

        // Set CPU and memory
        let ret = krun_set_vm_config(ctx, config.vcpus, config.ram_mib);
        if ret < 0 {
            krun_free_ctx(ctx);
            return Err(KrunError::SetVmConfig(ret));
        }

        // Set root filesystem
        let root = CString::new(config.root_path.as_str())?;
        let ret = krun_set_root(ctx, root.as_ptr());
        if ret < 0 {
            krun_free_ctx(ctx);
            return Err(KrunError::SetRoot(ret));
        }

        // Set working directory
        let workdir = CString::new(config.workdir.as_str())?;
        let ret = krun_set_workdir(ctx, workdir.as_ptr());
        if ret < 0 {
            krun_free_ctx(ctx);
            return Err(KrunError::Other("krun_set_workdir failed".into()));
        }

        // Set port map (TSI networking)
        if !config.port_map.is_empty() {
            let c_ports: Vec<CString> = config
                .port_map
                .iter()
                .map(|p| CString::new(p.as_str()))
                .collect::<Result<_, _>>()?;
            let mut port_ptrs: Vec<*const c_char> =
                c_ports.iter().map(|p| p.as_ptr()).collect();
            port_ptrs.push(ptr::null());
            let ret = krun_set_port_map(ctx, port_ptrs.as_ptr());
            if ret < 0 {
                krun_free_ctx(ctx);
                return Err(KrunError::Other("krun_set_port_map failed".into()));
            }
        }

        // Set executable, args, and environment
        let exec = CString::new(config.exec_path.as_str())?;

        let c_args: Vec<CString> = config
            .args
            .iter()
            .map(|a| CString::new(a.as_str()))
            .collect::<Result<_, _>>()?;
        let mut argv_ptrs: Vec<*const c_char> =
            c_args.iter().map(|a| a.as_ptr()).collect();
        argv_ptrs.push(ptr::null());

        let c_env: Vec<CString> = config
            .env
            .iter()
            .map(|e| CString::new(e.as_str()))
            .collect::<Result<_, _>>()?;
        let mut envp_ptrs: Vec<*const c_char> =
            c_env.iter().map(|e| e.as_ptr()).collect();
        envp_ptrs.push(ptr::null());

        let ret = krun_set_exec(
            ctx,
            exec.as_ptr(),
            argv_ptrs.as_ptr(),
            envp_ptrs.as_ptr(),
        );
        if ret < 0 {
            krun_free_ctx(ctx);
            return Err(KrunError::SetExec(ret));
        }

        // Start VM — this call never returns on success
        let ret = krun_start_enter(ctx);
        Err(KrunError::StartEnter(ret))
    }
}
