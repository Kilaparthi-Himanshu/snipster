#![windows_subsystem = "windows"] // Enable this only while producing a production build.

mod app;
mod clipboard;
mod hotkey;
mod snipping;
mod shortcut;
mod settings;
mod tray;
mod single_instance;
mod startup;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Handle Windows protocol activation separately.
    if let Some(uri) = args.iter().find(|arg| arg.starts_with("snipster://")) {
        snipping::handle_callback_uri(uri)?;
        return Ok(());
    }

    let launched_at_startup = args.iter().any(|arg| arg == "--startup");

    let Some(single_instance) =
        single_instance::SingleInstance::acquire(!launched_at_startup)?
    else {
        return Ok(());
    };

    let (settings, settings_path) = settings::Settings::load()?;
    let hotkey_manager = hotkey::HotkeyManager::start(settings.shortcut.clone())?;

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([760.0, 520.0])
            .with_min_inner_size([650.0, 450.0])
            .with_title("Snipster")
            .with_icon(load_app_icon())
            .with_visible(!launched_at_startup),
        ..Default::default()
    };

    eframe::run_native(
        "Snipster",
        options,
        Box::new(move |cc| {
            Ok(Box::new(app::SnipsterApp::new(
                hotkey_manager,
                settings,
                settings_path,
                single_instance,
                launched_at_startup,
                cc.egui_ctx.clone(),
            )))
        }),
    )
    .map_err(|error| anyhow::anyhow!("{error}"))?;

    Ok(())
}

fn load_app_icon() -> eframe::egui::IconData {
    let image = image::load_from_memory(
        include_bytes!("../assets/icons/app-icon-256.png")
    )
    .expect("Failed to load Snipster icon")
    .into_rgba8();

    let (width, height) = image.dimensions();

    eframe::egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    }
}
