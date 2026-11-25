# Snipster

<div align="center">
  <img src="assets/icons/master.png" alt="Snipster Logo" width="128" />

  <h3>A lightweight, native Windows screenshot utility.</h3>

  **Capture. Copy. Paste. Done.**

  A faster, clipboard-first screenshot workflow powered by the built-in Windows Snipping Tool.

  <p>
    <img src="https://img.shields.io/badge/Version-2.0.0-blue" alt="Version 2.0.0" />
    <img src="https://img.shields.io/badge/Language-Rust-orange?logo=rust" alt="Built with Rust" />
    <img src="https://img.shields.io/badge/Platform-Windows-0078D4?logo=windows" alt="Windows" />
    <img src="https://img.shields.io/badge/License-MIT-green.svg" alt="MIT License" />
  </p>
</div>

---

## Overview

Snipster brings a faster screenshot workflow to Windows without replacing the native Snipping Tool.

Press a custom global keyboard shortcut, select an area of your screen using the genuine Windows Snipping Tool interface, and have the captured image copied directly to your clipboard.

No additional screenshot editor. No unnecessary windows. No manual saving.

Snipster runs quietly in the system tray and stays ready whenever you need it.

## Features

| Feature | Description |
|---|---|
| **Native screenshot selection** | Uses the genuine Windows Snipping Tool interface. |
| **Clipboard-first workflow** | Automatically copies captured screenshots to the Windows clipboard. |
| **Global keyboard shortcut** | Trigger screenshots from anywhere in Windows. |
| **System tray integration** | Runs quietly in the background with convenient tray controls. |
| **Startup integration** | Optionally launches automatically when Windows starts. |
| **Startup management** | Manage automatic startup through Windows Settings or Task Manager. |
| **Single-instance execution** | Avoids unnecessary duplicate application instances. |
| **Lightweight** | Built with Rust and native Windows APIs. |
| **MSIX packaging** | Distributed as a signed Windows application package. |

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Win + Shift + S` | Opens the default Windows screenshot experience. |
| `Win + Shift + Ctrl + S` | Opens Snipster's screenshot workflow. |

After selecting an area, the screenshot is automatically copied to your clipboard.

Simply press `Ctrl + V` in any application that accepts images.

The Snipster shortcut can also be customized through the application settings.

## Installation

### Option 1: GitHub Releases (Recommended)

1. Visit the [GitHub Releases](../../releases) page.
2. Download the latest Snipster release ZIP.
3. Extract the ZIP into a folder.
4. Double-click **Install Snipster.bat**.
5. Approve the Windows administrator permission prompt.
6. Wait for installation to finish.

The installer verifies the signing certificate, installs the MSIX package, and launches Snipster automatically.

Once launched, Snipster appears in the Windows system tray and is ready to use.

To uninstall, double-click **Uninstall Snipster.bat** in the extracted folder.

### Option 2: Build and Install from Source

Clone the repository:

```powershell
git clone https://github.com/Kilaparthi-Himanshu/snipster.git
cd snipster
```

Build the application and generate the installer:

```powershell
.\scripts\build-installer.ps1
```

After a successful build, open the generated `installer` directory:

```powershell
cd installer
```

Double-click:

```text
Install Snipster.bat
```

Alternatively, open the `installer` folder in File Explorer and launch the installer.

To uninstall, double-click:

```text
Uninstall Snipster.bat
```

### Security Notice

Snipster releases are currently signed using a self-signed development certificate.

The installer verifies the certificate fingerprint before importing the public certificate into the Local Machine Trusted People certificate store.

Only install packages and trust certificates from sources you trust. Always verify the certificate fingerprint before installation.

## System Tray

Snipster is designed to stay in the background rather than occupy your taskbar.

Its system tray menu provides access to:

- **Open Snipster** — Open the application settings.
- **Capture Screenshot** — Start a screenshot capture.
- **Exit** — Completely close Snipster.

Closing the settings window hides it in the system tray instead of terminating the application.

## Startup Behavior

Snipster can register itself as a Windows startup application.

You can manage its startup behavior through:

- Task Manager → Startup apps
- Windows Settings → Apps → Startup

Disable Snipster there if you don't want it to launch automatically when you sign in.

You can still open it manually from the Windows Start menu.

## How It Works

Snipster is written in Rust and integrates directly with Windows APIs and Windows Runtime components.

The screenshot workflow follows these steps:

1. **Global shortcut registration** — Registers a keyboard shortcut using the Windows `RegisterHotKey` API.
2. **Native capture launch** — Launches the Windows Snipping Tool selection interface through the `ms-screenclip://` protocol.
3. **Capture callback** — Receives the screenshot callback through Snipster's registered application protocol.
4. **Capture retrieval** — Redeems the one-time capture token to retrieve the temporary screenshot file.
5. **Clipboard transfer** — Copies the image to the Windows clipboard using native Win32 clipboard APIs.
6. **Temporary file cleanup** — Deletes the temporary capture file after processing.

