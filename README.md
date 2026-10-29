# su-profile-switcher

> Lightweight, high-performance profile switcher and automation manager for Trimble SketchUp (2022 - 2026+), designed with Windows 11 Fluent Design and Lepo.co's WPF-UI.

[![License: MIT](https://img.shields.io/badge/License-MIT-emerald.svg)](LICENSE)
[![Platform: Windows](https://img.shields.io/badge/Platform-Windows%2011%20%7C%2010-blue.svg)]()
[![.NET: 8.0 WPF](https://img.shields.io/badge/.NET-8.0%20WPF-purple.svg)]()
[![Single File: Portable](https://img.shields.io/badge/Release-Single%20File%20Portable-success.svg)]()

---

## Overview

**su-profile-switcher** enables SketchUp professionals, 3D visualizers, and architects to hot-swap plugin configurations, custom keyboard shortcuts, and toolbar layouts instantly (sub-0.1s) across generic or customized profile sets (e.g., *Modeling*, *V-Ray Rendering*, *Layout Production*, or team-shared profiles).

Traditional profile switches in SketchUp require manual copying or deleting gigabytes of plugin files. **su-profile-switcher** solves this by leveraging filesystem-level atomic moves (`Directory.Move`), eliminating data duplication while keeping profile swapping instantaneous regardless of library size.

---

## Key Features

- **Instant Hot-Swapping (O(1) Filesystem Operation)**
  Moves active plugin folders and updates preferences within milliseconds on NTFS volumes.
- **Full Configuration Synchronization**
  Swaps all essential SketchUp configuration targets simultaneously:
  - `Plugins/` directory (Ruby extensions, libraries, loader scripts)
  - `SharedPreferences.json` (Custom keyboard shortcuts and general preferences)
  - `PrivatePreferences.json` (Toolbar layouts, dock states, dialog positions)
- **Multi-Version Auto-Detection**
  Scans and supports Trimble SketchUp installations from 2022 through 2026+ across both Roaming and Local AppData.
- **Safe Process Management**
  Gracefully requests SketchUp to save work via `CloseMainWindow()` before swapping, waits for clean process termination, and automatically relaunches SketchUp afterwards.
- **Cloud & Local Profile Import**
  - **Google Drive Direct Downloader**: Parses Google Drive share links and automatically handles large-file confirmation tokens (`confirm=...`).
  - **Smart Zip Mapper**: Intelligently unpacks archives whether they contain root `Plugins/` directories, loose `.rb` files, or preferences.
- **Export / Packaging to Zip**
  One-click packaging of the active or any selected profile into a standardized `.zip` archive ready to upload to Google Drive or share across team workstations.
- **Desktop Shortcut Generator**
  Employs native Win32 COM `IShellLinkW` / `IPersistFile` to generate one-click Desktop shortcuts with silent execution arguments (`--switch "<ProfileName>"`).
- **System Tray (Taskbar) Integration**
  Minimizes to the Windows Notification Area with a rich context menu:
  - Active profile status indicator
  - Quick switch between profiles directly from taskbar
  - Toggle "Start with Windows" (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`)
  - Instant language switcher
  - Open Dashboard / Exit
- **Bilingual Interface**
  Full English and Pure Vietnamese localization switchable at runtime.
- **100% Portable**
  Single self-contained `.exe` binary with zero dependencies. Runs out-of-the-box on any Windows machine.

---

## Target Paths

The application operates across standard SketchUp data directories:

| Component | Target Location | Description |
| :--- | :--- | :--- |
| **Plugins** | `%APPDATA%\SketchUp\SketchUp 202x\SketchUp\Plugins\` | Active Ruby plugins & scripts |
| **Shortcuts** | `%APPDATA%\SketchUp\SketchUp 202x\SketchUp\SharedPreferences.json` | Shortcuts and settings |
| **Toolbars** | `%LOCALAPPDATA%\SketchUp\SketchUp 202x\SketchUp\PrivatePreferences.json` | Workspace & dock layout |
| **Profile Storage**| `%APPDATA%\SketchUp\SketchUp 202x\SketchUp\Profiles\<ProfileName>\` | Local profiles archive |

---

## CLI Automation & Silent Mode

You can trigger silent profile swaps via command-line or external scripts:

```bash
# Switch to a specific profile silently (closes SU -> swaps -> relaunches SU -> exits)
su-profile-switcher.exe --switch "Profile A"

# Target a specific SketchUp version explicitly
su-profile-switcher.exe --switch "V-Ray Setup" --version "SketchUp 2026"

# Launch minimized to tray on system startup
su-profile-switcher.exe --minimized
```

---

## Building from Source

### Prerequisites
- Windows 10 (19041+) or Windows 11
- [.NET 8.0 SDK](https://dotnet.microsoft.com/download/dotnet/8.0)

### Clone & Build
```bash
git clone https://github.com/phathua/su-profile-switcher.git
cd su-profile-switcher

# Debug build
dotnet build -c Debug

# Single-file self-contained release build
dotnet publish -c Release -r win-x64 /p:PublishSingleFile=true /p:SelfContained=true /p:PublishReadyToRun=true -o ./Build
```

The resulting standalone executable will be generated at `./Build/su-profile-switcher.exe`.

---

## License

This project is licensed under the [MIT License](LICENSE).
