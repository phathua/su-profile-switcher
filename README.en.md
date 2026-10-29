# SketchUp Profile Switcher (SuPS)

<p align="center">
  <img src="public/icon.svg" width="96" height="96" alt="SketchUp Profile Switcher Logo" />
</p>

<p align="center">
  <strong>Ultra-lightweight, safe, and high-performance profile switcher for SketchUp.</strong><br>
  Built with <strong>Tauri v2</strong>, <strong>Svelte 5</strong>, and <strong>Rust</strong>.
</p>

<p align="center">
  <a href="#-key-features">Features</a> •
  <a href="#-screenshots">Screenshots</a> •
  <a href="#-download--installation">Download</a> •
  <a href="#-usage-guide">Usage</a> •
  <a href="#-tech-stack">Tech Stack</a> •
  <a href="README.md">Tài liệu tiếng Việt</a>
</p>

---

## 📸 Screenshots

| Main Dashboard (Dark Mode) | Safe Deletion Confirmation |
| :---: | :---: |
| ![Main Dashboard](docs/screenshots/main_interface.png) | ![Delete Confirmation](docs/screenshots/delete_modal.png) |

---

## ⚡ Key Features

- 🚀 **Blazing Fast & Ultra-Lightweight**: Minimal memory footprint (~20MB RAM), independent of bulky runtimes or full Chromium browsers.
- 🎨 **Visual Profile Customization**: Freely select Phosphor Icons (curated design & 3D workflow icons), assign accent colors, and manage distinct configurations.
- ⌨️ **Global Hotkeys**: Switch profiles from anywhere with registered system-wide shortcuts (e.g., `Ctrl+Alt+1`, `Ctrl+Shift+D`).
- 🔔 **Windows Native Notifications**: Utilizes native Windows Toast Notification APIs with Action Center banners and audio cues.
- 🛡️ **Data Integrity Protection**: Real-time process monitoring detects active SketchUp instances, prompting confirmation and offering single-click **Force Close** protection.
- 🔍 **Multi-Version Auto Detection**: Seamlessly detects and manages configurations across SketchUp 2021 through SketchUp 2026.
- 🧩 **Render Plugin Isolation**: Keep render engines (V-Ray, Enscape, D5 Render, Thea Render) intact to avoid licensing and path issues.
- 📦 **Zip Backup & Migration**: Export portable `.zip` archives of any profile for easy migration or teamwork sharing.

---

## 📥 Download & Installation

Grab the latest release from the [**GitHub Releases**](https://github.com/phathua/su-profile-switcher/releases) page:

- `SuPS-x64-v*.zip`: Standard 64-bit Windows (Recommended).
- `SuPS-x86-v*.zip`: 32-bit Windows.
- `SuPS-arm64-v*.zip`: Windows on ARM (Surface Pro Copilot+, Snapdragon X Elite).

> 💡 **Portable Application**: No setup required. Extract the `.zip` archive and launch the `.exe`.

---

## 🖥️ Usage Guide

1. **Select Version**: Pick your target SketchUp version from the top-right dropdown.
2. **Create Profile**: Click `+ New Profile`, specify a name, and check target components (Plugins, Shortcuts, Toolbars, Materials, Templates).
3. **Customize Profile & Hotkeys**: Click the pencil icon on any profile item to choose an icon, color dot, and record a global shortcut.
4. **Switch Configuration**:
   - Click the switch icon next to the profile in the list.
   - Or right-click the System Tray icon to switch instantly.
   - Or trigger the assigned global keyboard shortcut.
5. **Launch / Terminate SketchUp**: The top header button dynamically adapts between `Launch SketchUp` (Sky Blue) and `Force Close SketchUp` (Red) based on the process state.

---

## 🛠️ Tech Stack

- **Core Engine**: [Rust](https://www.rust-lang.org/) + [Tauri v2](https://v2.tauri.app/)
- **Frontend Framework**: [Svelte 5](https://svelte.dev/) (Runes reactivity)
- **Styling**: [Tailwind CSS v4](https://tailwindcss.com/)
- **Icon System**: [Phosphor Icons](https://phosphoricons.com/)
- **CI/CD Pipeline**: GitHub Actions with Node.js 24, multi-architecture packaging, and automated changelog releases.

---

## 📄 License

Licensed under the [MIT License](LICENSE).
