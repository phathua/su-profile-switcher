<script>
  import { appStore, switchProfile, deleteProfile, t } from "../store.svelte.js";
  import { invoke } from "@tauri-apps/api/core";
  import {
    Play,
    ArrowsClockwise,
    PencilSimple,
    Trash,
    FolderSimpleStar,
    Folder,
    Plus,
    Check,
    PuzzlePiece,
    Keyboard,
    Toolbox,
    Palette,
    FileText,
  } from "phosphor-svelte";

  const currentVersion = $derived(appStore.state.selectedVersion || "SketchUp");
  const versionConfig = $derived(appStore.state.versions[currentVersion] || { activeProfile: "Default", profiles: [] });
  const activeProfile = $derived(versionConfig.activeProfile || "Default");
  const profilesList = $derived(versionConfig.profiles || []);

  async function handleLaunchSketchUp() {
    const ok = await invoke("launch_sketchup", { version: currentVersion });
    if (!ok) {
      alert("Không tìm thấy SketchUp.");
    }
  }

  async function handleSwitch(profile) {
    if (appStore.sketchupRunning) {
      appStore.modal = {
        type: "confirm_switch",
        targetProfile: profile,
      };
    } else {
      await switchProfile(profile);
    }
  }

  function handleOpenCreateModal() {
    appStore.modal = { type: "create_profile" };
  }

  function handleOpenRenameModal(profile) {
    appStore.modal = { type: "rename", oldName: profile };
  }

  function handleDelete(profile) {
    appStore.modal = {
      type: "confirm_delete",
      profileName: profile,
    };
  }

  function getComponentsFor(profile) {
    return appStore.profileComponents.find((c) => c.name === profile) || {
      hasPlugins: true,
      hasShortcuts: true,
      hasToolbars: false,
      hasMaterials: false,
      hasTemplates: false,
    };
  }
</script>

