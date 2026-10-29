<script>
  import { appStore, setTheme, setLanguage, t } from "../store.svelte.js";
  import { APP_VERSION } from "../version.js";
  import { invoke } from "@tauri-apps/api/core";
  import CustomCheckbox from "./CustomCheckbox.svelte";
  import {
    GearSix,
    ArrowClockwise,
    ArrowsClockwise,
    ShieldCheck,
    FolderUser,
    Moon,
    Sun,
    Monitor,
    Translate,
  } from "phosphor-svelte";

  let autoCheck = $state(appStore.state.autoCheckUpdates);
  let excludeRender = $state(appStore.state.excludeRenderAndDefaultPlugins);
  let remapUserPaths = $state(appStore.state.remapUserPaths ?? true);
  let checkingUpdate = $state(false);

  $effect(() => {
    autoCheck = appStore.state.autoCheckUpdates;
    excludeRender = appStore.state.excludeRenderAndDefaultPlugins;
    remapUserPaths = appStore.state.remapUserPaths ?? true;
  });

  async function handleToggleAutoCheck(val) {
    autoCheck = val;
    await saveSettings();
  }

  async function handleToggleExcludeRender(val) {
    excludeRender = val;
    await saveSettings();
  }

  async function handleToggleRemapUserPaths(val) {
    remapUserPaths = val;
    await saveSettings();
  }

  async function saveSettings() {
    const s = await invoke("update_settings", {
      autoCheckUpdates: autoCheck,
      excludeRender: excludeRender,
      remapUserPaths: remapUserPaths,
      language: appStore.language,
      theme: appStore.theme,
      ignoredUpdateVersion: null,
      remindLater: false,
    });
    appStore.state = s;
  }

  async function handleCheckUpdateNow() {
    checkingUpdate = true;
    try {
      const info = await invoke("check_for_updates", { simulate: null });
      if (info.hasUpdate) {
        appStore.modal = { type: "update", info };
      } else {
        alert(t("alreadyLatest"));
      }
    } catch (e) {
      alert(String(e));
    } finally {
      checkingUpdate = false;
    }
  }
</script>

