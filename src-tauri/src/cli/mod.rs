use serde_json::json;
use crate::{engine, hash, mcp, process};

pub fn is_cli_invocation(args: &[String]) -> bool {
    if args.len() < 2 {
        return false;
    }
    let sub = args[1].to_lowercase();
    matches!(
        sub.as_str(),
        "list"
            | "current"
            | "switch"
            | "save"
            | "delete"
            | "inspect"
            | "sketchup"
            | "mcp"
            | "--mcp"
            | "help"
            | "--help"
            | "-h"
            | "version"
            | "--version"
            | "-v"
            | "--switch"
            | "--save-profile"
            | "--list-profiles"
            | "--hash-check"
            | "--check-update"
            | "--simulate-update"
            | "--test-toast"
    )
}

pub fn run_cli(args: &[String]) -> i32 {
    if args.len() < 2 {
        print_help();
        return 0;
    }

    let subcmd = args[1].as_str();

    match subcmd {
        "mcp" | "--mcp" => {
            mcp::run_mcp_server();
            0
        }
        "help" | "--help" | "-h" => {
            print_help();
            0
        }
        "version" | "--version" | "-V" => {
            println!("sups {}", env!("CARGO_PKG_VERSION"));
            0
        }
        "list" => handle_list(&args[2..]),
        "current" => handle_current(&args[2..]),
        "switch" => handle_switch(&args[2..]),
        "save" => handle_save(&args[2..]),
        "delete" => handle_delete(&args[2..]),
        "inspect" => handle_inspect(&args[2..]),
        "sketchup" => handle_sketchup(&args[2..]),
        _ => {
            // Backward compatibility for legacy flags e.g. --switch, --list-profiles
            if subcmd.starts_with("--") {
                handle_legacy_flags(args)
            } else {
                eprintln!("[ERROR] Unknown command '{}'. Run 'sups --help' for usage.", subcmd);
                1
            }
        }
    }
}

fn print_help() {
    println!("SketchUp Profile Switcher CLI (sups) v{}", env!("CARGO_PKG_VERSION"));
    println!("Fast, lightweight SketchUp multi-profile manager & MCP bridge\n");
    println!("USAGE:");
    println!("  sups <COMMAND> [OPTIONS]\n");
    println!("COMMANDS:");
    println!("  list                         List all profiles for a SketchUp version");
    println!("  current                      Show the active profile, configuration hash, and match status");
    println!("  switch <PROFILE>             Switch to the specified profile");
    println!("  save <PROFILE>               Save active configuration as a new or existing profile");
    println!("  delete <PROFILE>             Delete a saved profile");
    println!("  inspect <PROFILE>            Inspect plugins and shortcut diffs in a profile");
    println!("  sketchup <status|launch|kill> Manage SketchUp process");
    println!("  mcp                          Run Model Context Protocol (MCP) server over stdio");
    println!("  help, --help, -h             Show this help information");
    println!("  version, --version, -V       Show version information\n");
    println!("OPTIONS:");
    println!("  --version <VERSION>          Target SketchUp version (e.g. 'SketchUp 2026')");
    println!("  --json                       Output result in JSON format for automated tooling");
    println!("  --force                      Force kill SketchUp if running before switching");
    println!("  --no-plugins                 Exclude plugins when saving profile");
    println!("  --no-shortcuts               Exclude shortcuts when saving profile\n");
    println!("EXAMPLES:");
    println!("  sups list --version \"SketchUp 2026\"");
    println!("  sups switch \"Production V-Ray\" --force");
    println!("  sups save \"Backup_2026\" --no-shortcuts");
    println!("  sups sketchup status --json");
    println!("  sups mcp");
}

fn parse_flag_value(args: &[String], flag: &str) -> Option<String> {
    for i in 0..args.len() {
        if args[i].eq_ignore_ascii_case(flag) && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
    }
    None
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| a.eq_ignore_ascii_case(flag))
}

fn resolve_version(args: &[String], state: &crate::models::AppState) -> String {
    if let Some(v) = parse_flag_value(args, "--version") {
        if !v.trim().is_empty() {
            return v.trim().to_string();
        }
    }
    if !state.selected_version.is_empty() {
        return state.selected_version.clone();
    }
    engine::detect_installed_versions()
        .first()
        .cloned()
        .unwrap_or_else(|| "SketchUp 2026".to_string())
}

