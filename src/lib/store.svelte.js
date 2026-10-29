import { invoke } from "@tauri-apps/api/core";
import { translations } from "./i18n.js";

export const appStore = $state({
  state: {
    selectedVersion: "",
    language: "vi",
    theme: "dark",
    autoCheckUpdates: true,
    ignoredUpdateVersion: "",
    remindLaterUntil: null,
    excludeRenderAndDefaultPlugins: true,
    remapUserPaths: true,
    versions: {},
  },
  activeTab: "profiles",
  installedVersions: [],
  profileComponents: [], // Array of { name, hasPlugins, hasShortcuts, hasToolbars, hasMaterials, hasTemplates }
  sketchupRunning: false,
  matchingProfile: null,
  updateInfo: null, // { hasUpdate, latestVersion, currentVersion, downloadUrl, releaseNotes }
  isCheckingUpdate: false,
  busy: {
    active: false,
    message: "",
  },
  modal: null, // { type: 'create_profile' | 'rename' | 'save' | 'update' | 'confirm_switch', ... }
  statusMessage: "Sẵn sàng",
  language: "vi",
  theme: "dark",
});

// Translation helper
export function t(key) {
  const lang = appStore.language || "vi";
  const dict = translations[lang] || translations.vi;
  return dict[key] || translations.vi[key] || key;
}

export function setBusy(active, message = "Đang xử lý dữ liệu...") {
  appStore.busy.active = active;
  appStore.busy.message = message;
}

let sysMediaListener = null;

export function applyTheme(themeMode) {
  let isDark = true;
  if (themeMode === "light") {
    isDark = false;
  } else if (themeMode === "system") {
    isDark = window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches;
    if (window.matchMedia && !sysMediaListener) {
      sysMediaListener = window.matchMedia("(prefers-color-scheme: dark)");
      sysMediaListener.addEventListener("change", () => {
        if (appStore.theme === "system") {
          applyTheme("system");
        }
      });
    }
  } else {
    isDark = true;
  }

  if (isDark) {
    document.documentElement.classList.add("dark");
    document.documentElement.setAttribute("data-theme", "dark");
  } else {
    document.documentElement.classList.remove("dark");
    document.documentElement.setAttribute("data-theme", "light");
  }
}

export async function setTheme(newTheme) {
  appStore.theme = newTheme;
  applyTheme(newTheme);
  try {
    await invoke("update_settings", {
      autoCheckUpdates: appStore.state.autoCheckUpdates,
      excludeRender: appStore.state.excludeRenderAndDefaultPlugins,
      remapUserPaths: appStore.state.remapUserPaths ?? true,
      language: appStore.language,
      theme: newTheme,
      ignoredUpdateVersion: null,
      remindLater: false,
    });
  } catch (e) {
    console.error("Set theme error:", e);
  }
}

export async function setLanguage(newLang) {
  appStore.language = newLang;
  try {
    await invoke("update_settings", {
      autoCheckUpdates: appStore.state.autoCheckUpdates,
      excludeRender: appStore.state.excludeRenderAndDefaultPlugins,
      remapUserPaths: appStore.state.remapUserPaths ?? true,
      language: newLang,
      theme: appStore.theme,
      ignoredUpdateVersion: null,
      remindLater: false,
    });
  } catch (e) {
    console.error("Set language error:", e);
  }
}

export async function initAppStore() {
  try {
    const installed = await invoke("get_installed_versions");
    appStore.installedVersions = installed;

    const s = await invoke("get_app_state");
    appStore.state = s;
    if (s.language) appStore.language = s.language;
    if (s.theme) appStore.theme = s.theme;

    applyTheme(appStore.theme);

    if (!appStore.state.selectedVersion && installed.length > 0) {
      await selectVersion(installed[0]);
    } else {
      await loadProfileComponents();
    }

    await checkProcessStatus();
    await checkHashMatch();

    if (appStore.state.autoCheckUpdates) {
      checkForUpdates(true);
    }
  } catch (e) {
    console.error("Init error:", e);
  }
}

export async function loadProfileComponents() {
  const ver = appStore.state.selectedVersion;
  if (!ver) return;
  try {
    const comps = await invoke("get_profile_components", { version: ver });
    appStore.profileComponents = comps;
  } catch (e) {
    console.error("Load components error:", e);
  }
}

export async function selectVersion(version) {
  setBusy(true, `Đang chuyển sang ${version}...`);
  try {
    const s = await invoke("select_version", { version });
    appStore.state = s;
    await loadProfileComponents();
    await checkHashMatch();
  } catch (e) {
    console.error("Select version error:", e);
  } finally {
    setBusy(false);
  }
}

export async function checkProcessStatus() {
  try {
    const running = await invoke("is_sketchup_running");
    appStore.sketchupRunning = running;
  } catch (e) {
    console.error("Process status error:", e);
  }
}

