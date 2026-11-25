use eframe::egui;

use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState,
    VK_LWIN,
    VK_RWIN,
};

use windows::{
    core::w,
    Win32::UI::WindowsAndMessaging::{
        FindWindowW,
        SetForegroundWindow,
        ShowWindow,
        SW_HIDE,
        SW_RESTORE,
        SW_SHOW,
    },
};

use crate::hotkey::HotkeyManager;

use crate::shortcut::Shortcut;

use std::path::PathBuf;
use crate::settings::Settings;

use crate::tray::SnipsterTray;
use std::sync::mpsc::{self, Receiver, Sender};
use tray_icon::{
    menu::MenuEvent,
    TrayIconEvent,
};

use crate::single_instance::SingleInstance;

use crate::startup::{self, StartupState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    General,
    Shortcuts,
    About,
}

enum TrayEvent {
    Menu(MenuEvent),
    Icon(TrayIconEvent),
}

pub struct SnipsterApp {
    current_page: Page,
    recording_shortcut: bool,
    shortcut: Shortcut,

    _hotkey_manager: HotkeyManager,
    shortcut_error: Option<String>,

    settings: Settings,
    settings_path: PathBuf,

    tray: SnipsterTray,
    exit_requested: bool,
    tray_event_sender: Sender<TrayEvent>,
    tray_event_receiver: Receiver<TrayEvent>,
    tray_handlers_installed: bool,
    single_instance: SingleInstance,

    startup_state: StartupState,
    startup_error: Option<String>,
    start_hidden_until: Option<std::time::Instant>,
}

impl eframe::App for SnipsterApp {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _frame: &mut eframe::Frame,
    ) {
        self.handle_shortcut_recording(ui.ctx());

        let close_requested =
            ui.ctx().input(|input| input.viewport().close_requested());

        if close_requested && !self.exit_requested {
            // Prevent the application from terminating.
            ui.ctx().send_viewport_cmd(
                egui::ViewportCommand::CancelClose,
            );

            // Hide the native window instead.
            hide_main_window();
        }

        // Use dark mode for the initial prototype.
        ui.ctx().set_visuals(egui::Visuals::dark());

        // Sidebar
        egui::Panel::left("sidebar")
            .exact_size(175.0)
            .show(ui, |ui| {
                ui.add_space(8.0);

                ui.heading("✂ Snipster");

                ui.add_space(28.0);

                ui.label(
                    egui::RichText::new("PREFERENCES")
                        .size(13.0)
                        .weak()
                        .strong(),
                );

                ui.add_space(10.0);

                self.navigation_item(
                    ui,
                    Page::General,
                    "⚙",
                    "General",
                );

                self.navigation_item(
                    ui,
                    Page::Shortcuts,
                    "⌨",
                    "Shortcuts",
                );

                ui.add_space(20.0);

                ui.label(
                    egui::RichText::new("APP")
                        .size(13.0)
                        .weak()
                        .strong(),
                );

                ui.add_space(10.0);

                self.navigation_item(
                    ui,
                    Page::About,
                    "i",
                    "About",
                );
            });

        // Main content
        egui::CentralPanel::default()
            .show(ui, |ui| {
                ui.add_space(12.0);

                match self.current_page {
                    Page::General => self.show_general(ui),
                    Page::Shortcuts => self.show_shortcuts(ui),
                    Page::About => self.show_about(ui),
                }
            });
    }

    fn logic(
        &mut self,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        if let Some(deadline) = self.start_hidden_until {
            hide_main_window();

            if std::time::Instant::now() >= deadline {
                self.start_hidden_until = None;
            }
        }

        self.handle_tray_events(_ctx);
    }
}

