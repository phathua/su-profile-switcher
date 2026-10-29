use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileMetadata {
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub shortcut: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionConfig {
    #[serde(rename = "activeProfile", default = "default_profile")]
    pub active_profile: String,

    #[serde(rename = "profiles", default)]
    pub profiles: Vec<String>,

    #[serde(rename = "recentProfiles", default)]
    pub recent_profiles: Vec<String>,

    #[serde(rename = "customExePath", default)]
    pub custom_exe_path: Option<String>,

    #[serde(rename = "profilesMetadata", default)]
    pub profiles_metadata: HashMap<String, ProfileMetadata>,
}

fn default_profile() -> String {
    "Default".to_string()
}

impl Default for VersionConfig {
    fn default() -> Self {
        Self {
            active_profile: "Default".to_string(),
            profiles: vec!["Default".to_string()],
            recent_profiles: vec!["Default".to_string()],
            custom_exe_path: None,
            profiles_metadata: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    #[serde(rename = "selectedVersion", default)]
    pub selected_version: String,

    #[serde(rename = "language", default = "default_lang")]
    pub language: String,

    #[serde(rename = "theme", default = "default_theme")]
    pub theme: String,

    #[serde(rename = "autoCheckUpdates", default = "default_true")]
    pub auto_check_updates: bool,

    #[serde(rename = "ignoredUpdateVersion", default)]
    pub ignored_update_version: String,

    #[serde(rename = "remindLaterUntil", default)]
    pub remind_later_until: Option<String>,

    #[serde(rename = "excludeRenderAndDefaultPlugins", default = "default_true")]
    pub exclude_render_and_default_plugins: bool,

    #[serde(rename = "remapUserPaths", default = "default_true")]
    pub remap_user_paths: bool,

    #[serde(rename = "versions", default)]
    pub versions: HashMap<String, VersionConfig>,
}

fn default_lang() -> String {
    "vi".to_string()
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            selected_version: String::new(),
            language: "vi".to_string(),
            theme: "dark".to_string(),
            auto_check_updates: true,
            ignored_update_version: String::new(),
            remind_later_until: None,
            exclude_render_and_default_plugins: true,
            remap_user_paths: true,
            versions: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileComponents {
    pub name: String,
    #[serde(rename = "hasPlugins")]
    pub has_plugins: bool,
    #[serde(rename = "hasShortcuts")]
    pub has_shortcuts: bool,
    #[serde(rename = "hasToolbars")]
    pub has_toolbars: bool,
    #[serde(rename = "hasMaterials")]
    pub has_materials: bool,
    #[serde(rename = "hasTemplates")]
    pub has_templates: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportAnalysis {
    #[serde(rename = "sourcePath")]
    pub source_path: String,
    #[serde(rename = "detectedVersion")]
    pub detected_version: Option<String>,
    #[serde(rename = "hasPlugins")]
    pub has_plugins: bool,
    #[serde(rename = "hasShortcuts")]
    pub has_shortcuts: bool,
    #[serde(rename = "hasToolbars")]
    pub has_toolbars: bool,
    #[serde(rename = "hasMaterials")]
    pub has_materials: bool,
    #[serde(rename = "hasTemplates")]
    pub has_templates: bool,
    #[serde(rename = "pluginCount")]
    pub plugin_count: usize,
    #[serde(rename = "suggestedProfileName")]
    pub suggested_profile_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    #[serde(rename = "hasUpdate")]
    pub has_update: bool,

    #[serde(rename = "currentVersion")]
    pub current_version: String,

    #[serde(rename = "latestVersion")]
    pub latest_version: String,

    #[serde(rename = "releaseNotes")]
    pub release_notes: String,

    #[serde(rename = "downloadUrl")]
    pub download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileItem {
    pub name: String,
    #[serde(rename = "isActive")]
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginItem {
    pub name: String,
    pub filename: String,
    #[serde(rename = "isExcluded")]
    pub is_excluded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutItem {
    pub raw: String,
    #[serde(rename = "keyDisplay")]
    pub key_display: String,
    pub command: String,
    #[serde(rename = "isCustom")]
    pub is_custom: bool,
    #[serde(rename = "originalKey")]
    pub original_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileDetails {
    #[serde(rename = "profileName")]
    pub profile_name: String,
    pub plugins: Vec<PluginItem>,
    #[serde(rename = "customShortcuts")]
    pub custom_shortcuts: Vec<ShortcutItem>,
    #[serde(rename = "totalShortcutsCount")]
    pub total_shortcuts_count: usize,
    #[serde(rename = "defaultShortcutsCount")]
    pub default_shortcuts_count: usize,
}
