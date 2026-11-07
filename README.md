# Snipster

**A lightweight, native Windows screenshot utility.**

Snipster brings a faster, clipboard-first screenshot workflow to Windows by using the built-in Windows Snipping Tool.

Press a custom keyboard shortcut, select an area of your screen, and paste the screenshot wherever you need it. No extra screenshot editor, no console windows, and no unnecessary background UI.

## Features

- **Native screenshot selection** — Uses the genuine Windows Snipping Tool interface.
- **Clipboard-first workflow** — Captures are automatically copied to the Windows clipboard.
- **No console windows** — Runs silently in the background.
- **Global keyboard shortcut** — Trigger screenshots from anywhere.
- **Windows startup integration** — Registers Snipster as a Windows startup application.
- **Task Manager integration** — Enable or disable automatic startup through Windows Startup Apps.
- **Lightweight** — Built with Rust and native Windows APIs.

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Win + Shift + S` | Default Windows screenshot experience |
| `Win + Shift + Ctrl + S` | Snipster screenshot → clipboard |

After selecting an area, simply press `Ctrl + V` in any application that accepts images.

## Installation

### Option 1: Install from a GitHub Release

1. Open the [Releases](../../releases) page.
2. Download the latest Snipster release ZIP.
3. Extract the ZIP.
4. Double-click **Install Snipster.bat**.
5. Approve the administrator permission prompt.

The installer verifies the signing certificate, installs Snipster, and launches the application.

To uninstall, double-click **Uninstall Snipster.bat** in the extracted folder.

### Option 2: Install by Cloning the Repository

You can also install Snipster directly from the source repository.

Clone the repository:

```powershell
git clone https://github.com/OWNER/REPOSITORY.git
cd REPOSITORY
```

Open the `installer` directory:

```powershell
cd installer
```

Double-click:

```text
Install Snipster.bat
```

Alternatively, open the `installer` folder in File Explorer and double-click the installer.

To uninstall, double-click:

```text
Uninstall Snipster.bat
```

> **Security notice:** Snipster releases are currently signed using a self-signed development certificate. The installer imports the public certificate into the Local Machine Trusted People store. Only trust certificates from sources you trust. Verify the certificate fingerprint before installation.

## Build from Source

### Requirements

- Windows 10 version 2004 (build 19041) or later, or Windows 11.
- Rust toolchain.
- Windows SDK.
- PowerShell.

Clone the repository:

```powershell
git clone https://github.com/OWNER/REPOSITORY.git
cd REPOSITORY
```

Build the optimized executable:

```powershell
cargo build --release
```

The executable will be generated at:

```text
target/release/snipster.exe
```

### Generate the Installer

Snipster includes a packaging script that builds the executable, generates the MSIX package, signs it, and copies the generated release files into the `installer` directory.

Run:

```powershell
.\scripts\build-installer.ps1
```

This requires:

- Windows SDK tools (`MakeAppx.exe` and `SignTool.exe`).
- The Snipster development signing certificate installed in your current user's Personal certificate store.

After successful execution, the `installer` directory will contain the updated MSIX package and public certificate alongside the installation scripts.

## Startup Behavior

Snipster registers itself as a Windows startup application.

You can manage its startup behavior through:

- Task Manager → Startup apps
- Settings → Apps → Startup

Disable Snipster there if you don't want it to start automatically when you sign in. You can still launch it manually from the Start menu.

## How It Works

Snipster is written in Rust and integrates directly with Windows:

1. Registers a global keyboard shortcut using the Windows API.
2. Launches the native Snipping Tool using the `ms-screenclip://` protocol.
3. Receives the screenshot callback through its registered application protocol.
4. Redeems the capture token to retrieve the screenshot.
5. Transfers the image to the Windows clipboard using native clipboard APIs.
6. Deletes the temporary capture file after processing.

Snipster does not intentionally retain a permanent screenshot copy. Windows Snipping Tool's own saving preferences may also affect capture storage behavior.

## Technology Stack

- Rust
- Windows API (`windows` crate)
- Windows Runtime (WinRT)
- Windows MSIX packaging
- Native Windows clipboard APIs

## Uninstallation

You can uninstall Snipster using either method:

**Graphical method**

Double-click `Uninstall Snipster.bat` inside the `installer` folder.

**PowerShell method**

```powershell
Get-AppxPackage -Name Snipster | Remove-AppxPackage
```

The signing certificate is retained after uninstalling. It can be removed separately from the Local Machine Trusted People certificate store if it is no longer needed.

## Contributing

Contributions, bug reports, and suggestions are welcome.

Feel free to open an issue or submit a pull request.

## License

[Choose a license and add its corresponding LICENSE file.]

---

Built with Rust for Windows.