impl SnipsterApp {
    pub fn new(
        hotkey_manager: HotkeyManager,
        settings: Settings,
        settings_path: PathBuf,
        single_instance: SingleInstance,
        start_hidden: bool,
        ctx: egui::Context,
    ) -> Self {
        let tray = SnipsterTray::new()
            .expect("Could not initialize Snipster system tray");

        let (tray_event_sender, tray_event_receiver) = mpsc::channel();

        let (startup_state, startup_error) =
            match startup::get_startup_state() {
                Ok(state) => (state, None),
                Err(error) => (
                    StartupState::Disabled,
                    Some(error.to_string()),
                ),
            };

        let mut app = Self {
            current_page: Page::General,
            recording_shortcut: false,
            shortcut: settings.shortcut.clone(),
            _hotkey_manager: hotkey_manager,
            shortcut_error: None,
            settings,
            settings_path,
            tray,
            exit_requested: false,
            tray_event_sender,
            tray_event_receiver,
            tray_handlers_installed: false,
            single_instance,
            startup_state,
            startup_error,
            start_hidden_until: start_hidden.then(|| {
                std::time::Instant::now() + std::time::Duration::from_secs(3)
            }),
        };

        app.install_tray_event_handlers(&ctx);
        app.single_instance.start_listener(ctx);

        app
    }
    
    fn navigation_item(
        &mut self,
        ui: &mut egui::Ui,
        page: Page,
        icon: &str,
        label: &str,
    ) {
        let selected = self.current_page == page;
        let width = ui.available_width();

        let button = egui::Button::new(
            format!("{icon}    {label}")
        )
        .selected(selected)
        .corner_radius(6.0);

        let response = ui.add_sized(
            [width, 36.0],
            button,
        );

        if response.clicked() {
            self.current_page = page;
        }

        response.on_hover_cursor(egui::CursorIcon::PointingHand);

        ui.add_space(4.0);
    }

