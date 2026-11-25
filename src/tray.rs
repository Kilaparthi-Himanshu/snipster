
use anyhow::{Context, Result};
use tray_icon::{
    menu::{Menu, MenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};

pub struct SnipsterTray {
    // Keep these alive for the lifetime of the tray.
    pub _icon: TrayIcon,
    pub open_item: MenuItem,
    pub capture_item: MenuItem,
    pub exit_item: MenuItem,
}

impl SnipsterTray {
    pub fn new() -> Result<Self> {
        let menu = Menu::new();

        let open_item = MenuItem::new("Open Snipster", true, None);
        let capture_item = MenuItem::new("Capture Screenshot", true, None);
        let exit_item = MenuItem::new("Exit", true, None);

        menu.append_items(&[
            &open_item,
            &capture_item,
            &exit_item,
        ])
        .context("Could not create tray menu")?;

        let icon = create_icon()?;

        let tray_icon = TrayIconBuilder::new()
            .with_icon(icon)
            .with_tooltip("Snipster")
            .with_menu(Box::new(menu))
            .build()
            .context("Could not create Snipster tray icon")?;

        Ok(Self {
            _icon: tray_icon,
            open_item,
            capture_item,
            exit_item,
        })
    }
}

fn create_icon() -> Result<Icon> {
    let image = image::load_from_memory(
        include_bytes!("../assets/icons/app-icon-48.png")
    )?
    .into_rgba8();

    let (width, height) = image.dimensions();

    Icon::from_rgba(
        image.into_raw(),
        width,
        height,
    )
    .context("Could not create Snipster tray icon")
}
