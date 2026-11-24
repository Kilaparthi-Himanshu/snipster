
use anyhow::{anyhow, Context, Result};
use std::{
    sync::mpsc::{self, Receiver, Sender, SyncSender},
    thread::{self, JoinHandle},
    time::Duration,
};

use windows::Win32::{
    Foundation::{LPARAM, WPARAM},
    System::Threading::GetCurrentThreadId,
    UI::{
        Input::KeyboardAndMouse::{
            RegisterHotKey,
            UnregisterHotKey,
            HOT_KEY_MODIFIERS,
            MOD_ALT,
            MOD_CONTROL,
            MOD_NOREPEAT,
            MOD_SHIFT,
            MOD_WIN,
        },
        WindowsAndMessaging::{
            GetMessageW,
            PeekMessageW,
            PostThreadMessageW,
            MSG,
            PM_NOREMOVE,
            WM_APP,
            WM_HOTKEY,
        },
    },
};

use crate::{shortcut::Shortcut, snipping};

const FIRST_HOTKEY_ID: i32 = 1;
const SECOND_HOTKEY_ID: i32 = 2;
const WM_HOTKEY_COMMAND: u32 = WM_APP + 1;

enum Command {
    SetShortcut(
        Shortcut,
        SyncSender<std::result::Result<(), String>>,
    ),
    Capture,
    Shutdown,
}

pub struct HotkeyManager {
    sender: Sender<Command>,
    thread_id: u32,
    thread: Option<JoinHandle<()>>,
}

impl HotkeyManager {
    pub fn start(initial: Shortcut) -> Result<Self> {
        let (sender, receiver) = mpsc::channel();
        let (ready_sender, ready_receiver) = mpsc::sync_channel(1);

        let thread = thread::spawn(move || {
            run_hotkey_thread(initial, receiver, ready_sender);
        });

        let thread_id = match ready_receiver.recv() {
            Ok(Ok(thread_id)) => thread_id,
            Ok(Err(error)) => {
                let _ = thread.join();
                return Err(anyhow!(error));
            }
            Err(error) => {
                let _ = thread.join();
                return Err(anyhow!(
                    "Hotkey thread failed to initialize: {error}"
                ));
            }
        };

        Ok(Self {
            sender,
            thread_id,
            thread: Some(thread),
        })
    }

    pub fn set_shortcut(&self, shortcut: Shortcut) -> Result<()> {
        let (reply_sender, reply_receiver) = mpsc::sync_channel(1);

        self.sender
            .send(Command::SetShortcut(shortcut, reply_sender))
            .context("Hotkey manager is not running")?;

        unsafe {
            PostThreadMessageW(
                self.thread_id,
                WM_HOTKEY_COMMAND,
                WPARAM(0),
                LPARAM(0),
            )
            .context("Could not notify hotkey manager")?;
        }

        match reply_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(Ok(())) => Ok(()),
            Ok(Err(error)) => Err(anyhow!(error)),
            Err(error) => Err(anyhow!(
                "Hotkey manager did not respond: {error}"
            )),
        }
    }

    pub fn request_capture(&self) -> Result<()> {
        self.sender
            .send(Command::Capture)
            .context("Hotkey manager is not running")?;

        unsafe {
            PostThreadMessageW(
                self.thread_id,
                WM_HOTKEY_COMMAND,
                WPARAM(0),
                LPARAM(0),
            )
            .context("Could not notify hotkey manager")?;
        }

        Ok(())
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        let _ = self.sender.send(Command::Shutdown);

        let wake_result = unsafe {
            PostThreadMessageW(
                self.thread_id,
                WM_HOTKEY_COMMAND,
                WPARAM(0),
                LPARAM(0),
            )
        };

        if wake_result.is_ok() {
            if let Some(thread) = self.thread.take() {
                let _ = thread.join();
            }
        }
    }
}

fn run_hotkey_thread(
    initial: Shortcut,
    receiver: Receiver<Command>,
    ready: SyncSender<std::result::Result<u32, String>>,
) {
    unsafe {
        // Create this thread's message queue before accepting commands.
        let mut message = MSG::default();
        let _ = PeekMessageW(
            &mut message,
            None,
            0,
            0,
            PM_NOREMOVE,
        );

        let thread_id = GetCurrentThreadId();

        if let Err(error) = register_shortcut(
            FIRST_HOTKEY_ID,
            &initial,
        ) {
            let _ = ready.send(Err(format!(
                "Could not register initial shortcut: {error}"
            )));
            return;
        }

        let _ = ready.send(Ok(thread_id));

        let mut active_id = FIRST_HOTKEY_ID;
        let mut active_shortcut = initial;

        loop {
            let result = GetMessageW(
                &mut message,
                None,
                0,
                0,
            );

            if result.0 == -1 {
                eprintln!("Hotkey message loop failed.");
                break;
            }

            if result.0 == 0 {
                break;
            }

            if message.message == WM_HOTKEY {
                if message.wParam.0 as i32 == active_id {
                    if let Err(error) = snipping::launch_capture() {
                        eprintln!("Capture launch failed: {error:#}");
                    }
                }
            } else if message.message == WM_HOTKEY_COMMAND {
                while let Ok(command) = receiver.try_recv() {
                    match command {
                        Command::SetShortcut(shortcut, reply) => {
                            let result = replace_shortcut(
                                &shortcut,
                                &mut active_shortcut,
                                &mut active_id,
                            );

                            let _ = reply.send(result);
                        }

                        Command::Shutdown => {
                            let _ = UnregisterHotKey(
                                None,
                                active_id,
                            );

                            return;
                        }

                        Command::Capture => {
                            if let Err(error) = snipping::launch_capture() {
                                eprintln!("Tray capture launch failed: {error:#}");
                            }
                        }
                    }
                }
            }
        }

        let _ = UnregisterHotKey(None, active_id);
    }
}

fn replace_shortcut(
    new_shortcut: &Shortcut,
    active_shortcut: &mut Shortcut,
    active_id: &mut i32,
) -> std::result::Result<(), String> {
    if new_shortcut == active_shortcut {
        return Ok(());
    }

    let new_id = if *active_id == FIRST_HOTKEY_ID {
        SECOND_HOTKEY_ID
    } else {
        FIRST_HOTKEY_ID
    };

    // Register the new shortcut BEFORE removing the old one.
    register_shortcut(new_id, new_shortcut)
        .map_err(|error| format!(
            "This shortcut is unavailable or already in use: {error}"
        ))?;

    // Only remove the previous shortcut after the new one succeeds.
    if let Err(error) = unsafe {
        UnregisterHotKey(None, *active_id)
    } {
        let _ = unsafe {
            UnregisterHotKey(None, new_id)
        };

        return Err(format!(
            "Could not replace the previous shortcut: {error}"
        ));
    }

    *active_id = new_id;
    *active_shortcut = new_shortcut.clone();

    Ok(())
}

fn register_shortcut(
    id: i32,
    shortcut: &Shortcut,
) -> windows::core::Result<()> {
    let mut modifiers: HOT_KEY_MODIFIERS = MOD_NOREPEAT;

    if shortcut.win {
        modifiers |= MOD_WIN;
    }

    if shortcut.ctrl {
        modifiers |= MOD_CONTROL;
    }

    if shortcut.alt {
        modifiers |= MOD_ALT;
    }

    if shortcut.shift {
        modifiers |= MOD_SHIFT;
    }

    unsafe {
        RegisterHotKey(
            None,
            id,
            modifiers,
            shortcut.key,
        )
    }
}
