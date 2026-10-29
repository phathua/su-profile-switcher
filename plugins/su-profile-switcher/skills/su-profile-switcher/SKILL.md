---
name: su-profile-switcher
description: Manage, inspect, and safely switch SketchUp profiles (plugins, shortcuts, templates, materials) using native MCP tools.
---

# SketchUp Profile Switcher Skill

Use this skill when the user wants to inspect, manage, save, or switch SketchUp profiles, diagnose extension issues, or prepare a specific modeling or rendering workspace.

## Typical Triggers
- "Switch my SketchUp to the V-Ray profile"
- "Backup my current SketchUp plugins and shortcuts"
- "Check which SketchUp profile is active"
- "Inspect plugins in my Default profile"
- "Is SketchUp running right now?"

## Safety Protocol
1. **Always verify process status first**:
   Call `sups_sketchup_status` to see if SketchUp is open.
2. **Handle open SketchUp instances**:
   Never force-switch without permission. Prompt the user: "SketchUp is running. Would you like me to close it safely and proceed with switching to profile X?"
3. **Audit extensions**:
   Use `sups_inspect_profile` to review plugins and detect potential extension conflicts before troubleshooting.
4. **Restore baseline**:
   If an experimental setup crashes SketchUp, use `sups_switch_profile` to revert back to `"Default"` or a known good profile.
