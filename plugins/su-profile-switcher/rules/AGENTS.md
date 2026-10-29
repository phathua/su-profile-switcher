# SketchUp Profile Switcher Safety & Automation Rules

## 1. Process Guardrails & Safe Switching
- Never switch a profile while SketchUp is active without explicit confirmation or setting `force_kill_sketchup: true`.
- Always check `sups_sketchup_status` before attempting profile changes.
- If SketchUp is running, inform the user and request confirmation to close before switching.

## 2. Extension & Render Engine Preservation
- Do not modify or remove render plugin directories (V-Ray, Enscape, D5, Thea) located inside the Plugins folder.
- Ensure render engine protection remains enabled when switching or saving profiles to prevent licensing invalidation.

## 3. Configuration Integrity
- When diagnosing extension conflicts, always run `sups_get_current_profile` to verify whether the on-disk files match the recorded profile hash.
- Take a backup profile snapshot via `sups_save_profile` before testing new or experimental Ruby plugins.