    fn show_general(&mut self, ui: &mut egui::Ui) {
        ui.heading("General");
        ui.add_space(6.0);

        ui.label(
            egui::RichText::new(
                "Configure how Snipster behaves."
            )
            .weak(),
        );

        ui.add_space(28.0);

        ui.heading(
            egui::RichText::new("Startup")
                .size(17.0),
        );

        ui.add_space(10.0);

        egui::Frame::group(ui.style())
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Launch at startup");

                        ui.label(
                            egui::RichText::new(
                                "Start Snipster when Windows starts."
                            )
                            .weak()
                            .size(12.0),
                        );
                    });

                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            let mut enabled = self.startup_state.is_enabled();

                            if ui.checkbox(&mut enabled, "").changed() {
                                let result = if enabled {
                                    startup::enable_startup()
                                } else {
                                    startup::disable_startup()
                                };

                                match result {
                                    Ok(state) => {
                                        self.startup_state = state;
                                        self.startup_error = None;
                                    }
                                    Err(error) => {
                                        self.startup_error = Some(error.to_string());
                                    }
                                }
                            }
                        },
                    );
                });
            });

        if let Some(error) = &self.startup_error {
            ui.add_space(6.0);

            ui.colored_label(
                egui::Color32::LIGHT_RED,
                error,
            );
        }

        if matches!(
            self.startup_state,
            StartupState::DisabledByUser | StartupState::DisabledByPolicy
        ) {
            ui.add_space(6.0);

            ui.label(
                egui::RichText::new(self.startup_state.description())
                    .weak()
                    .size(12.0),
            );
        }

        ui.add_space(24.0);

        ui.heading(
            egui::RichText::new("Appearance")
                .size(17.0),
        );

        ui.add_space(10.0);

        egui::Frame::group(ui.style())
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Application theme");
                    ui.with_layout(
                        egui::Layout::right_to_left(
                            egui::Align::Center,
                        ),
                        |ui| {
                            ui.label(
                                egui::RichText::new("Dark")
                                    .weak(),
                            );
                        },
                    );
                });
            });
    }

    
    fn show_shortcuts(&mut self, ui: &mut egui::Ui) {
        ui.heading("Shortcuts");

        ui.add_space(6.0);

        ui.label(
            egui::RichText::new(
                "Manage your keyboard shortcuts."
            )
            .weak(),
        );

        ui.add_space(28.0);

        ui.heading(
            egui::RichText::new("Screenshot capture")
                .size(17.0),
        );

        ui.add_space(10.0);

        egui::Frame::group(ui.style())
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Capture screenshot");

                        ui.label(
                            egui::RichText::new(
                                "Start the Windows Snipping Tool."
                            )
                            .weak()
                            .size(12.0),
                        );
                    });

                    ui.with_layout(
                        egui::Layout::right_to_left(
                            egui::Align::Center,
                        ),
                        |ui| {
                            ui.label(
                                egui::RichText::new(
                                    &self.shortcut.display()
                                )
                                .monospace(),
                            );
                        },
                    );
                });
            });

        ui.add_space(12.0);

        let button_label = if self.recording_shortcut {
            "Press a key combination..."
        } else {
            "Change shortcut"
        };

        let change_button = egui::Button::new(
            egui::RichText::new(button_label)
                .size(14.0)
                .strong(),
        )
        .corner_radius(6.0);

        let response = ui.add_sized(
            [180.0, 40.0],
            change_button,
        );

        if response.clicked() {
            self.recording_shortcut = true;
        }

        response.on_hover_cursor(egui::CursorIcon::PointingHand);

        // Display shortcut registration errors
        if let Some(error) = &self.shortcut_error {
            ui.add_space(8.0);

            ui.colored_label(
                egui::Color32::LIGHT_RED,
                error,
            );
        }

        if self.recording_shortcut {
            ui.horizontal(|ui| {
                ui.spinner();

                ui.label(
                    egui::RichText::new(
                        "Waiting for keyboard input. Press a combination."
                    )
                    .weak(),
                );
            });
        }
    }


    fn show_about(&self, ui: &mut egui::Ui) {
        ui.heading("About");

        ui.add_space(6.0);

        ui.label(
            egui::RichText::new(
                "About Snipster and application information."
            )
            .weak(),
        );

        ui.add_space(28.0);

        ui.heading("Snipster");

        ui.label("A lightweight screenshot utility for Windows.");

        ui.add_space(12.0);

        ui.label(
            egui::RichText::new("Version 2.0.0")
                .weak(),
        );

        ui.add_space(20.0);

        ui.hyperlink_to(
            "GitHub repository",
            "https://github.com/Kilaparthi-Himanshu/snipster",
        );
    }

    
    fn handle_shortcut_recording(&mut self, ctx: &egui::Context) {
        if !self.recording_shortcut {
            return;
        }

        let events = ctx.input(|input| input.events.clone());

        for event in events {
            if let egui::Event::Key {
                key,
                pressed: true,
                repeat: false,
                modifiers,
                ..
            } = event
            {
                // Ignore modifier-only key presses.
                let key_name = format!("{key:?}");

                let is_modifier = matches!(
                    key_name.as_str(),
                    "ControlLeft"
                        | "ControlRight"
                        | "ShiftLeft"
                        | "ShiftRight"
                        | "AltLeft"
                        | "AltRight"
                        | "SuperLeft"
                        | "SuperRight"
                );

                if is_modifier {
                    continue;
                }

                // Escape cancels recording.
                if key == egui::Key::Escape {
                    self.recording_shortcut = false;
                    break;
                }

                // Read the actual physical modifier states from Windows.
                let win = unsafe {
                    key_is_down(VK_LWIN) || key_is_down(VK_RWIN)
                };

                let ctrl = modifiers.ctrl;
                let alt = modifiers.alt;
                let shift = modifiers.shift;

                let Some(key_code) = egui_key_to_vk(key) else {
                    continue;
                };

                let new_shortcut = Shortcut {
                    key: key_code,
                    win,
                    ctrl,
                    alt,
                    shift,
                };

                let previous_shortcut = self.shortcut.clone();

                match self._hotkey_manager.set_shortcut(new_shortcut.clone()) {
                    Ok(()) => {
                        self.settings.shortcut = new_shortcut.clone();

                        match self.settings.save(&self.settings_path) {
                            Ok(()) => {
                                self.shortcut = new_shortcut;
                                self.shortcut_error = None;
                            }

                            Err(save_error) => {
                                // Restore the previous active shortcut.
                                let rollback_result = self
                                    ._hotkey_manager
                                    .set_shortcut(previous_shortcut.clone());

                                self.settings.shortcut = previous_shortcut.clone();
                                self.shortcut = previous_shortcut;

                                let message = match rollback_result {
                                    Ok(()) => format!(
                                        "Could not save settings: {save_error}"
                                    ),
                                    Err(rollback_error) => format!(
                                        "Could not save settings: {save_error}. \
                                        Also failed to restore the previous shortcut: \
                                        {rollback_error}"
                                    ),
                                };

                                self.shortcut_error = Some(message);
                            }
                        }
                    }

                    Err(error) => {
                        self.shortcut_error = Some(error.to_string());
                    }
                }

                self.recording_shortcut = false;

                break;
            }
        }
    }

    fn handle_tray_events(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.tray_event_receiver.try_recv() {
            match event {
                TrayEvent::Menu(event) => {
                    if event.id == *self.tray.open_item.id() {
                        self.start_hidden_until = None;
                        show_main_window();
                        ctx.request_repaint();
                    } else if event.id == *self.tray.capture_item.id() {
                        if let Err(error) =
                            self._hotkey_manager.request_capture()
                        {
                            eprintln!("Could not request capture: {error:#}");
                        }
                    } else if event.id == *self.tray.exit_item.id() {
                        self.exit_requested = true;

                        ctx.send_viewport_cmd(
                            egui::ViewportCommand::Close,
                        );
                    }
                }

                TrayEvent::Icon(event) => {
                    if let TrayIconEvent::DoubleClick { .. } = event {
                        self.start_hidden_until = None;
                        show_main_window();
                        ctx.request_repaint();
                    }
                }
            }
        }
    }

    fn install_tray_event_handlers(&mut self, ctx: &egui::Context) {
        // These handlers are process-wide and must only be installed once.
        if self.tray_handlers_installed {
            return;
        }

        let menu_sender = self.tray_event_sender.clone();
        let menu_ctx = ctx.clone();

        let capture_id = self.tray.capture_item.id().clone();
        let exit_id = self.tray.exit_item.id().clone();

        let icon_ctx = menu_ctx.clone();

        TrayIconEvent::set_event_handler(Some(move |event| {
            if let TrayIconEvent::DoubleClick { .. } = event {
                icon_ctx.send_viewport_cmd(
                    egui::ViewportCommand::Visible(true),
                );

                icon_ctx.send_viewport_cmd(
                    egui::ViewportCommand::Minimized(false),
                );

                icon_ctx.send_viewport_cmd(
                    egui::ViewportCommand::Focus,
                );

                icon_ctx.request_repaint();
            }
        }));

        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            if event.id == capture_id {
                // Don't run this on the UI thread: winit has already initialized COM
                // there, so RoInitialize(MULTITHREADED) would fail with a mode conflict.
                std::thread::spawn(|| {
                    if let Err(e) = crate::snipping::launch_capture() {
                        eprintln!("Tray capture launch failed: {e:#}");
                    }
                });
                return;
            }

            if event.id == exit_id {
                std::process::exit(0);
            }

            // "Open" still goes through the channel
            let _ = menu_sender.send(TrayEvent::Menu(event));
            menu_ctx.request_repaint();
        }));
    }
}

