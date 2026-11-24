use anyhow::{Context, Result};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use eframe::egui;
use windows::{
    core::w,
    Win32::{
        Foundation::{
            CloseHandle,
            GetLastError,
            ERROR_ALREADY_EXISTS,
            HANDLE,
            WAIT_OBJECT_0,
        },
        System::Threading::{
            CreateEventW,
            CreateMutexW,
            SetEvent,
            WaitForSingleObject,
        },
    },
};

const WAIT_TIMEOUT_MS: u32 = 250;

pub struct SingleInstance {
    mutex: HANDLE,
    activation_event: HANDLE,
    stop: Arc<AtomicBool>,
    listener: Option<JoinHandle<()>>,
}

impl SingleInstance {
    pub fn acquire(activate_existing: bool) -> Result<Option<Self>> {
        unsafe {
            // Create/open the activation event first. It is auto-reset.
            let activation_event = CreateEventW(
                None,
                false,
                false,
                w!("Local\\Snipster_Activation_Event"),
            )
            .context("Could not create activation event")?;

            // The existence of this named mutex identifies the primary process.
            let mutex = CreateMutexW(
                None,
                false,
                w!("Local\\Snipster_Instance_Mutex"),
            )
            .context("Could not create instance mutex")?;

            if GetLastError() == ERROR_ALREADY_EXISTS {
                if activate_existing {
                    SetEvent(activation_event)
                        .context("Failed to signal existing Snipster instance")?;
                }

                let _ = CloseHandle(mutex);
                let _ = CloseHandle(activation_event);

                return Ok(None);
            }

            Ok(Some(Self {
                mutex,
                activation_event,
                stop: Arc::new(AtomicBool::new(false)),
                listener: None,
            }))
        }
    }

    pub fn start_listener(&mut self, ctx: egui::Context) {
        if self.listener.is_some() {
            return;
        }

        // Convert the raw handle into a Send-compatible integer.
        let event_handle = self.activation_event.0 as usize;
        let stop = Arc::clone(&self.stop);

        self.listener = Some(thread::spawn(move || {
            // Reconstruct the Windows handle inside this thread.
            let event = HANDLE(event_handle as *mut std::ffi::c_void);

            while !stop.load(Ordering::Acquire) {
                let result = unsafe {
                    WaitForSingleObject(event, WAIT_TIMEOUT_MS)
                };

                if result == WAIT_OBJECT_0 {
                    crate::app::show_main_window();
                    ctx.request_repaint();
                }
            }
        }));
    }
}

impl Drop for SingleInstance {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);

        if let Some(listener) = self.listener.take() {
            let _ = listener.join();
        }

        unsafe {
            let _ = CloseHandle(self.activation_event);
            let _ = CloseHandle(self.mutex);
        }
    }
}
