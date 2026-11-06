use anyhow::{anyhow, Result};
use crate::snipping;

use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey,
    UnregisterHotKey,
    MOD_CONTROL,
    MOD_NOREPEAT,
    MOD_SHIFT,
    MOD_WIN,
};

use windows::Win32::UI::WindowsAndMessaging::{
    GetMessageW,
    MSG,
    WM_HOTKEY,
};

const HOTKEY_ID: i32 = 1;

pub fn run() -> Result<()> {
    unsafe {
        // Win + Shift + Ctrl + S
        let modifiers = MOD_WIN | MOD_SHIFT | MOD_CONTROL | MOD_NOREPEAT;

        RegisterHotKey(
            None,
            HOTKEY_ID,
            modifiers,
            b'S' as u32,
        )
        .map_err(|e| anyhow!(
            "Failed to register Snipster hotkey: {e}. \
            The shortcut may already be in use."
        ))?;

        println!("Snipster is running!");
        println!("Press Win + Shift + Ctrl + S to test.");

        let mut message = MSG::default();

        loop {
            let result = GetMessageW(
                &mut message,
                None,
                0,
                0
            );

            // -1 means GetMessageW encountered an error.
            if result.0 == -1 {
                let _ = UnregisterHotKey(None, HOTKEY_ID);
                return Err(anyhow!("Windows message loop failed"));
            }

            // 0 means WM_QUIT.
            if result.0 == 0 {
                break;
            }

            if message.message == WM_HOTKEY {
                println!("Snipster hotkey detected!");

                if let Err(error) = snipping::launch_capture() {
                    eprintln!("Capture launch failed: {error:#}");
                }
            }

        }

        UnregisterHotKey(None, HOTKEY_ID)?;
    }

    Ok(())
}
