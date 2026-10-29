<script>
  import { appStore, switchProfile, deleteProfile, setBusy, t } from "../store.svelte.js";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import {
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
    ClockCounterClockwise,
    Funnel,
    FileArrowUp,
    FolderOpen,
    FileZip,
    CaretDown,
    MagnifyingGlass,
    X,
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

  let showFilter = $state(false);
  let searchQuery = $state("");
  let activeFilterChip = $state("all"); // 'all' | 'has_plugins' | 'has_shortcuts' | 'active'
  let isImportMenuOpen = $state(false);

  function getComponentsFor(profile) {
    return appStore.profileComponents.find((c) => c.name === profile) || {
      hasPlugins: true,
      hasShortcuts: true,
      hasToolbars: false,
      hasMaterials: false,
      hasTemplates: false,
      pluginCount: 0,
      shortcutCount: 0,
      createdAt: null,
      updatedAt: null,
    };
  }

  const filteredProfiles = $derived(
    profilesList.filter((p) => {
      if (searchQuery.trim() && !p.toLowerCase().includes(searchQuery.trim().toLowerCase())) {
        return false;
      }
      const comp = getComponentsFor(p);
      if (activeFilterChip === "has_plugins" && (!comp.hasPlugins || comp.pluginCount === 0)) {
        return false;
      }
      if (activeFilterChip === "has_shortcuts" && (!comp.hasShortcuts || comp.shortcutCount === 0)) {
        return false;
      }
      if (activeFilterChip === "active" && p !== activeProfile) {
        return false;
      }
      return true;
    })
  );

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

  async function handleSelectFolder() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Chọn thư mục dữ liệu SketchUp",
      });
      if (selected) {
        await handleRunImportAnalysis(selected);
      }
    } catch (e) {
      alert("Lỗi mở thư mục: " + String(e));
    }
  }

  async function handleSelectZip() {
    try {
      const selected = await open({
        filters: [{ name: "Zip", extensions: ["zip"] }],
        multiple: false,
        title: "Chọn tệp Zip hồ sơ SketchUp",
      });
      if (selected) {
        await handleRunImportAnalysis(selected);
      }
    } catch (e) {
      alert("Lỗi mở tệp Zip: " + String(e));
    }
  }

  async function handleRunImportAnalysis(sourcePath) {
    setBusy(true, "Đang quét và phân tích dữ liệu...");
    try {
      const res = await invoke("analyze_import_source", { sourcePath });
      appStore.modal = {
        type: "import_profile_modal",
        sourcePath,
        analysis: res,
      };
    } catch (e) {
      alert("Lỗi phân tích nguồn nhập: " + String(e));
    } finally {
      setBusy(false);
    }
  }
</script>

