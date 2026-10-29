# AI Agent Operating Guide: SketchUp Profile Switcher MCP

This document contains authoritative instructions for autonomous AI agents (such as Google Antigravity, Anthropic Claude, OpenAI Codex, and Cursor) interacting with Trimble SketchUp installations via the `su-profile-switcher-mcp` server.

---

## 1. Core Operating Invariants

When operating the SketchUp Profile Switcher MCP server, you MUST strictly adhere to the following safety invariants:

### Invariant 1: Non-Destructive Switching (Process Gating)
- **Rule**: Never attempt to switch profiles while SketchUp is actively running without explicit user confirmation or an explicit force instruction.
- **Verification**: Always query `sups_sketchup_status` prior to invoking `sups_switch_profile`.
- **Handling**: If SketchUp is running and the user requested a profile switch, inform the user that SketchUp is running, or set `force_kill_sketchup: true` only if the user explicitly confirmed that unsaved work has been saved.

### Invariant 2: Extension & Render Protection
- **Rule**: Complex rendering suites (Chaos V-Ray, Enscape 3D, D5 Render, Thea Render) frequently bind licenses to specific machine directories or registry keys.
- **Handling**: The switcher automatically excludes these directories when `exclude_render_and_default_plugins` is enabled. Do not modify or delete these excluded folders unless explicitly directed.

### Invariant 3: Configuration Hash Integrity
- **Rule**: Profile metadata can become desynchronized if a human user manually copies files into `%APPDATA%\SketchUp\SketchUp <Year>\SketchUp\Plugins`.
- **Handling**: Use `sups_get_current_profile` to evaluate `matching_profile_on_disk` and `configuration_hash`. If `matching_profile_on_disk` is null or does not match `active_profile_recorded`, notify the user that local modifications exist before overwriting or switching.

---

## 2. Decision Tree & Workflows

```
                                 [Agent Intent]
                                        |
         +------------------------------+------------------------------+
         |                                                             |
 [Prepare Specific Workflow]                               [Audit / Diagnostic]
         |                                                             |
 Query sups_sketchup_status                               Query sups_list_versions
         |                                                             |
 Is SketchUp Running?                                     Query sups_list_profiles
   /           \                                                       |
 [Yes]        [No]                                        Query sups_inspect_profile
   |            |                                                      |
 Prompt user   Invoke sups_switch_profile                 Inspect extension list & shortcuts
 or pass force        |
               Invoke sups_sketchup_control (launch)
```

### Workflow A: Automated Environment Setup (e.g. Rendering Task)
1. **Discover Installed Versions**:
   Call `sups_list_versions` to verify which versions are available (e.g., `"SketchUp 2026"`).
2. **Inspect Existing Profiles**:
   Call `sups_list_profiles` with `version: "SketchUp 2026"`. Find the profile tailored for visualization (e.g., `"V-Ray Production"`).
3. **Verify Process State**:
   Call `sups_sketchup_status`. If running, prompt the user or terminate safely.
4. **Execute Switch**:
   Call `sups_switch_profile` with `profile_name: "V-Ray Production"`.
5. **Launch Application**:
   Call `sups_sketchup_control` with `action: "launch"` to start SketchUp with the clean environment.

### Workflow B: Safe Extension Testing / Sandboxing
1. **Take Baseline Snapshot**:
   Call `sups_save_profile` with `profile_name: "Pre_Test_Backup"`, saving plugins and shortcuts.
2. **Perform Modification**:
   Install or modify the experimental Ruby plugin in the active Plugins folder.
3. **Audit**:
   Call `sups_inspect_profile` to inspect newly registered plugins and shortcuts.
4. **Rollback if Failed**:
   If an extension causes instability or crashes, call `sups_switch_profile` with `profile_name: "Pre_Test_Backup"` to restore the exact pre-test state.

---

## 3. Tool Summary & Dispatch Matrix

| Tool Name | Primary Purpose | Key Parameters | Idempotent |
| :--- | :--- | :--- | :--- |
| `sups_list_versions` | Detect all installed SketchUp versions | None | Yes |
| `sups_list_profiles` | Get available profiles and active selection | `version` (optional) | Yes |
| `sups_get_current_profile` | Compute hash and detect uncommitted disk diffs | `version` (optional) | Yes |
| `sups_switch_profile` | Activate target profile | `profile_name`, `force_kill_sketchup` | Yes |
| `sups_save_profile` | Create new snapshot or update existing profile | `profile_name`, component flags | Yes |
| `sups_inspect_profile` | Detailed breakdown of plugins and shortcut diffs | `profile_name`, `version` | Yes |
| `sups_sketchup_status` | Check running state and executable path | `version` (optional) | Yes |
| `sups_sketchup_control` | Start or terminate the SketchUp process | `action` (`"launch"` / `"kill"`) | No |
