use std::io::{self, BufRead, Write};
use serde_json::{json, Value};
use crate::{engine, hash, process};

pub fn run_mcp_server() {
    eprintln!("[MCP] SketchUp Profile Switcher MCP Server starting over stdio...");
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[MCP] Error reading line from stdin: {}", e);
                break;
            }
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let parsed: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[MCP] Invalid JSON received: {}", e);
                let err_resp = json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": {
                        "code": -32700,
                        "message": "Parse error"
                    }
                });
                let _ = writeln!(stdout, "{}", err_resp);
                let _ = stdout.flush();
                continue;
            }
        };

        if let Some(resp) = handle_mcp_message(&parsed) {
            let resp_str = serde_json::to_string(&resp).unwrap_or_default();
            let _ = writeln!(stdout, "{}", resp_str);
            let _ = stdout.flush();
        }
    }

    eprintln!("[MCP] Stdio pipe closed. MCP Server shutting down.");
}

fn handle_mcp_message(msg: &Value) -> Option<Value> {
    let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let id = msg.get("id").cloned();

    match method {
        "initialize" => {
            let resp = json!({
                "jsonrpc": "2.0",
                "id": id.unwrap_or(Value::Null),
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {
                            "listChanged": false
                        }
                    },
                    "serverInfo": {
                        "name": "su-profile-switcher-mcp",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }
            });
            Some(resp)
        }
        "notifications/initialized" => {
            eprintln!("[MCP] Client initialized successfully.");
            None
        }
        "ping" => {
            let resp = json!({
                "jsonrpc": "2.0",
                "id": id.unwrap_or(Value::Null),
                "result": {}
            });
            Some(resp)
        }
        "tools/list" => {
            let tools = get_tool_definitions();
            let resp = json!({
                "jsonrpc": "2.0",
                "id": id.unwrap_or(Value::Null),
                "result": {
                    "tools": tools
                }
            });
            Some(resp)
        }
        "tools/call" => {
            let params = msg.get("params");
            let tool_name = params
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("");
            let arguments = params
                .and_then(|p| p.get("arguments"))
                .cloned()
                .unwrap_or_else(|| json!({}));

            let (result_text, is_error) = execute_tool(tool_name, &arguments);

            let resp = json!({
                "jsonrpc": "2.0",
                "id": id.unwrap_or(Value::Null),
                "result": {
                    "content": [
                        {
                            "type": "text",
                            "text": result_text
                        }
                    ],
                    "isError": is_error
                }
            });
            Some(resp)
        }
        _ => {
            if id.is_some() {
                let err_resp = json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32601,
                        "message": format!("Method not found: {}", method)
                    }
                });
                Some(err_resp)
            } else {
                None
            }
        }
    }
}

fn get_tool_definitions() -> Value {
    json!([
        {
            "name": "sups_list_versions",
            "description": "List all installed SketchUp versions detected on Windows (e.g. SketchUp 2021 to 2026) and identify the currently selected active version.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        },
        {
            "name": "sups_list_profiles",
            "description": "List all saved configuration profiles for a given SketchUp version, including the currently active profile and recent switch history.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "version": {
                        "type": "string",
                        "description": "Target SketchUp version (e.g. 'SketchUp 2026'). If omitted, defaults to the currently selected or newest detected version."
                    }
                },
                "required": []
            }
        },
        {
            "name": "sups_get_current_profile",
            "description": "Get current active profile name, calculate SHA-256 configuration hash of active AppData, and find matching saved profile on disk.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "version": {
                        "type": "string",
                        "description": "Target SketchUp version (e.g. 'SketchUp 2026')."
                    }
                },
                "required": []
            }
        },
        {
            "name": "sups_switch_profile",
            "description": "Safely switch SketchUp configuration to a target profile. If SketchUp is running and force_kill_sketchup is false, switch is aborted safely.",
            "inputSchema": {
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
        },
        {
            "name": "sups_save_profile",
            "description": "Save current active SketchUp configuration as a new profile or overwrite an existing profile with customizable component flags.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "profile_name": {
                        "type": "string",
                        "description": "Name for the profile."
                    },
                    "version": {
                        "type": "string",
                        "description": "Target SketchUp version (e.g. 'SketchUp 2026')."
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
        },
        {
            "name": "sups_inspect_profile",
            "description": "Inspect the contents of a profile: list all installed plugins/extensions with file names and excluded status, plus all custom shortcut overrides compared against SketchUp 2026 defaults.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "profile_name": {
                        "type": "string",
                        "description": "Name of the profile to inspect."
                    },
                    "version": {
                        "type": "string",
                        "description": "Target SketchUp version (e.g. 'SketchUp 2026')."
                    }
                },
                "required": ["profile_name"]
            }
        },
        {
            "name": "sups_sketchup_status",
            "description": "Check if SketchUp is currently running on Windows, and detect the full executable path for the specified version.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "version": {
                        "type": "string",
                        "description": "Target SketchUp version (e.g. 'SketchUp 2026')."
                    }
                },
                "required": []
            }
        },
        {
            "name": "sups_sketchup_control",
            "description": "Launch or terminate the SketchUp application safely.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["launch", "kill"],
                        "description": "Action to perform: 'launch' to open SketchUp, 'kill' to gracefully close running instances."
                    },
                    "version": {
                        "type": "string",
                        "description": "Target SketchUp version (e.g. 'SketchUp 2026'). Used when action is 'launch'."
                    }
                },
                "required": ["action"]
            }
        }
    ])
}

