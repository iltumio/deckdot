#[cfg(target_os = "windows")]
pub fn focus_window_by_title(title: &str) -> Result<(), String> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{FindWindowA, SetForegroundWindow, ShowWindow, SW_RESTORE};
    use std::ffi::CString;

    let title_cstr = CString::new(title)
        .map_err(|e| format!("Invalid window title: {}", e))?;

    unsafe {
        let hwnd = FindWindowA(None, Some(&title_cstr));
        
        if hwnd.0 == 0 {
            return Err(format!("Window with title '{}' not found", title));
        }

        ShowWindow(hwnd, SW_RESTORE);
        SetForegroundWindow(hwnd)
            .ok()
            .map_err(|e| format!("Failed to set foreground window: {:?}", e))?;

        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
pub fn focus_window_by_title(_title: &str) -> Result<(), String> {
    // Focus logic for other platforms can be implemented here
    // For now, just return Ok() as a no-op
    Ok(())
}

