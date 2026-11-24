
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
    // Temporary 32x32 purple icon.
    // We'll replace this with the real Snipster branding later.
    let size = 32u32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];

    for y in 0..size {
        for x in 0..size {
            let offset = ((y * size + x) * 4) as usize;

            let dx = x as i32 - 16;
            let dy = y as i32 - 16;

            let inside_circle = dx * dx + dy * dy <= 14 * 14;

            if inside_circle {
                rgba[offset] = 130;
                rgba[offset + 1] = 90;
                rgba[offset + 2] = 240;
                rgba[offset + 3] = 255;
            }
        }
    }

    Icon::from_rgba(rgba, size, size)
        .context("Could not create tray icon image")
}
