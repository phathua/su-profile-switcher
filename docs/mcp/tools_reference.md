# MCP Tools Formal Reference: SketchUp Profile Switcher

This document details the exact JSON schema, argument contracts, and response formats for all 8 tools exposed by the `su-profile-switcher-mcp` server.

---

## 1. `sups_list_versions`

Detects all installed SketchUp versions on the Windows host and identifies the currently selected version.

### Input Schema
```json
{
  "type": "object",
  "properties": {},
  "required": []
}
```

### Return Format
```json
{
  "installed_versions": [
    "SketchUp 2026",
    "SketchUp 2025"
  ],
  "selected_version": "SketchUp 2025"
}
```

---

## 2. `sups_list_profiles`

Lists all saved profiles, active selection, and recent switch history for a given version.

### Input Schema
```json
{
  "type": "object",
  "properties": {
    "version": {
      "type": "string",
      "description": "Target SketchUp version (e.g. 'SketchUp 2026'). If omitted, defaults to the active version."
    }
  },
  "required": []
}
```

### Return Format
```json
{
  "version": "SketchUp 2025",
  "active_profile": "Default",
  "profiles": [
    "Default",
    "Modeling_Fast",
    "Production_VRay"
  ],
  "recent_profiles": [
    "Default",
    "Production_VRay"
  ],
  "metadata": {
    "Default": { "icon": "Folder", "color": "#00c8ff", "shortcut": "Ctrl+Alt+1" }
  }
}
```

---

## 3. `sups_get_current_profile`

Calculates the SHA-256 configuration hash of the active SketchUp AppData directory and matches it against saved profile snapshots on disk.

### Input Schema
```json
{
  "type": "object",
  "properties": {
    "version": {
      "type": "string",
      "description": "Target SketchUp version (e.g. 'SketchUp 2026')."
    }
  },
  "required": []
}
```

### Return Format
```json
{
  "version": "SketchUp 2025",
  "active_profile_recorded": "Default",
  "configuration_hash": "539f7043367c1ccc12dfc9192f3a0dc00b9c7960a874a4b126076596afdf0e31",
  "matching_profile_on_disk": "Default",
  "is_render_protection_enabled": true
}
```

---

## 4. `sups_switch_profile`

Switches the active SketchUp configuration to a specified target profile.

### Input Schema
```json
{
  "type": "object",
  "properties": {
    "profile_name": {
      "type": "string",
      "description": "Name of the target profile to activate."
    },
    "version": {
      "type": "string",
      "description": "Target SketchUp version (e.g. 'SketchUp 2026')."
    },
    "force_kill_sketchup": {
      "type": "boolean",
      "description": "If true, gracefully terminates running SketchUp instances before switching. Default: false."
    }
  },
  "required": ["profile_name"]
}
```

### Return Format
```json
{
  "success": true,
  "switched_to": "Production_VRay",
  "version": "SketchUp 2026",
  "message": "Successfully switched to profile 'Production_VRay' for SketchUp 2026."
}
```

---

## 5. `sups_save_profile`

Saves the current live SketchUp AppData configuration as a new profile or updates an existing profile with customizable component flags.

### Input Schema
```json
{
  "type": "object",
  "properties": {
    "profile_name": {
      "type": "string",
      "description": "Name for the saved profile."
    },
    "version": {
      "type": "string",
      "description": "Target SketchUp version."
    },
    "save_plugins": {
      "type": "boolean",
      "description": "Include Plugins folder. Default: true."
    },
    "save_shortcuts": {
      "type": "boolean",
      "description": "Include Shortcuts preferences. Default: true."
    },
    "save_toolbars": {
      "type": "boolean",
      "description": "Include Toolbar layout preferences. Default: false."
    },
    "save_materials": {
      "type": "boolean",
      "description": "Include Materials folder. Default: false."
    },
    "save_templates": {
      "type": "boolean",
      "description": "Include Templates folder. Default: false."
    }
  },
  "required": ["profile_name"]
}
```

---

## 6. `sups_inspect_profile`

Returns a structural inspection of plugins and customized shortcuts within a profile.

### Input Schema
```json
{
  "type": "object",
  "properties": {
    "profile_name": {
      "type": "string",
      "description": "Name of the profile to inspect."
    },
    "version": {
      "type": "string",
      "description": "Target SketchUp version."
    }
  },
  "required": ["profile_name"]
}
```

### Return Format
```json
{
  "profileName": "Production_VRay",
  "plugins": [
    {
      "name": "VRayForSketchUp",
      "filename": "vray.rb",
      "isExcluded": false,
      "iconData": null
    }
  ],
  "customShortcuts": [
    {
      "raw": "Edit/Paste in Place\tShift+V",
      "keyDisplay": "Shift+V",
      "command": "Edit/Paste in Place",
      "isCustom": true,
      "originalKey": null
    }
  ],
  "totalShortcutsCount": 45,
  "defaultShortcutsCount": 22
}
```

---

## 7. `sups_sketchup_status`

Checks if any SketchUp process is currently running and detects the executable path.

### Input Schema
```json
{
  "type": "object",
  "properties": {
    "version": {
      "type": "string",
      "description": "Target SketchUp version."
    }
  },
  "required": []
}
```

### Return Format
```json
{
  "version": "SketchUp 2026",
  "is_running": false,
  "executable_path": "C:\\Program Files\\SketchUp\\SketchUp 2026\\SketchUp\\SketchUp.exe"
}
```

---

## 8. `sups_sketchup_control`

Launches or terminates SketchUp instances.

### Input Schema
```json
{
  "type": "object",
  "properties": {
    "action": {
      "type": "string",
      "enum": ["launch", "kill"],
      "description": "Action to perform: 'launch' or 'kill'."
    },
    "version": {
      "type": "string",
      "description": "Target SketchUp version to launch."
    }
  },
  "required": ["action"]
}
```
