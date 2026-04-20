use anyhow::{Context, Result};
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Foundation::ERROR_FILE_NOT_FOUND;
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
    KEY_WRITE, REG_SZ,
};

const RUN_SUBKEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "Hatchmark";

pub fn set_enabled(enabled: bool) -> Result<()> {
    let exe = std::env::current_exe().context("current_exe")?;
    let exe_str = format!("\"{}\"", exe.display());
    unsafe {
        let subkey = HSTRING::from(RUN_SUBKEY);
        let value_name = HSTRING::from(VALUE_NAME);
        let mut key = HKEY::default();
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            0,
            KEY_WRITE,
            &mut key,
        )
        .ok()
        .context("open Run key")?;

        let result = if enabled {
            let wide: Vec<u16> = exe_str.encode_utf16().chain(std::iter::once(0)).collect();
            let bytes = std::slice::from_raw_parts(wide.as_ptr() as *const u8, wide.len() * 2);
            RegSetValueExW(key, PCWSTR(value_name.as_ptr()), 0, REG_SZ, Some(bytes))
                .ok()
                .context("write Run value")
        } else {
            let r = RegDeleteValueW(key, PCWSTR(value_name.as_ptr()));
            if r == ERROR_FILE_NOT_FOUND {
                Ok(())
            } else {
                r.ok().context("delete Run value")
            }
        };
        let _ = RegCloseKey(key);
        result
    }
}
