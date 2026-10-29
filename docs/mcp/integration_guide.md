# MCP Client Integration Guide: SketchUp Profile Switcher

This guide details how to integrate the native Rust `sups` MCP server with major AI coding assistants and orchestrators.

---

## 1. Binary Path Conventions

Select the binary location based on your setup:

- **Local Development / Source Build**:
  `d:\\2-VibeCode\\1_sketchup-extensions\\su-profile-switcher\\src-tauri\\target\\release\\sups.exe`
- **Standard Installed Application (Per-user AppData)**:
  `%LOCALAPPDATA%\\Programs\\SketchUp Profile Switcher\\sups.exe` (or `su-profile-switcher.exe`)

---

## 2. Google Antigravity Integration

### Option A: Standard Plugin Installation (Recommended)
Place or link the plugin directory into `~/.gemini/config/plugins/su-profile-switcher/`.

`mcp_config.json` inside the plugin directory:
```json
{
  "mcpServers": {
    "su-profile-switcher": {
      "command": "d:\\2-VibeCode\\1_sketchup-extensions\\su-profile-switcher\\src-tauri\\target\\release\\sups.exe",
      "args": ["mcp"]
    }
  }
}
```

### Option B: User-Level MCP Configuration
Add directly to `~/.gemini/config/mcp_config.json`:
```json
{
  "mcpServers": {
    "su-profile-switcher": {
      "command": "d:\\2-VibeCode\\1_sketchup-extensions\\su-profile-switcher\\src-tauri\\target\\release\\sups.exe",
      "args": ["mcp"]
    }
  }
}
```

---

## 3. Anthropic Claude Desktop Integration

Edit `%APPDATA%\\Claude\\claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "sketchup-profile-switcher": {
      "command": "d:\\2-VibeCode\\1_sketchup-extensions\\su-profile-switcher\\src-tauri\\target\\release\\sups.exe",
      "args": ["mcp"]
    }
  }
}
```

Restart Claude Desktop. Verify that the hammer/tool icon displays `sups_*` tools.

---

## 4. Claude Code CLI Integration

Add to project-level or global `.claude.json`:

```json
{
  "mcpServers": {
    "sketchup-profile-switcher": {
      "command": "d:\\2-VibeCode\\1_sketchup-extensions\\su-profile-switcher\\src-tauri\\target\\release\\sups.exe",
      "args": ["mcp"]
    }
  }
}
```

Or invoke via Claude Code CLI:
```bash
claude mcp add sketchup-profile-switcher d:\\2-VibeCode\\1_sketchup-extensions\\su-profile-switcher\\src-tauri\\target\\release\\sups.exe mcp
```

---

## 5. Cursor IDE Integration

Create or edit `.cursor/mcp.json` in your workspace root:

```json
{
  "mcpServers": {
    "sketchup-profile-switcher": {
      "command": "d:\\2-VibeCode\\1_sketchup-extensions\\su-profile-switcher\\src-tauri\\target\\release\\sups.exe",
      "args": ["mcp"]
    }
  }
}
```

In Cursor Settings > Features > MCP, click Refresh to ensure the green status light appears.

---

## 6. Windsurf, Roo Code & Cline Integration

In `.windsurfrules` / `mcp_config.json` or VS Code extension settings (`cline_mcp_settings.json`):

```json
{
  "mcpServers": {
    "sketchup-profile-switcher": {
      "command": "d:\\2-VibeCode\\1_sketchup-extensions\\su-profile-switcher\\src-tauri\\target\\release\\sups.exe",
      "args": ["mcp"]
    }
  }
}
```

---

## 7. Troubleshooting & Verification

Test the server stdio pipe directly in PowerShell:

```powershell
'{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | .\src-tauri\target\release\sups.exe mcp
```

A valid JSON-RPC 2.0 response with `serverInfo` confirms operational readiness.