fn handle_list(args: &[String]) -> i32 {
    let mut state = engine::load_state();
    let ver = resolve_version(args, &state);
    let as_json = has_flag(args, "--json");

    engine::ensure_initialized(&ver, &mut state);

    if let Some(cfg) = state.versions.get(&ver) {
        if as_json {
            let out = json!({
                "version": ver,
                "active_profile": cfg.active_profile,
                "profiles": cfg.profiles,
                "recent_profiles": cfg.recent_profiles,
                "metadata": cfg.profiles_metadata
            });
            println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
        } else {
            println!("SketchUp Version: {}", ver);
            println!("Active Profile:   {}", cfg.active_profile);
            println!("Available Profiles ({}):", cfg.profiles.len());
            for p in &cfg.profiles {
                let mark = if p == &cfg.active_profile { "* " } else { "  " };
                println!("  {}{}", mark, p);
            }
            if !cfg.recent_profiles.is_empty() {
                println!("\nRecent: {}", cfg.recent_profiles.join(", "));
            }
        }
        0
    } else {
        eprintln!("[ERROR] Version '{}' not found in configurations.", ver);
        1
    }
}

fn handle_current(args: &[String]) -> i32 {
    let mut state = engine::load_state();
    let ver = resolve_version(args, &state);
    let as_json = has_flag(args, "--json");

    engine::ensure_initialized(&ver, &mut state);

    let roaming = engine::get_roaming_sketchup_path(&ver);
    let local = engine::get_local_sketchup_path(&ver);
    let profiles_dir = engine::get_profiles_dir(&ver);

    let active_hash = hash::compute_configuration_hash(
        &roaming,
        Some(&local),
        state.exclude_render_and_default_plugins,
    );
    let matching = hash::find_matching_profile(
        &roaming,
        &local,
        &profiles_dir,
        state.exclude_render_and_default_plugins,
    );

    let active_profile = state
        .versions
        .get(&ver)
        .map(|v| v.active_profile.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    if as_json {
        let out = json!({
            "version": ver,
            "active_profile_recorded": active_profile,
            "configuration_hash": active_hash,
            "matching_profile_on_disk": matching,
            "render_protection": state.exclude_render_and_default_plugins
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
    } else {
        println!("SketchUp Version:      {}", ver);
        println!("Active Profile Name:   {}", active_profile);
        println!("Configuration SHA-256: {}", active_hash);
        println!("Matching Saved Profile: {}", matching.as_deref().unwrap_or("None (Modified)"));
        println!("Render Protection:     {}", if state.exclude_render_and_default_plugins { "Enabled" } else { "Disabled" });
    }
    0
}

fn handle_switch(args: &[String]) -> i32 {
    if args.is_empty() || args[0].starts_with("--") {
        eprintln!("[ERROR] Missing target profile name. Usage: sups switch <PROFILE> [OPTIONS]");
        return 1;
    }

    let target = &args[0];
    let mut state = engine::load_state();
    let ver = resolve_version(args, &state);
    let force = has_flag(args, "--force");

    if process::is_sketchup_running() {
        if force {
            println!("[INFO] SketchUp is running. Force closing before switch...");
            if !process::close_sketchup_gracefully() {
                eprintln!("[ERROR] Could not terminate SketchUp. Aborting switch.");
                return 1;
            }
        } else {
            eprintln!("[ERROR] SketchUp is currently running. Close SketchUp or use '--force' to close automatically.");
            return 1;
        }
    }

    match engine::switch_profile(&ver, target, &mut state) {
        Ok(_) => {
            println!("[SUCCESS] Switched to profile '{}' for {}.", target, ver);
            0
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to switch profile: {}", e);
            1
        }
    }
}

fn handle_save(args: &[String]) -> i32 {
    if args.is_empty() || args[0].starts_with("--") {
        eprintln!("[ERROR] Missing profile name. Usage: sups save <PROFILE> [OPTIONS]");
        return 1;
    }

    let profile_name = &args[0];
    let mut state = engine::load_state();
    let ver = resolve_version(args, &state);

    let save_plugins = !has_flag(args, "--no-plugins");
    let save_shortcuts = !has_flag(args, "--no-shortcuts");
    let save_toolbars = has_flag(args, "--toolbars");
    let save_materials = has_flag(args, "--materials");
    let save_templates = has_flag(args, "--templates");

    match engine::save_profile_custom(
        &ver,
        profile_name,
        save_plugins,
        save_shortcuts,
        save_toolbars,
        save_materials,
        save_templates,
        None,
        None,
        &mut state,
    ) {
        Ok(_) => {
            println!("[SUCCESS] Saved profile '{}' for {}.", profile_name, ver);
            println!("  Plugins:   {}", save_plugins);
            println!("  Shortcuts: {}", save_shortcuts);
            println!("  Toolbars:  {}", save_toolbars);
            println!("  Materials: {}", save_materials);
            println!("  Templates: {}", save_templates);
            0
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to save profile: {}", e);
            1
        }
    }
}

fn handle_delete(args: &[String]) -> i32 {
    if args.is_empty() || args[0].starts_with("--") {
        eprintln!("[ERROR] Missing profile name. Usage: sups delete <PROFILE> [OPTIONS]");
        return 1;
    }

    let profile_name = &args[0];
    let mut state = engine::load_state();
    let ver = resolve_version(args, &state);

    match engine::delete_profile(&ver, profile_name, &mut state) {
        Ok(_) => {
            println!("[SUCCESS] Deleted profile '{}' for {}.", profile_name, ver);
            0
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to delete profile: {}", e);
            1
        }
    }
}

fn handle_inspect(args: &[String]) -> i32 {
    if args.is_empty() || args[0].starts_with("--") {
        eprintln!("[ERROR] Missing profile name. Usage: sups inspect <PROFILE> [OPTIONS]");
        return 1;
    }

    let profile_name = &args[0];
    let state = engine::load_state();
    let ver = resolve_version(args, &state);
    let as_json = has_flag(args, "--json");

    match engine::get_profile_details(&ver, profile_name, &state) {
        Ok(details) => {
            if as_json {
                println!("{}", serde_json::to_string_pretty(&details).unwrap_or_default());
            } else {
                println!("Inspection for Profile '{}' ({}):", profile_name, ver);
                println!("Plugins ({}) :", details.plugins.len());
                for p in &details.plugins {
                    let status = if p.is_excluded { "[Excluded]" } else { "[Included]" };
                    println!("  {} {} ({})", status, p.name, p.filename);
                }
                println!("\nCustom Shortcuts ({}) :", details.custom_shortcuts.len());
                for s in &details.custom_shortcuts {
                    let orig = s.original_key.as_deref().unwrap_or("None");
                    println!("  {} => {} (Original: {})", s.key_display, s.command, orig);
                }
                println!("\nTotal Shortcuts:   {}", details.total_shortcuts_count);
                println!("Default Shortcuts: {}", details.default_shortcuts_count);
            }
            0
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to inspect profile '{}': {}", profile_name, e);
            1
        }
    }
}

fn handle_sketchup(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("[ERROR] Missing sketchup action. Usage: sups sketchup <status|launch|kill> [OPTIONS]");
        return 1;
    }

    let action = args[0].to_lowercase();
    let state = engine::load_state();
    let ver = resolve_version(args, &state);
    let as_json = has_flag(args, "--json");

    match action.as_str() {
        "status" => {
            let is_running = process::is_sketchup_running();
            let exe_path = process::find_sketchup_exe(&ver).map(|p| p.to_string_lossy().to_string());

            if as_json {
                let out = json!({
                    "version": ver,
                    "is_running": is_running,
                    "executable_path": exe_path
                });
                println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
            } else {
                println!("SketchUp Status ({}):", ver);
                println!("  Running:         {}", if is_running { "YES" } else { "NO" });
                println!("  Executable Path: {}", exe_path.unwrap_or_else(|| "Not found".to_string()));
            }
            0
        }
        "launch" => {
            if process::launch_sketchup(&ver, None) {
                println!("[SUCCESS] SketchUp ({}) launched.", ver);
                0
            } else {
                eprintln!("[ERROR] Failed to launch SketchUp (executable not found for {}).", ver);
                1
            }
        }
        "kill" => {
            if process::close_sketchup_gracefully() {
                println!("[SUCCESS] SketchUp terminated.");
                0
            } else {
                println!("[INFO] No running SketchUp process found or terminated.");
                0
            }
        }
        _ => {
            eprintln!("[ERROR] Unknown action '{}'. Supported actions: status, launch, kill.", action);
            1
        }
    }
}

fn handle_legacy_flags(args: &[String]) -> i32 {
    let mut switch_profile: Option<String> = None;
    let mut save_profile: Option<String> = None;
    let mut target_version: Option<String> = None;
    let mut list_profiles = false;
    let mut hash_check = false;
    let mut save_plugins = true;
    let mut save_shortcuts = true;

    let mut i = 1;
    while i < args.len() {
        let arg = args[i].as_str();
        if arg.eq_ignore_ascii_case("--switch") && i + 1 < args.len() {
            switch_profile = Some(args[i + 1].clone());
            i += 2;
        } else if arg.eq_ignore_ascii_case("--save-profile") && i + 1 < args.len() {
            save_profile = Some(args[i + 1].clone());
            i += 2;
        } else if arg.eq_ignore_ascii_case("--version") && i + 1 < args.len() {
            target_version = Some(args[i + 1].clone());
            i += 2;
        } else if arg.eq_ignore_ascii_case("--hash-check") {
            hash_check = true;
            i += 1;
        } else if arg.eq_ignore_ascii_case("--list-profiles") {
            list_profiles = true;
            i += 1;
        } else if arg.eq_ignore_ascii_case("--plugins") && i + 1 < args.len() {
            save_plugins = args[i + 1].parse().unwrap_or(true);
            i += 2;
        } else if arg.eq_ignore_ascii_case("--shortcuts") && i + 1 < args.len() {
            save_shortcuts = args[i + 1].parse().unwrap_or(true);
            i += 2;
        } else {
            i += 1;
        }
    }

    let mut state = engine::load_state();
    let ver = target_version.unwrap_or_else(|| {
        if !state.selected_version.is_empty() {
            state.selected_version.clone()
        } else {
            engine::detect_installed_versions().first().cloned().unwrap_or_else(|| "SketchUp 2026".to_string())
        }
    });

    engine::ensure_initialized(&ver, &mut state);

    if hash_check {
        let roaming = engine::get_roaming_sketchup_path(&ver);
        let local = engine::get_local_sketchup_path(&ver);
        let profiles_dir = engine::get_profiles_dir(&ver);
        let active_hash = hash::compute_configuration_hash(&roaming, Some(&local), state.exclude_render_and_default_plugins);
        let matching = hash::find_matching_profile(&roaming, &local, &profiles_dir, state.exclude_render_and_default_plugins);
        println!("[HASH_CHECK] Version: {}, ActiveHash: {}, MatchingProfile: {}", ver, active_hash, matching.as_deref().unwrap_or("None"));
    }

    if let Some(save_name) = save_profile {
        match engine::save_current_profile_as(&ver, &save_name, save_plugins, save_shortcuts, &mut state) {
            Ok(_) => println!("[SAVE_PROFILE] Saved '{}' for {} (Plugins: {}, Shortcuts: {})", save_name, ver, save_plugins, save_shortcuts),
            Err(e) => {
                eprintln!("[ERROR] {}", e);
                return 1;
            }
        }
    }

    if let Some(target) = switch_profile {
        match engine::switch_profile(&ver, &target, &mut state) {
            Ok(_) => println!("[SWITCH_PROFILE] Switched to '{}' for {}", target, ver),
            Err(e) => {
                eprintln!("[ERROR] {}", e);
                return 1;
            }
        }
    }

    if list_profiles {
        if let Some(cfg) = state.versions.get(&ver) {
            println!("[LIST_PROFILES] Version: {}, Active: {}", ver, cfg.active_profile);
            println!("[ALL_PROFILES] {}", cfg.profiles.join(", "));
        }
    }

    0
}