fn key_is_down(
    key: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY,
) -> bool {
    unsafe {
        GetAsyncKeyState(key.0 as i32) < 0
    }
}

fn egui_key_to_vk(key: egui::Key) -> Option<u32> {
    let name = format!("{key:?}");

    // Letters A-Z
    if name.len() == 1 && name.as_bytes()[0].is_ascii_alphabetic() {
        return Some(name.as_bytes()[0].to_ascii_uppercase() as u32);
    }

    // Numbers 0-9
    if let Some(number) = name.strip_prefix("Num") {
        if let Ok(number) = number.parse::<u32>() {
            if number <= 9 {
                return Some(0x30 + number);
            }
        }
    }

    match name.as_str() {
        "Tab" => Some(0x09),
        "CapsLock" => Some(0x14),
        "Backtick" => Some(0xC0),

        _ => None,
    }
}

fn hide_main_window() {
    unsafe {
        if let Ok(hwnd) = FindWindowW(None, w!("Snipster")) {
            if !hwnd.0.is_null() {
                let _ = ShowWindow(hwnd, SW_HIDE);
            }
        }
    }
}

pub(crate) fn show_main_window() {
    unsafe {
        if let Ok(hwnd) = FindWindowW(None, w!("Snipster")) {
            if !hwnd.0.is_null() {
                let _ = ShowWindow(hwnd, SW_RESTORE);
                let _ = ShowWindow(hwnd, SW_SHOW);
                let _ = SetForegroundWindow(hwnd);
            }
        }
    }
}
