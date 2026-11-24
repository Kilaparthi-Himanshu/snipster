#![windows_subsystem = "windows"]

use std::{
    path::PathBuf,
    process::{Command, ExitCode},
};

fn main() -> ExitCode {
    let executable = match std::env::current_exe() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("Could not locate startup launcher: {error}");
            return ExitCode::FAILURE;
        }
    };

    let Some(directory) = executable.parent() else {
        eprintln!("Could not locate Snipster installation directory");
        return ExitCode::FAILURE;
    };

    let app_path: PathBuf = directory.join("snipster.exe");

    match Command::new(app_path)
        .arg("--startup")
        .spawn()
    {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Could not start Snipster: {error}");
            ExitCode::FAILURE
        }
    }
}