<div class="flex flex-col flex-1 h-full space-y-2 overflow-hidden">
  <!-- Top Combined Action Bar -->
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

    <!-- Right: Changelog + Filter + Import + Create Profile -->
    <div class="flex items-center space-x-1.5">
      <!-- Changelog Button -->
      <button
        onclick={() => (appStore.modal = { type: "changelog" })}
        class="flex items-center space-x-1 px-2.5 py-1 bg-slate-100 dark:bg-[#141417] hover:bg-slate-200 dark:hover:bg-[#202026] border border-slate-300 dark:border-[#27272f] rounded-lg text-xs font-semibold text-slate-800 dark:text-[#f4f4f6] transition-colors cursor-pointer"
        title={t("changelogBtn")}
      >
        <ClockCounterClockwise size={13} weight="bold" class="text-sky-600 dark:text-[#00c8ff]" />
        <span>{t("changelogBtn")}</span>
      </button>

      <!-- Filter Toggle Button -->
      <button
        onclick={() => (showFilter = !showFilter)}
        class="flex items-center space-x-1 px-2.5 py-1 rounded-lg text-xs font-semibold transition-colors cursor-pointer {showFilter
          ? 'bg-sky-50 dark:bg-sky-500/20 border border-sky-300 dark:border-sky-500/40 text-sky-700 dark:text-sky-300 font-bold'
          : 'bg-slate-100 dark:bg-[#141417] hover:bg-slate-200 dark:hover:bg-[#202026] border border-slate-300 dark:border-[#27272f] text-slate-800 dark:text-[#f4f4f6]'}"
        title={t("filterBtn")}
      >
        <Funnel size={13} weight={showFilter ? 'fill' : 'bold'} class="text-sky-600 dark:text-[#00c8ff]" />
        <span>{t("filterBtn")}</span>
      </button>

      <!-- Import Profile Dropdown -->
      <div
        role="group"
        class="relative inline-block"
        onmouseenter={() => (isImportMenuOpen = true)}
        onmouseleave={() => (isImportMenuOpen = false)}
      >
        <button
          onclick={() => (isImportMenuOpen = !isImportMenuOpen)}
          class="flex items-center space-x-1 px-2.5 py-1 bg-slate-100 dark:bg-[#141417] hover:bg-slate-200 dark:hover:bg-[#202026] border border-slate-300 dark:border-[#27272f] rounded-lg text-xs font-semibold text-slate-800 dark:text-[#f4f4f6] transition-colors cursor-pointer"
          title={t("importProfileBtn")}
        >
          <FileArrowUp size={13} weight="bold" class="text-sky-600 dark:text-[#00c8ff]" />
          <span>{t("importProfileBtn")}</span>
          <CaretDown size={10} weight="bold" class="text-slate-500" />
        </button>

        {#if isImportMenuOpen}
          <div class="absolute right-0 top-full mt-1 w-36 bg-white dark:bg-[#121216] border border-[#e2e8f0] dark:border-[#262630] rounded-lg shadow-lg z-50 py-1 overflow-hidden">
            <button
              onclick={() => {
                isImportMenuOpen = false;
                handleSelectFolder();
              }}
              class="w-full px-3 py-1.5 flex items-center space-x-2 text-xs font-medium text-slate-700 dark:text-[#d4d4d8] hover:bg-sky-50 dark:hover:bg-[#1c1c24] hover:text-sky-600 dark:hover:text-[#00c8ff] cursor-pointer text-left"
            >
              <FolderOpen size={13} class="text-sky-600 dark:text-[#00c8ff]" />
              <span>{t("importFolderItem")}</span>
            </button>
            <button
              onclick={() => {
                isImportMenuOpen = false;
                handleSelectZip();
              }}
              class="w-full px-3 py-1.5 flex items-center space-x-2 text-xs font-medium text-slate-700 dark:text-[#d4d4d8] hover:bg-sky-50 dark:hover:bg-[#1c1c24] hover:text-sky-600 dark:hover:text-[#00c8ff] cursor-pointer text-left"
            >
              <FileZip size={13} class="text-sky-600 dark:text-[#00c8ff]" />
              <span>{t("importZipItem")}</span>
            </button>
          </div>
        {/if}
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
    </div>
  </div>

  <!-- Dynamic Filter Bar (Appears when Filter button is active) -->
  {#if showFilter}
    <div class="bg-[#ffffff] dark:bg-[#0c0c0e] p-2 rounded-lg border border-[#e2e8f0] dark:border-[#1e1e24] flex items-center space-x-2 shrink-0">
      <!-- Search Input -->
      <div class="relative flex-1">
        <div class="absolute inset-y-0 left-0 pl-2.5 flex items-center pointer-events-none text-slate-400">
          <MagnifyingGlass size={13} weight="bold" />
        </div>
        <input
          type="text"
          bind:value={searchQuery}
          placeholder={t("filterPlaceholder")}
          class="w-full bg-[#f8fafc] dark:bg-[#141418] border border-slate-300 dark:border-[#24242e] rounded-md pl-8 pr-7 py-1 text-xs text-slate-900 dark:text-[#f4f4f6] placeholder-slate-400 focus:outline-none focus:border-sky-500"
        />
        {#if searchQuery}
          <button
            onclick={() => (searchQuery = "")}
            class="absolute inset-y-0 right-0 pr-2 flex items-center text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 cursor-pointer"
          >
            <X size={12} weight="bold" />
          </button>
        {/if}
      </div>

      <!-- Quick Filter Chips -->
      <div class="flex items-center space-x-1 shrink-0">
        {#each [
          { id: 'all', label: t('filterAll') },
          { id: 'has_plugins', label: t('filterHasPlugins') },
          { id: 'has_shortcuts', label: t('filterHasShortcuts') },
          { id: 'active', label: t('filterActive') }
        ] as chip}
          {@const isSelected = activeFilterChip === chip.id}
          <button
            onclick={() => (activeFilterChip = chip.id)}
            class="px-2 py-0.5 rounded text-[11px] transition-colors cursor-pointer {isSelected
              ? 'bg-sky-500/15 text-sky-700 dark:text-sky-300 border border-sky-500/30 font-bold'
              : 'bg-slate-100 dark:bg-[#18181e] text-slate-600 dark:text-[#a1a1aa] hover:bg-slate-200 dark:hover:bg-[#202026] border border-transparent font-medium'}"
          >
            {chip.label}
          </button>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Profiles List Container (Flex-1 expands dynamically with window height) -->
  <div class="flex-1 bg-[#ffffff] dark:bg-[#0a0a0c] border border-[#e2e8f0] dark:border-[#1e1e24] rounded-lg overflow-hidden flex flex-col min-h-0">
    <div class="flex-1 overflow-y-auto divide-y divide-[#e2e8f0] dark:divide-[#1a1a20]">
      {#each filteredProfiles as profile}
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

              <!-- Timestamps (Created & Updated) -->
              <div class="flex items-center space-x-1.5 text-[10px] text-slate-400 dark:text-[#71717a] font-normal pl-2">
                {#if comp.createdAt}
                  <span>{t("createdLabel")}: {comp.createdAt}</span>
                {/if}
                {#if comp.updatedAt}
                  <span>•</span>
                  <span>{t("updatedLabel")}: {comp.updatedAt}</span>
                {/if}
              </div>
            </div>

            <!-- Profile Components Pills (Icons & High Contrast) -->
            <div class="flex items-center space-x-1 pl-7 text-[10px]">
              {#if comp.hasPlugins}
                <span class="inline-flex items-center space-x-1 px-1.5 py-0.5 rounded bg-slate-100 dark:bg-[#141418] text-slate-700 dark:text-[#d4d4d8] border border-slate-300 dark:border-[#222228] font-medium">
                  <PuzzlePiece size={10} weight="bold" class="text-sky-600 dark:text-[#00c8ff]" />
                  <span>{t("tagPlugins")} {comp.pluginCount ? `(${comp.pluginCount})` : ''}</span>
                </span>
              {/if}
              {#if comp.hasShortcuts}
                <span class="inline-flex items-center space-x-1 px-1.5 py-0.5 rounded bg-slate-100 dark:bg-[#141418] text-slate-700 dark:text-[#d4d4d8] border border-slate-300 dark:border-[#222228] font-medium">
                  <Keyboard size={10} weight="bold" class="text-sky-600 dark:text-[#00c8ff]" />
                  <span>{t("tagShortcuts")} {comp.shortcutCount ? `(${comp.shortcutCount})` : ''}</span>
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

      {#if filteredProfiles.length === 0}
        <div class="flex flex-col items-center justify-center py-12 text-slate-400 dark:text-[#71717a]">
          <MagnifyingGlass size={26} weight="duotone" class="mb-2 text-slate-400" />
          <span class="text-xs font-semibold">{t("noProfilesMatch")}</span>
        </div>
      {/if}
    </div>
  </div>
</div>