export async function checkHashMatch() {
  try {
    const ver = appStore.state.selectedVersion;
    if (ver) {
      const match = await invoke("check_matching_hash", { version: ver });
      appStore.matchingProfile = match;
    }
  } catch (e) {
    console.error("Hash match error:", e);
  }
}

export async function switchProfile(targetProfile) {
  const ver = appStore.state.selectedVersion;
  if (!ver) return;
  setBusy(true, `Đang chuyển sang hồ sơ '${targetProfile}'...`);
  try {
    appStore.statusMessage = t("statusRunning");
    const s = await invoke("switch_profile", {
      version: ver,
      targetProfile,
    });
    appStore.state = s;
    await loadProfileComponents();
    await checkHashMatch();
    await checkProcessStatus();
  } catch (e) {
    alert(String(e));
  } finally {
    setBusy(false);
  }
}

export async function saveProfile(name, savePlugins, saveShortcuts) {
  const ver = appStore.state.selectedVersion;
  if (!ver) return;
  setBusy(true, `Đang lưu hồ sơ '${name}'...`);
  try {
    const s = await invoke("save_profile", {
      version: ver,
      name,
      savePlugins,
      saveShortcuts,
    });
    appStore.state = s;
    await loadProfileComponents();
    await checkHashMatch();
  } catch (e) {
    alert(String(e));
  } finally {
    setBusy(false);
  }
}

export async function saveProfileCustom(
  name,
  savePlugins,
  saveShortcuts,
  saveToolbars,
  saveMaterials,
  saveTemplates,
  icon = null,
  color = null
) {
  let ver = appStore.state.selectedVersion;
  if (!ver) {
    const s = await invoke("get_app_state");
    appStore.state = s;
    ver = s.selectedVersion;
  }
  if (!ver) return;
  setBusy(true, `Đang lưu hồ sơ '${name}'...`);
  try {
    const s = await invoke("save_profile_custom", {
      version: ver,
      name,
      savePlugins,
      saveShortcuts,
      saveToolbars,
      saveMaterials,
      saveTemplates,
      icon,
      color,
    });
    appStore.state = s;
    await loadProfileComponents();
    await checkHashMatch();
  } catch (e) {
    alert(String(e));
  } finally {
    setBusy(false);
  }
}

export async function getProfileDetails(profileName) {
  let ver = appStore.state.selectedVersion;
  if (!ver) {
    const s = await invoke("get_app_state");
    appStore.state = s;
    ver = s.selectedVersion;
  }
  if (!ver) return null;
  try {
    return await invoke("get_profile_details", {
      version: ver,
      profileName,
    });
  } catch (e) {
    console.error("Get profile details error:", e);
    return null;
  }
}

export async function renameProfile(oldName, newName) {
  setBusy(true, `Đang đổi tên hồ sơ...`);
  try {
    const ver = appStore.state.selectedVersion;
    const s = await invoke("rename_profile", {
      version: ver,
      oldName,
      newName,
    });
    appStore.state = s;
    await loadProfileComponents();
  } catch (e) {
    alert(String(e));
  } finally {
    setBusy(false);
  }
}

export async function updateProfileMetadata(profile, icon, color, shortcut) {
  setBusy(true, "Đang cập nhật hồ sơ...");
  try {
    const ver = appStore.state.selectedVersion;
    const s = await invoke("update_profile_metadata", {
      version: ver,
      profile,
      icon,
      color,
      shortcut,
    });
    appStore.state = s;
  } catch (e) {
    alert(String(e));
  } finally {
    setBusy(false);
  }
}

export async function deleteProfile(name) {
  setBusy(true, `Đang xóa hồ sơ '${name}'...`);
  try {
    const ver = appStore.state.selectedVersion;
    const s = await invoke("delete_profile", {
      version: ver,
      name,
    });
    appStore.state = s;
    await loadProfileComponents();
    await checkHashMatch();
  } catch (e) {
    alert(String(e));
  } finally {
    setBusy(false);
  }
}

export async function checkForUpdates(silent = false) {
  appStore.isCheckingUpdate = true;
  try {
    const info = await invoke("check_for_updates", { simulate: null });
    appStore.updateInfo = info;
    if (info.hasUpdate) {
      // Found update
      return info;
    } else if (!silent) {
      alert(t("alreadyLatest"));
    }
  } catch (e) {
    if (!silent) alert(String(e));
  } finally {
    appStore.isCheckingUpdate = false;
  }
  return null;
}

export async function applyUpdateAndRestart() {
  if (!appStore.updateInfo || !appStore.updateInfo.hasUpdate) return;
  setBusy(true, "Đang tải bản cập nhật và tự khởi động lại...");
  try {
    await invoke("apply_update_and_restart", {
      downloadUrl: appStore.updateInfo.downloadUrl || null,
      latestVersion: appStore.updateInfo.latestVersion,
    });
  } catch (e) {
    alert("Lỗi cập nhật: " + String(e));
    setBusy(false);
  }
}
