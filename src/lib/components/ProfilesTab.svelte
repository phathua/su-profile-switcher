<script>
  import { appStore, switchProfile, deleteProfile, t } from "../store.svelte.js";
  import { invoke } from "@tauri-apps/api/core";
  import {
    Play,
    Stop,
    ArrowsClockwise,
    PencilSimple,
    Eye,
    Trash,
    FolderSimpleStar,
    Folder,
    Briefcase,
    PaintBrush,
    Cube,
    Compass,
    Lightning,
    Star,
    Sparkle,
    Cpu,
    GameController,
    Plus,
    Check,
    PuzzlePiece,
    Keyboard,
    Toolbox,
    Palette,
    FileText,
  } from "phosphor-svelte";

  const iconMap = {
    Folder,
    Briefcase,
    PaintBrush,
    Cube,
    Compass,
    Lightning,
    Star,
    Sparkle,
    Cpu,
    GameController,
  };

  const currentVersion = $derived(appStore.state.selectedVersion || "SketchUp");
  const versionConfig = $derived(appStore.state.versions[currentVersion] || { activeProfile: "Default", profiles: [], profilesMetadata: {} });
  const activeProfile = $derived(versionConfig.activeProfile || "Default");
  const profilesList = $derived(versionConfig.profiles || []);
  const profilesMetadata = $derived(versionConfig.profilesMetadata || {});

  async function handleActionSketchUp() {
    if (appStore.sketchupRunning) {
      await invoke("close_sketchup");
      appStore.sketchupRunning = false;
    } else {
      const ok = await invoke("launch_sketchup", { version: currentVersion });
      if (!ok) {
        alert("Không tìm thấy SketchUp.");
      }
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

  function handleOpenViewModal(profile) {
    appStore.modal = { type: "view_profile", profileName: profile };
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

<div class="flex flex-col flex-1 h-full space-y-2 overflow-hidden">
  <!-- Top Combined Action Bar (Active profile info + Launch & Add actions) -->
  <div class="flex items-center justify-between bg-[#ffffff] dark:bg-[#0c0c0e] p-2 rounded-lg border border-[#e2e8f0] dark:border-[#1e1e24] shrink-0">
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

    <!-- Right: Add Profile + Action SketchUp Button -->
    <div class="flex items-center space-x-1.5">
      <!-- Add Profile Button (+) -->
      <button
        onclick={handleOpenCreateModal}
        class="flex items-center space-x-1 px-2.5 py-1 bg-slate-100 dark:bg-[#141417] hover:bg-slate-200 dark:hover:bg-[#202026] border border-slate-300 dark:border-[#27272f] rounded-lg text-xs font-semibold text-slate-800 dark:text-[#f4f4f6] transition-colors cursor-pointer"
        title={t("addProfile")}
      >
        <Plus size={13} weight="bold" class="text-sky-600 dark:text-[#00c8ff]" />
        <span>{t("addProfile")}</span>
      </button>

      <!-- Launch / Force Close SketchUp Button (High contrast, dynamic state) -->
      <button
        onclick={handleActionSketchUp}
        class="flex items-center space-x-1.5 px-3 py-1 rounded-lg text-xs font-bold shadow-xs transition-colors cursor-pointer {appStore.sketchupRunning
          ? 'bg-rose-600 hover:bg-rose-500 text-white'
          : 'bg-sky-600 hover:bg-sky-500 text-white'}"
      >
        {#if appStore.sketchupRunning}
          <Stop size={12} weight="fill" />
          <span>{t("killSu")}</span>
        {:else}
          <Play size={12} weight="fill" />
          <span>{t("launchSu")}</span>
        {/if}
      </button>
    </div>
  </div>

  <!-- Profiles List Container (Flex-1 expands dynamically with window height) -->
  <div class="flex-1 bg-[#ffffff] dark:bg-[#0a0a0c] border border-[#e2e8f0] dark:border-[#1e1e24] rounded-lg overflow-hidden flex flex-col min-h-0">
    <div class="flex-1 overflow-y-auto divide-y divide-[#e2e8f0] dark:divide-[#1a1a20]">
      {#each profilesList as profile}
        {@const isActive = profile === activeProfile}
        {@const comp = getComponentsFor(profile)}
        {@const meta = profilesMetadata[profile] || {}}
        {@const CustomIcon = meta.icon && iconMap[meta.icon] ? iconMap[meta.icon] : Folder}
        {@const customColor = meta.color || (isActive ? '#0284c7' : '#64748b')}
        <div class="px-3 py-2 flex items-center justify-between hover:bg-[#f8fafc] dark:hover:bg-[#121216] transition-colors {isActive ? 'bg-[#f0f9ff] dark:bg-[#0f1117]' : ''}">
          <!-- Left: Profile Info & Components -->
          <div class="space-y-1">
            <div class="flex items-center space-x-2">
              <div
                class="w-5 h-5 rounded flex items-center justify-center {isActive ? 'ring-1 ring-offset-1 ring-sky-500/50' : ''}"
                style="background-color: {customColor}22; color: {customColor};"
              >
                <CustomIcon size={12} weight={isActive ? 'fill' : 'regular'} />
              </div>
              <span class="text-xs font-semibold {isActive ? 'text-[#0284c7] dark:text-[#ffffff]' : 'text-[#334155] dark:text-[#d4d4d8]'}">{profile}</span>
              {#if meta.shortcut}
                <span class="px-1.5 py-0.2 bg-slate-100 dark:bg-[#1a1a20] border border-slate-300 dark:border-[#2e2e38] rounded text-[9px] font-mono text-slate-600 dark:text-[#94a3b8]">
                  {meta.shortcut}
                </span>
              {/if}
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
              onclick={() => handleOpenViewModal(profile)}
              class="p-1.5 hover:bg-[#e2e8f0] dark:hover:bg-[#1e1e26] rounded text-[#64748b] dark:text-[#a1a1aa] hover:text-[#0284c7] dark:hover:text-[#00c8ff] transition-colors cursor-pointer"
              title="Xem chi tiết"
            >
              <Eye size={13} />
            </button>

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
