<script>
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { appStore, initAppStore, checkProcessStatus, selectVersion, t } from "./lib/store.svelte.js";
  import TitleBar from "./lib/components/TitleBar.svelte";
  import NavTabs from "./lib/components/NavTabs.svelte";
  import ProfilesTab from "./lib/components/ProfilesTab.svelte";
  import BackupTab from "./lib/components/BackupTab.svelte";
  import SettingsTab from "./lib/components/SettingsTab.svelte";
  import Modal from "./lib/components/Modal.svelte";
  import SaveWindow from "./lib/components/SaveWindow.svelte";
  import CustomDropdown from "./lib/components/CustomDropdown.svelte";
  import { Play, Stop } from "phosphor-svelte";
  import { invoke } from "@tauri-apps/api/core";

  const isSaveMode = window.location.search.includes("mode=save");

  async function handleActionSketchUp() {
    if (appStore.sketchupRunning) {
      await invoke("close_sketchup");
      appStore.sketchupRunning = false;
    } else {
      const ok = await invoke("launch_sketchup", { version: appStore.state.selectedVersion });
      if (!ok) {
        alert("Không tìm thấy SketchUp.");
      }
    }
  }

  onMount(() => {
    if (isSaveMode) return;

    initAppStore();

    // Listen for tray events
    const unlistenSwitched = listen("profile-switched", () => {
      initAppStore();
    });

    const unlistenVer = listen("version-changed", () => {
      initAppStore();
    });

    const unlistenSettings = listen("settings-updated", () => {
      initAppStore();
    });

    const unlistenProfiles = listen("profiles-updated", () => {
      initAppStore();
    });

    const unlistenSave = listen("trigger-save-profile", () => {
      appStore.modal = { type: "create_profile" };
    });

    const unlistenConfirmSwitch = listen("request-confirm-switch", (event) => {
      const payload = event.payload;
      appStore.modal = {
        type: "confirm_switch",
        targetProfile: payload.targetProfile,
      };
    });

    // Check SketchUp process every 3 seconds
    const interval = setInterval(() => {
      checkProcessStatus();
    }, 3000);

    return () => {
      unlistenSwitched.then((f) => f());
      unlistenVer.then((f) => f());
      unlistenSettings.then((f) => f());
      unlistenProfiles.then((f) => f());
      unlistenSave.then((f) => f());
      unlistenConfirmSwitch.then((f) => f());
      clearInterval(interval);
    };
  });
</script>

{#if isSaveMode}
  <SaveWindow />
{:else}
  <main class="h-screen w-screen flex flex-col bg-[#f8fafc] dark:bg-[#000000] text-[#0f172a] dark:text-[#fafafa] font-sans overflow-hidden select-none rounded-none shadow-none">
    <!-- Custom Draggable Title Bar -->
    <TitleBar />

    <!-- App Header & Navigation Tabs with SketchUp Version Dropdown on the right -->
    <div class="px-3.5 pt-2 pb-2 flex items-center justify-between border-b border-[#e2e8f0] dark:border-[#18181b] bg-[#ffffff] dark:bg-[#050507]">
      <NavTabs />

      <!-- Right Controls: Launch/Close SketchUp + Version Dropdown -->
      <div class="flex items-center space-x-2 relative z-30">
        <button
          onclick={handleActionSketchUp}
          class="flex items-center space-x-1.5 px-3 py-1.5 rounded-lg text-xs font-bold shadow-xs transition-colors cursor-pointer {appStore.sketchupRunning
            ? 'bg-rose-600 hover:bg-rose-500 text-white'
            : 'bg-sky-600 hover:bg-sky-500 text-white'}"
          title={appStore.sketchupRunning ? t("killSu") : t("launchSu")}
        >
          {#if appStore.sketchupRunning}
            <Stop size={13} weight="fill" />
            <span>{t("killSu")}</span>
          {:else}
            <Play size={13} weight="fill" />
            <span>{t("launchSu")}</span>
          {/if}
        </button>

        <CustomDropdown
          options={appStore.installedVersions}
          value={appStore.state.selectedVersion}
          onchange={(ver) => selectVersion(ver)}
        />
      </div>
    </div>

    <!-- Tab Content (Flex-1 to dynamically expand on window resize) -->
    <div class="flex-1 p-3 overflow-hidden flex flex-col bg-[#f8fafc] dark:bg-[#000000]">
      {#if appStore.activeTab === "profiles"}
        <ProfilesTab />
      {:else if appStore.activeTab === "backup"}
        <BackupTab />
      {:else if appStore.activeTab === "settings"}
        <SettingsTab />
      {/if}
    </div>

    <!-- Central Modal Controller -->
    <Modal />

    <!-- Global Processing Overlay (Blocks all interactions during heavy tasks) -->
    {#if appStore.busy.active}
      <div
        class="fixed inset-0 z-[9999] bg-black/50 backdrop-blur-[2px] flex flex-col items-center justify-center select-none cursor-wait transition-all animate-in fade-in duration-150"
      >
        <div class="bg-white dark:bg-[#121215] border border-slate-200 dark:border-[#27272f] rounded-xl px-6 py-5 shadow-2xl flex flex-col items-center space-y-3 max-w-[300px] text-center">
          <div class="w-8 h-8 rounded-full border-3 border-sky-500 border-t-transparent animate-spin"></div>
          <div class="text-xs font-bold text-slate-800 dark:text-white">
            {appStore.busy.message || "Đang xử lý dữ liệu..."}
          </div>
          <div class="text-[10px] text-slate-500 dark:text-[#a1a1aa]">
            Vui lòng đợi trong giây lát...
          </div>
        </div>
      </div>
    {/if}
  </main>
{/if}