fn resolve_version(version_arg: Option<&str>, state: &crate::models::AppState) -> String {
    if let Some(v) = version_arg {
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

fn execute_tool(name: &str, args: &Value) -> (String, bool) {
    let mut state = engine::load_state();

    match name {
        "sups_list_versions" => {
            let installed = engine::detect_installed_versions();
            let selected = if !state.selected_version.is_empty() {
                state.selected_version.clone()
            } else {
                installed.first().cloned().unwrap_or_default()
            };
            let result = json!({
                "installed_versions": installed,
                "selected_version": selected
            });
            (serde_json::to_string_pretty(&result).unwrap_or_default(), false)
        }
        "sups_list_profiles" => {
            let ver_arg = args.get("version").and_then(|v| v.as_str());
            let ver = resolve_version(ver_arg, &state);
            engine::ensure_initialized(&ver, &mut state);

            if let Some(cfg) = state.versions.get(&ver) {
                let result = json!({
                    "version": ver,
                    "active_profile": cfg.active_profile,
                    "profiles": cfg.profiles,
                    "recent_profiles": cfg.recent_profiles,
                    "metadata": cfg.profiles_metadata
                });
                (serde_json::to_string_pretty(&result).unwrap_or_default(), false)
            } else {
                (format!("Version configuration for '{}' not found.", ver), true)
            }
        }
        "sups_get_current_profile" => {
            let ver_arg = args.get("version").and_then(|v| v.as_str());
            let ver = resolve_version(ver_arg, &state);
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

            let result = json!({
                "version": ver,
                "active_profile_recorded": active_profile,
                "configuration_hash": active_hash,
                "matching_profile_on_disk": matching,
                "is_render_protection_enabled": state.exclude_render_and_default_plugins
            });
            (serde_json::to_string_pretty(&result).unwrap_or_default(), false)
        }
        "sups_switch_profile" => {
            let profile_name = match args.get("profile_name").and_then(|p| p.as_str()) {
                Some(p) if !p.trim().is_empty() => p.trim(),
                _ => return ("Missing required parameter: profile_name".to_string(), true),
            };

            let ver_arg = args.get("version").and_then(|v| v.as_str());
            let ver = resolve_version(ver_arg, &state);
            let force = args
                .get("force_kill_sketchup")
                .and_then(|f| f.as_bool())
                .unwrap_or(false);

            if process::is_sketchup_running() {
                if force {
                    eprintln!("[MCP] SketchUp is running. Force closing before switch...");
                    let closed = process::close_sketchup_gracefully();
                    if !closed {
                        return (
                            "SketchUp is currently running and could not be terminated. Switch aborted."
                                .to_string(),
                            true,
                        );
                    }
                } else {
                    return (
                        "SketchUp is currently running. Please close SketchUp first, or pass 'force_kill_sketchup: true' to close it automatically."
                            .to_string(),
                        true,
                    );
                }
            }

            match engine::switch_profile(&ver, profile_name, &mut state) {
                Ok(_) => {
                    let result = json!({
                        "success": true,
                        "switched_to": profile_name,
                        "version": ver,
                        "message": format!("Successfully switched to profile '{}' for {}.", profile_name, ver)
                    });
                    (serde_json::to_string_pretty(&result).unwrap_or_default(), false)
                }
                Err(e) => (format!("Failed to switch profile: {}", e), true),
            }
        }
        "sups_save_profile" => {
            let profile_name = match args.get("profile_name").and_then(|p| p.as_str()) {
                Some(p) if !p.trim().is_empty() => p.trim(),
                _ => return ("Missing required parameter: profile_name".to_string(), true),
            };

            let ver_arg = args.get("version").and_then(|v| v.as_str());
            let ver = resolve_version(ver_arg, &state);

            let save_plugins = args.get("save_plugins").and_then(|v| v.as_bool()).unwrap_or(true);
            let save_shortcuts = args.get("save_shortcuts").and_then(|v| v.as_bool()).unwrap_or(true);
            let save_toolbars = args.get("save_toolbars").and_then(|v| v.as_bool()).unwrap_or(false);
            let save_materials = args.get("save_materials").and_then(|v| v.as_bool()).unwrap_or(false);
            let save_templates = args.get("save_templates").and_then(|v| v.as_bool()).unwrap_or(false);

            let icon = args.get("icon").and_then(|v| v.as_str()).map(|s| s.to_string());
            let color = args.get("color").and_then(|v| v.as_str()).map(|s| s.to_string());

            match engine::save_profile_custom(
                &ver,
                profile_name,
                save_plugins,
                save_shortcuts,
                save_toolbars,
                save_materials,
                save_templates,
                icon,
                color,
                &mut state,
            ) {
                Ok(_) => {
                    let result = json!({
                        "success": true,
                        "saved_profile": profile_name,
                        "version": ver,
                        "components": {
                            "plugins": save_plugins,
                            "shortcuts": save_shortcuts,
                            "toolbars": save_toolbars,
                            "materials": save_materials,
                            "templates": save_templates
                        }
                    });
                    (serde_json::to_string_pretty(&result).unwrap_or_default(), false)
                }
                Err(e) => (format!("Failed to save profile: {}", e), true),
            }
        }
        "sups_inspect_profile" => {
            let profile_name = match args.get("profile_name").and_then(|p| p.as_str()) {
                Some(p) if !p.trim().is_empty() => p.trim(),
                _ => return ("Missing required parameter: profile_name".to_string(), true),
            };

            let ver_arg = args.get("version").and_then(|v| v.as_str());
            let ver = resolve_version(ver_arg, &state);

            match engine::get_profile_details(&ver, profile_name, &state) {
                Ok(details) => (serde_json::to_string_pretty(&details).unwrap_or_default(), false),
                Err(e) => (format!("Failed to inspect profile: {}", e), true),
            }
        }
        "sups_sketchup_status" => {
            let ver_arg = args.get("version").and_then(|v| v.as_str());
            let ver = resolve_version(ver_arg, &state);
            let is_running = process::is_sketchup_running();
            let exe_path = process::find_sketchup_exe(&ver).map(|p| p.to_string_lossy().to_string());

            let result = json!({
                "version": ver,
                "is_running": is_running,
                "executable_path": exe_path
            });
            (serde_json::to_string_pretty(&result).unwrap_or_default(), false)
        }
        "sups_sketchup_control" => {
            let action = match args.get("action").and_then(|a| a.as_str()) {
                Some(a) => a.to_lowercase(),
                None => return ("Missing required parameter: action ('launch' or 'kill')".to_string(), true),
            };

            let ver_arg = args.get("version").and_then(|v| v.as_str());
            let ver = resolve_version(ver_arg, &state);

            if action == "launch" {
                let success = process::launch_sketchup(&ver, None);
                let result = json!({
                    "action": "launch",
                    "version": ver,
                    "success": success,
                    "message": if success { "SketchUp launched successfully." } else { "Failed to launch SketchUp (executable not found)." }
                });
                (serde_json::to_string_pretty(&result).unwrap_or_default(), !success)
            } else if action == "kill" {
                let closed = process::close_sketchup_gracefully();
                let result = json!({
                    "action": "kill",
                    "success": closed,
                    "message": if closed { "SketchUp process terminated successfully." } else { "No SketchUp process found or failed to terminate." }
                });
                (serde_json::to_string_pretty(&result).unwrap_or_default(), !closed)
            } else {
                (format!("Invalid action '{}'. Expected 'launch' or 'kill'.", action), true)
            }
        }
        _ => (format!("Unknown tool: {}", name), true),
    }
}