Snipster does not intentionally retain a permanent screenshot copy. Windows Snipping Tool's own saving preferences may also affect capture storage behavior.

## Architecture

```text
                 User
                  |
                  v
          Global Keyboard
             Shortcut
                  |
                  v
          Snipster Hotkey
             Manager
                  |
                  v
       Windows Snipping Tool
          Native Selection
                  |
                  v
           Screenshot
             Capture
                  |
                  v
        Snipster Protocol
             Callback
                  |
                  v
          Capture Token
             Retrieval
                  |
                  v
          Win32 Clipboard
                  |
                  v
          Windows Clipboard
                  |
                  v
          Ctrl + V Anywhere
```

## Technology Stack

| Technology | Purpose |
|---|---|
| **Rust** | Core application implementation. |
| **`windows` crate** | Win32 API and Windows Runtime integration. |
| **WinRT** | Windows application and capture-related APIs. |
| **egui / eframe** | Application settings interface. |
| **`tray-icon`** | Native system tray integration. |
| **Windows APIs** | Global shortcuts, clipboard operations, window management, and process coordination. |
| **MSIX** | Windows application packaging and installation. |
| **PowerShell** | Build, packaging, signing, installation, and uninstallation automation. |

## Build from Source

### Requirements

- Windows 10 version 2004 (build 19041) or later, or Windows 11.
- Rust toolchain and Cargo.
- Windows SDK.
- PowerShell.
- A Windows-compatible development environment.

### Build the Application

Clone the repository:

```powershell
git clone https://github.com/Kilaparthi-Himanshu/snipster.git
cd snipster
```

Build the optimized application binaries:

```powershell
cargo build --release --bins
```

The executables will be generated under:

```text
target/release/
├── snipster.exe
└── startup_launcher.exe
```

### Generate the MSIX Installer

Snipster includes an automated packaging script that:

1. Builds the Rust executables.
2. Updates the MSIX package payload.
3. Copies the application manifest and visual assets.
4. Generates the MSIX package using MakeAppx.
5. Signs the package using SignTool.
6. Verifies the package signature.
7. Prepares the distributable installer directory.

Run:

```powershell
.\scripts\build-installer.ps1
```

The packaging process requires:

- Windows SDK tools (`MakeAppx.exe` and `SignTool.exe`).
- The Snipster development signing certificate installed in the current user's Personal certificate store.

After a successful build, the `installer` directory contains the generated MSIX package, public certificate, and installation/uninstallation scripts.

## Project Structure

```text
snipster/
├── assets/
│   └── icons/
│       ├── master.png
│       ├── app.ico
│       └── ...
│
├── package/
│   ├── Assets/
│   ├── payload/
│   └── Package.appxmanifest
│
├── scripts/
│   ├── build-installer.ps1
│   ├── Install Snipster.bat
│   ├── Install-Snipster.ps1
│   ├── Uninstall Snipster.bat
│   └── Uninstall-Snipster.ps1
│
├── src/
│   ├── app.rs
│   ├── clipboard.rs
│   ├── hotkey.rs
│   ├── main.rs
│   ├── settings.rs
│   ├── shortcut.rs
│   ├── single_instance.rs
│   ├── snipping.rs
│   ├── startup.rs
│   └── tray.rs
│
├── build.rs
├── Cargo.toml
├── LICENSE
└── README.md
```

## Uninstallation

Snipster can be removed using either method.

**Graphical method**

Double-click:

```text
Uninstall Snipster.bat
```

inside the extracted installer directory.

**PowerShell method**

```powershell
Get-AppxPackage -Name Snipster | Remove-AppxPackage
```

The signing certificate is retained after uninstalling. If it is no longer needed, it can be removed separately from the Local Machine Trusted People certificate store.

## Contributing

Contributions, bug reports, feature requests, and suggestions are welcome.

Feel free to open an issue or submit a pull request.

## License

Snipster is licensed under the [MIT License](LICENSE).

You are free to use, modify, distribute, and build upon Snipster, subject to the terms of the license.

Third-party dependencies remain subject to their respective licenses.

---

<div align="center">
  <img src="assets/icons/master.png" alt="Snipster" width="56" />

  **Snipster 2.0**

  Built with Rust. Made for Windows.

  <sub>Capture. Copy. Paste. Done.</sub>
</div>