<div class="space-y-2">
  <!-- Top Combined Action Bar (Active profile info + Launch & Add actions) -->
  <div class="flex items-center justify-between bg-[#ffffff] dark:bg-[#0c0c0e] p-2 rounded-lg border border-[#e2e8f0] dark:border-[#1e1e24]">
    <!-- Left: Active Profile Info & Count -->
    <div class="flex items-center space-x-2">
      <div class="w-7 h-7 rounded-lg bg-sky-500/15 flex items-center justify-center text-sky-600 dark:text-[#00c8ff]">
        <FolderSimpleStar size={16} weight="fill" />
      </div>
      <div class="flex items-center space-x-2">
        <span class="font-bold text-xs text-slate-900 dark:text-white">{activeProfile}</span>
        <span class="inline-flex items-center space-x-0.5 px-1.5 py-0.5 rounded-md text-[10px] font-bold bg-sky-500/15 text-sky-700 dark:text-sky-300 border border-sky-500/30">
          <Check size={9} weight="bold" />
          <span>{t("activeLabel")}</span>
        </span>
        <span class="text-[11px] font-medium text-slate-500 dark:text-[#71717a] ml-1">
          ({profilesList.length} {t("profileCount")})
        </span>
      </div>
    </div>

    <!-- Right: Process Status + Add Profile + Launch SketchUp -->
    <div class="flex items-center space-x-1.5">
      <!-- Process Status Pill -->
      <div class="flex items-center space-x-1.5 px-2 py-1 rounded-md {appStore.sketchupRunning ? 'bg-sky-500/15 text-sky-700 dark:text-[#00c8ff] border border-sky-500/30' : 'bg-slate-100 dark:bg-[#151518] text-slate-600 dark:text-[#a1a1aa] border border-slate-200 dark:border-[#242429]'} text-[11px] font-semibold">
        <span class="w-1.5 h-1.5 rounded-full {appStore.sketchupRunning ? 'bg-sky-500 animate-pulse' : 'bg-slate-400 dark:bg-[#71717a]'}"></span>
        <span>{appStore.sketchupRunning ? t("statusRunning") : t("statusClosed")}</span>
      </div>

      <!-- Add Profile Button (+) -->
      <button
        onclick={handleOpenCreateModal}
        class="flex items-center space-x-1 px-2.5 py-1 bg-slate-100 dark:bg-[#141417] hover:bg-slate-200 dark:hover:bg-[#202026] border border-slate-300 dark:border-[#27272f] rounded-lg text-xs font-semibold text-slate-800 dark:text-[#f4f4f6] transition-colors cursor-pointer"
        title={t("addProfile")}
      >
        <Plus size={13} weight="bold" class="text-sky-600 dark:text-[#00c8ff]" />
        <span>{t("addProfile")}</span>
      </button>

      <!-- Launch SketchUp Button (High contrast white text on sky background) -->
      <button
        onclick={handleLaunchSketchUp}
        class="flex items-center space-x-1.5 px-3 py-1 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-xs font-bold shadow-xs transition-colors cursor-pointer"
      >
        <Play size={12} weight="fill" />
        <span>{t("launchSu")}</span>
      </button>
    </div>
  </div>

  <!-- Profiles List Container (Expanded height to 220px) -->
  <div class="bg-[#ffffff] dark:bg-[#0a0a0c] border border-[#e2e8f0] dark:border-[#1e1e24] rounded-lg overflow-hidden">
    <div class="max-h-[220px] overflow-y-auto divide-y divide-[#e2e8f0] dark:divide-[#1a1a20]">
      {#each profilesList as profile}
        {@const isActive = profile === activeProfile}
        {@const comp = getComponentsFor(profile)}
        <div class="px-3 py-2 flex items-center justify-between hover:bg-[#f8fafc] dark:hover:bg-[#121216] transition-colors {isActive ? 'bg-[#f0f9ff] dark:bg-[#0f1117]' : ''}">
          <!-- Left: Profile Info & Components -->
          <div class="space-y-1">
            <div class="flex items-center space-x-2">
              <div class="w-5 h-5 rounded flex items-center justify-center {isActive ? 'bg-[#00c8ff]/20 text-[#0284c7] dark:text-[#00c8ff]' : 'bg-[#e2e8f0] dark:bg-[#1a1a20] text-[#64748b] dark:text-[#a1a1aa]'}">
                <Folder size={12} weight={isActive ? 'fill' : 'regular'} />
              </div>
              <span class="text-xs font-semibold {isActive ? 'text-[#0284c7] dark:text-[#ffffff]' : 'text-[#334155] dark:text-[#d4d4d8]'}">{profile}</span>
              {#if isActive}
                <span class="text-[10px] text-[#0284c7] dark:text-[#00c8ff] font-bold flex items-center space-x-0.5">
                  <Check size={10} weight="bold" />
                </span>
              {/if}
            </div>

            <!-- Profile Components Pills (Icons & High Contrast) -->
            <div class="flex items-center space-x-1 pl-7 text-[10px]">
              {#if comp.hasPlugins}
                <span class="inline-flex items-center space-x-1 px-1.5 py-0.5 rounded bg-slate-100 dark:bg-[#141418] text-slate-700 dark:text-[#d4d4d8] border border-slate-300 dark:border-[#222228] font-medium">
                  <PuzzlePiece size={10} weight="bold" class="text-sky-600 dark:text-[#00c8ff]" />
                  <span>{t("tagPlugins")}</span>
                </span>
              {/if}
              {#if comp.hasShortcuts}
                <span class="inline-flex items-center space-x-1 px-1.5 py-0.5 rounded bg-slate-100 dark:bg-[#141418] text-slate-700 dark:text-[#d4d4d8] border border-slate-300 dark:border-[#222228] font-medium">
                  <Keyboard size={10} weight="bold" class="text-sky-600 dark:text-[#00c8ff]" />
                  <span>{t("tagShortcuts")}</span>
                </span>
              {/if}
              {#if comp.hasToolbars}
                <span class="inline-flex items-center space-x-1 px-1.5 py-0.5 rounded bg-slate-100 dark:bg-[#141418] text-slate-700 dark:text-[#d4d4d8] border border-slate-300 dark:border-[#222228] font-medium">
                  <Toolbox size={10} weight="bold" class="text-sky-600 dark:text-[#00c8ff]" />
                  <span>{t("tagToolbars")}</span>
                </span>
              {/if}
              {#if comp.hasMaterials}
                <span class="inline-flex items-center space-x-1 px-1.5 py-0.5 rounded bg-slate-100 dark:bg-[#141418] text-slate-700 dark:text-[#d4d4d8] border border-slate-300 dark:border-[#222228] font-medium">
                  <Palette size={10} weight="bold" class="text-sky-600 dark:text-[#00c8ff]" />
                  <span>{t("tagMaterials")}</span>
                </span>
              {/if}
              {#if comp.hasTemplates}
                <span class="inline-flex items-center space-x-1 px-1.5 py-0.5 rounded bg-slate-100 dark:bg-[#141418] text-slate-700 dark:text-[#d4d4d8] border border-slate-300 dark:border-[#222228] font-medium">
                  <FileText size={10} weight="bold" class="text-sky-600 dark:text-[#00c8ff]" />
                  <span>{t("tagTemplates")}</span>
                </span>
              {/if}
            </div>
          </div>

          <!-- Actions -->
          <div class="flex items-center space-x-1 shrink-0">
            {#if !isActive}
              <button
                onclick={() => handleSwitch(profile)}
                class="p-1.5 hover:bg-[#e2e8f0] dark:hover:bg-[#1e1e26] rounded text-[#64748b] dark:text-[#a1a1aa] hover:text-[#0284c7] dark:hover:text-[#00c8ff] transition-colors cursor-pointer"
                title={t("switchTooltip")}
              >
                <ArrowsClockwise size={13} weight="bold" />
              </button>
            {/if}

            <button
              onclick={() => handleOpenRenameModal(profile)}
              class="p-1.5 hover:bg-[#e2e8f0] dark:hover:bg-[#1e1e26] rounded text-[#64748b] dark:text-[#a1a1aa] hover:text-[#0f172a] dark:hover:text-[#fafafa] transition-colors cursor-pointer"
              title={t("renameTooltip")}
            >
              <PencilSimple size={13} />
            </button>

            {#if !isActive}
              <button
                onclick={() => handleDelete(profile)}
                class="p-1.5 hover:bg-[#fee2e2] dark:hover:bg-[#ef4444]/20 rounded text-[#64748b] dark:text-[#a1a1aa] hover:text-[#ef4444] transition-colors cursor-pointer"
                title={t("deleteTooltip")}
              >
                <Trash size={13} />
              </button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  </div>
</div>
