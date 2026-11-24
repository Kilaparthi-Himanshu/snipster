use anyhow::{anyhow, Context, Result};
use std::{matches, thread};

use windows::{
    core::HSTRING,
    ApplicationModel::{StartupTask, StartupTaskState},
    Win32::System::WinRT::{
        RoInitialize,
        RoUninitialize,
        RO_INIT_MULTITHREADED,
    },
};

const TASK_ID: &str = "SnipsterStartup";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupState {
    Enabled,
    Disabled,
    DisabledByUser,
    DisabledByPolicy,
    EnabledByPolicy,
}

impl StartupState {
    pub fn is_enabled(self) -> bool {
        matches!(self, Self::Enabled | Self::EnabledByPolicy)
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Enabled => "Enabled",
            Self::Disabled => "Disabled",
            Self::DisabledByUser => "Disabled by user in Windows",
            Self::DisabledByPolicy => "Disabled by Windows policy",
            Self::EnabledByPolicy => "Enabled by Windows policy",
        }
    }
}

fn map_state(state: StartupTaskState) -> Result<StartupState> {
    if state == StartupTaskState::Enabled {
        Ok(StartupState::Enabled)
    } else if state == StartupTaskState::Disabled {
        Ok(StartupState::Disabled)
    } else if state == StartupTaskState::DisabledByUser {
        Ok(StartupState::DisabledByUser)
    } else if state == StartupTaskState::DisabledByPolicy {
        Ok(StartupState::DisabledByPolicy)
    } else if state == StartupTaskState::EnabledByPolicy {
        Ok(StartupState::EnabledByPolicy)
    } else {
        Err(anyhow!("Windows returned an unknown startup state"))
    }
}

// Run WinRT startup APIs on a dedicated MTA thread.
// This avoids conflicting with COM initialization performed by eframe.
fn run_winrt<T, F>(operation: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T> + Send + 'static,
{
    thread::spawn(move || {
        unsafe {
            RoInitialize(RO_INIT_MULTITHREADED)
                .context("Could not initialize WinRT")?;

            let result = operation();

            RoUninitialize();

            result
        }
    })
    .join()
    .map_err(|_| anyhow!("Startup API thread panicked"))?
}

pub fn get_startup_state() -> Result<StartupState> {
    run_winrt(|| {
        let task = StartupTask::GetAsync(&HSTRING::from(TASK_ID))?
            .join()
            .context("Could not retrieve Snipster startup task")?;

        map_state(task.State()?)
    })
}

pub fn enable_startup() -> Result<StartupState> {
    run_winrt(|| {
        let task = StartupTask::GetAsync(&HSTRING::from(TASK_ID))?
            .join()
            .context("Could not retrieve Snipster startup task")?;

        let state = task
            .RequestEnableAsync()?
            .join()
            .context("Could not enable Snipster startup")?;

        map_state(state)
    })
}

pub fn disable_startup() -> Result<StartupState> {
    run_winrt(|| {
        let task = StartupTask::GetAsync(&HSTRING::from(TASK_ID))?
            .join()
            .context("Could not retrieve Snipster startup task")?;

        task.Disable()
            .context("Could not disable Snipster startup")?;

        map_state(task.State()?)
    })
}