<div class="bg-[#ffffff] dark:bg-[#0c0c0e] border border-[#e2e8f0] dark:border-[#1e1e24] rounded-lg p-3 min-h-[265px] flex flex-col justify-between">
  <div class="space-y-2">
    <!-- Header -->
    <div class="flex items-center space-x-2 pb-1.5 border-b border-[#e2e8f0] dark:border-[#1a1a20]">
      <GearSix size={14} weight="bold" class="text-sky-600 dark:text-[#00c8ff]" />
      <span class="text-xs font-bold text-slate-800 dark:text-[#fafafa]">{t("settingsHeading")}</span>
    </div>

    <!-- Spacious Full-Width Settings Rows -->
    <div class="space-y-1.5 pt-0.5">
      <!-- Row 1: Auto Check Updates -->
      <div class="flex items-center justify-between p-2 rounded-lg bg-slate-50 dark:bg-[#121215] border border-slate-200 dark:border-[#1e1e24]">
        <div class="flex items-center space-x-2">
          <ArrowsClockwise size={14} weight="bold" class="text-sky-600 dark:text-[#00c8ff] shrink-0" />
          <span class="text-xs font-semibold text-slate-700 dark:text-[#d4d4d8]">{t("autoCheckUpdate")}</span>
        </div>
        <CustomCheckbox
          bind:checked={autoCheck}
          onchange={handleToggleAutoCheck}
          id="chkAuto"
        />
      </div>

      <!-- Row 2: Protect Render Plugins -->
      <div class="flex items-center justify-between p-2 rounded-lg bg-slate-50 dark:bg-[#121215] border border-slate-200 dark:border-[#1e1e24]">
        <div class="flex items-center space-x-2">
          <ShieldCheck size={14} weight="bold" class="text-sky-600 dark:text-[#00c8ff] shrink-0" />
          <span class="text-xs font-semibold text-slate-700 dark:text-[#d4d4d8]">{t("excludeRender")}</span>
        </div>
        <CustomCheckbox
          bind:checked={excludeRender}
          onchange={handleToggleExcludeRender}
          id="chkExclude"
        />
      </div>

      <!-- Row 3: Auto Remap User Paths -->
      <div class="flex items-center justify-between p-2 rounded-lg bg-slate-50 dark:bg-[#121215] border border-slate-200 dark:border-[#1e1e24]">
        <div class="flex items-center space-x-2">
          <FolderUser size={14} weight="bold" class="text-sky-600 dark:text-[#00c8ff] shrink-0" />
          <div class="flex flex-col">
            <span class="text-xs font-semibold text-slate-700 dark:text-[#d4d4d8]">{t("remapUserPaths")}</span>
            <span class="text-[10px] text-slate-500 dark:text-[#71717a]">{t("remapUserPathsDesc")}</span>
          </div>
        </div>
        <CustomCheckbox
          bind:checked={remapUserPaths}
          onchange={handleToggleRemapUserPaths}
          id="chkRemapPaths"
        />
      </div>

      <!-- Row 4: Theme & Language Split Row -->
      <div class="grid grid-cols-2 gap-2">
        <!-- Theme Controls -->
        <div class="flex items-center justify-between p-2 rounded-lg bg-slate-50 dark:bg-[#121215] border border-slate-200 dark:border-[#1e1e24]">
          <span class="text-xs font-semibold text-slate-700 dark:text-[#d4d4d8]">{t("themeLabel")}</span>
          <div class="flex space-x-0.5 bg-white dark:bg-[#08080a] p-0.5 rounded border border-slate-200 dark:border-[#222228]">
            <button
              onclick={() => setTheme("dark")}
              class="px-2 py-0.5 rounded text-[10px] font-bold flex items-center space-x-1 transition-all cursor-pointer {appStore.theme === 'dark' ? 'bg-sky-600 text-white shadow-xs' : 'text-slate-600 dark:text-[#a1a1aa] hover:text-slate-900 dark:hover:text-white'}"
              title="Giao diện tối"
            >
              <Moon size={11} weight="fill" />
              <span>{t("themeDark")}</span>
            </button>
            <button
              onclick={() => setTheme("light")}
              class="px-2 py-0.5 rounded text-[10px] font-bold flex items-center space-x-1 transition-all cursor-pointer {appStore.theme === 'light' ? 'bg-sky-600 text-white shadow-xs' : 'text-slate-600 dark:text-[#a1a1aa] hover:text-slate-900 dark:hover:text-white'}"
              title="Giao diện sáng"
            >
              <Sun size={11} weight="fill" />
              <span>{t("themeLight")}</span>
            </button>
            <button
              onclick={() => setTheme("system")}
              class="px-2 py-0.5 rounded text-[10px] font-bold flex items-center space-x-1 transition-all cursor-pointer {appStore.theme === 'system' ? 'bg-sky-600 text-white shadow-xs' : 'text-slate-600 dark:text-[#a1a1aa] hover:text-slate-900 dark:hover:text-white'}"
              title="Theo hệ thống"
            >
              <Monitor size={11} weight="fill" />
              <span>{t("themeSystem")}</span>
            </button>
          </div>
        </div>

        <!-- Language Controls -->
        <div class="flex items-center justify-between p-2 rounded-lg bg-slate-50 dark:bg-[#121215] border border-slate-200 dark:border-[#1e1e24]">
          <span class="text-xs font-semibold text-slate-700 dark:text-[#d4d4d8]">{t("langLabel")}</span>
          <div class="flex space-x-1 bg-white dark:bg-[#08080a] p-0.5 rounded border border-slate-200 dark:border-[#222228]">
            <button
              onclick={() => setLanguage("vi")}
              class="px-2 py-0.5 rounded text-[11px] font-bold flex items-center space-x-1.5 transition-all cursor-pointer {appStore.language === 'vi' ? 'bg-sky-600 text-white shadow-xs' : 'text-slate-600 dark:text-[#a1a1aa] hover:text-slate-900 dark:hover:text-white'}"
              title="Tiếng Việt"
            >
              <svg class="w-3.5 h-2.5 rounded-[1px] shrink-0 shadow-xs" viewBox="0 0 900 600">
                <rect width="900" height="600" fill="#da251d" />
                <polygon fill="#ffff00" points="450,135 487,250 608,250 510,321 547,436 450,365 353,436 390,321 292,250 413,250" />
              </svg>
              <span>VI</span>
            </button>
            <button
              onclick={() => setLanguage("en")}
              class="px-2 py-0.5 rounded text-[11px] font-bold flex items-center space-x-1.5 transition-all cursor-pointer {appStore.language === 'en' ? 'bg-sky-600 text-white shadow-xs' : 'text-slate-600 dark:text-[#a1a1aa] hover:text-slate-900 dark:hover:text-white'}"
              title="English"
            >
              <svg class="w-3.5 h-2.5 rounded-[1px] shrink-0 shadow-xs" viewBox="0 0 60 30">
                <clipPath id="uk-flag-cfg"><path d="M0,0 v30 h60 v-30 z"/></clipPath>
                <clipPath id="uk-diag-cfg"><path d="M30,15 h30 v15 z v15 h-30 z h-30 v-15 z v-15 h30 z"/></clipPath>
                <g clip-path="url(#uk-flag-cfg)">
                  <path d="M0,0 v30 h60 v-30 z" fill="#012169"/>
                  <path d="M0,0 L60,30 M60,0 L0,30" stroke="#fff" stroke-width="6"/>
                  <path d="M0,0 L60,30 M60,0 L0,30" clip-path="url(#uk-diag-cfg)" stroke="#c8102e" stroke-width="4"/>
                  <path d="M30,0 v30 M0,15 h60" stroke="#fff" stroke-width="10"/>
                  <path d="M30,0 v30 M0,15 h60" stroke="#c8102e" stroke-width="6"/>
                </g>
              </svg>
              <span>EN</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- Info & Check Updates Footer -->
  <div class="pt-2 border-t border-slate-200 dark:border-[#1a1a20] flex items-center justify-between mt-auto">
    <div class="text-[11px] text-slate-500 dark:text-[#71717a] font-medium">
      su-profile-switcher • v{APP_VERSION}
    </div>
    <button
      onclick={handleCheckUpdateNow}
      disabled={checkingUpdate}
      class="px-3 py-1.5 bg-slate-100 dark:bg-[#151518] hover:bg-slate-200 dark:hover:bg-[#202026] text-slate-800 dark:text-[#fafafa] rounded-lg text-xs font-bold border border-slate-300 dark:border-[#27272f] transition-colors flex items-center space-x-1.5 cursor-pointer disabled:opacity-50"
    >
      <ArrowClockwise size={13} class={checkingUpdate ? 'animate-spin text-sky-600 dark:text-[#00c8ff]' : 'text-sky-600 dark:text-[#00c8ff]'} />
      <span>{checkingUpdate ? t("checkingUpdate") : t("checkUpdateBtn")}</span>
    </button>
  </div>
</div>
