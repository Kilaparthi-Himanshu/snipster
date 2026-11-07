#![windows_subsystem = "windows"]

mod clipboard;
mod hotkey;
mod snipping;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Windows launches another Snipster process for protocol activation.
    // Detect that activation before registering the global hotkey.
    if let Some(uri) = args.iter().find(|arg| {
        arg.starts_with("snipster://")
    }) {
        snipping::handle_callback_uri(uri)?;
        return Ok(())
    }

    hotkey::run()?;

    Ok(())
}
