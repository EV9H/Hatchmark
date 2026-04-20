use anyhow::{bail, Result};
use windows::core::HSTRING;
use windows::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
use windows::Win32::System::Threading::CreateMutexW;

pub struct InstanceGuard {
    handle: HANDLE,
}

impl Drop for InstanceGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}

pub fn acquire() -> Result<InstanceGuard> {
    let name = HSTRING::from("Global\\HatchmarkDaemon");
    unsafe {
        let handle = CreateMutexW(None, true, &name)?;
        let err = GetLastError();
        if err == ERROR_ALREADY_EXISTS {
            let _ = CloseHandle(handle);
            bail!("daemon already running");
        }
        Ok(InstanceGuard { handle })
    }
}
