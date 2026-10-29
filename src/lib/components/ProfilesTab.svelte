<script>
  import { onMount } from "svelte";
  import { appStore, switchProfile, deleteProfile, setBusy, t } from "../store.svelte.js";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import CustomDropdown from "./CustomDropdown.svelte";
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
    DotsThreeVertical,
    SortAscending,
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
  let sortBy = $state("created_desc");
  let isImportMenuOpen = $state(false);
  let activeRowMenu = $state(null);

  const sortOptions = $derived([
    { value: "created_desc", label: t("sortCreatedDesc") },
    { value: "created_asc", label: t("sortCreatedAsc") },
    { value: "updated_desc", label: t("sortUpdatedDesc") },
    { value: "name_asc", label: t("sortNameAsc") },
    { value: "name_desc", label: t("sortNameDesc") },
    { value: "plugins_desc", label: t("sortPluginsDesc") },
    { value: "shortcuts_desc", label: t("sortShortcutsDesc") },
  ]);

  onMount(() => {
    const handleGlobalClick = (e) => {
      if (!e.target.closest(".import-dropdown-container")) {
        isImportMenuOpen = false;
      }
      if (!e.target.closest(".row-menu-container")) {
        activeRowMenu = null;
      }
    };
    window.addEventListener("click", handleGlobalClick);
    return () => {
      window.removeEventListener("click", handleGlobalClick);
    };
  });

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

  function parseDate(dStr) {
    if (!dStr) return 0;
    const parts = dStr.split(" ");
    const dateParts = parts[0].split("/");
    if (dateParts.length < 3) return 0;
    const day = parseInt(dateParts[0], 10);
    const month = parseInt(dateParts[1], 10) - 1;
    const year = parseInt(dateParts[2], 10);
    let hours = 0;
    let minutes = 0;
    if (parts[1]) {
      const timeParts = parts[1].split(":");
      hours = parseInt(timeParts[0] || "0", 10);
      minutes = parseInt(timeParts[1] || "0", 10);
    }
    return new Date(year, month, day, hours, minutes).getTime();
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

  const sortedProfiles = $derived(
    [...filteredProfiles].sort((a, b) => {
      const aComp = getComponentsFor(a);
      const bComp = getComponentsFor(b);

      switch (sortBy) {
        case "created_desc":
          return parseDate(bComp.createdAt) - parseDate(aComp.createdAt);
        case "created_asc":
          return parseDate(aComp.createdAt) - parseDate(bComp.createdAt);
        case "updated_desc":
          return parseDate(bComp.updatedAt) - parseDate(aComp.updatedAt);
        case "name_asc":
          return a.localeCompare(b);
        case "name_desc":
          return b.localeCompare(a);
        case "plugins_desc":
          return (bComp.pluginCount || 0) - (aComp.pluginCount || 0);
        case "shortcuts_desc":
          return (bComp.shortcutCount || 0) - (aComp.shortcutCount || 0);
        default:
          return 0;
      }
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
      const res = await invoke("analyze_batch_import_source", {
        version: currentVersion,
        sourcePath,
      });
      appStore.modal = {
        type: "batch_import_modal",
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

    <!-- Right: Filter + Import + Create Profile -->
    <div class="flex items-center space-x-1.5">
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

      <!-- Import Profile Dropdown (Fixed mouse hover gap) -->
      <div class="relative inline-block import-dropdown-container">
        <button
          onclick={(e) => {
            e.stopPropagation();
            isImportMenuOpen = !isImportMenuOpen;
          }}
          class="flex items-center space-x-1 px-2.5 py-1 bg-slate-100 dark:bg-[#141417] hover:bg-slate-200 dark:hover:bg-[#202026] border border-slate-300 dark:border-[#27272f] rounded-lg text-xs font-semibold text-slate-800 dark:text-[#f4f4f6] transition-colors cursor-pointer"
          title={t("importProfileBtn")}
        >
          <FileArrowUp size={13} weight="bold" class="text-sky-600 dark:text-[#00c8ff]" />
          <span>{t("importProfileBtn")}</span>
          <CaretDown size={10} weight="bold" class="text-slate-500" />
        </button>

        {#if isImportMenuOpen}
          <div class="absolute right-0 top-full pt-1 z-50">
            <div class="w-36 bg-white dark:bg-[#121216] border border-[#e2e8f0] dark:border-[#262630] rounded-lg shadow-xl py-1 overflow-hidden">
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

  <!-- Dynamic Filter & Sort Bar -->
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

      <!-- Sort Dropdown Selector -->
      <div class="flex items-center space-x-1.5 shrink-0 border-l border-slate-200 dark:border-[#1e1e24] pl-2">
        <span class="text-[11px] text-slate-500 dark:text-[#71717a] font-medium">{t("sortBy")}</span>
        <CustomDropdown
          options={sortOptions}
          value={sortBy}
          onchange={(val) => (sortBy = val)}
          class="!py-0.5 !px-2 !text-[11px]"
        />
      </div>
    </div>
  {/if}

  <!-- Profiles List Container -->
  <div class="flex-1 bg-[#ffffff] dark:bg-[#0a0a0c] border border-[#e2e8f0] dark:border-[#1e1e24] rounded-lg overflow-hidden flex flex-col min-h-0">
    <div class="flex-1 overflow-y-auto divide-y divide-[#e2e8f0] dark:divide-[#1a1a20]">
      {#each sortedProfiles as profile}
        {@const isActive = profile === activeProfile}
        {@const comp = getComponentsFor(profile)}
        {@const meta = profilesMetadata[profile] || {}}
        {@const CustomIcon = meta.icon && iconMap[meta.icon] ? iconMap[meta.icon] : Folder}
        {@const customColor = meta.color || (isActive ? '#0284c7' : '#64748b')}
        <div class="px-3 py-2 flex items-center justify-between hover:bg-[#f8fafc] dark:hover:bg-[#121216] transition-colors {isActive ? 'bg-[#f0f9ff] dark:bg-[#0f1117]' : ''}">
          <!-- Left: Profile Info & Components -->
          <div class="space-y-1 flex-1 min-w-0 pr-2">
            <div class="flex items-center space-x-2">
              <div
                class="w-5 h-5 rounded flex items-center justify-center shrink-0 {isActive ? 'ring-1 ring-offset-1 ring-sky-500/50' : ''}"
                style="background-color: {customColor}22; color: {customColor};"
              >
                <CustomIcon size={12} weight={isActive ? 'fill' : 'regular'} />
              </div>
              <span class="text-xs font-semibold truncate {isActive ? 'text-[#0284c7] dark:text-[#ffffff]' : 'text-[#334155] dark:text-[#d4d4d8]'}">{profile}</span>
              {#if meta.shortcut}
                <span class="px-1.5 py-0.2 bg-slate-100 dark:bg-[#1a1a20] border border-slate-300 dark:border-[#2e2e38] rounded text-[9px] font-mono text-slate-600 dark:text-[#94a3b8] shrink-0">
                  {meta.shortcut}
                </span>
              {/if}
              {#if isActive}
                <span class="text-[10px] text-[#0284c7] dark:text-[#00c8ff] font-bold flex items-center space-x-0.5 shrink-0">
                  <Check size={10} weight="bold" />
                </span>
              {/if}

              <!-- Timestamps (Created & Updated) -->
              <div class="flex items-center space-x-1.5 text-[10px] text-slate-400 dark:text-[#71717a] font-normal pl-2 shrink-0">
                {#if comp.createdAt}
                  <span>{t("createdLabel")}: {comp.createdAt}</span>
                {/if}
                {#if comp.updatedAt}
                  <span>•</span>
                  <span>{t("updatedLabel")}: {comp.updatedAt}</span>
                {/if}
              </div>
            </div>

            <!-- Profile Components Pills -->
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

          <!-- Right Actions: Switch Button + 3-dots Menu Button -->
          <div class="flex items-center space-x-1 shrink-0 relative row-menu-container">
            {#if !isActive}
              <button
                onclick={() => handleSwitch(profile)}
                class="p-1.5 hover:bg-[#e2e8f0] dark:hover:bg-[#1e1e26] rounded text-[#64748b] dark:text-[#a1a1aa] hover:text-[#0284c7] dark:hover:text-[#00c8ff] transition-colors cursor-pointer"
                title={t("switchTooltip")}
              >
                <ArrowsClockwise size={13} weight="bold" />
              </button>
            {/if}

            <!-- 3-dots Menu Button -->
            <div class="relative">
              <button
                onclick={(e) => {
                  e.stopPropagation();
                  activeRowMenu = activeRowMenu === profile ? null : profile;
                }}
                class="p-1.5 hover:bg-[#e2e8f0] dark:hover:bg-[#1e1e26] rounded text-[#64748b] dark:text-[#a1a1aa] hover:text-[#0f172a] dark:hover:text-white transition-colors cursor-pointer {activeRowMenu === profile ? 'bg-[#e2e8f0] dark:bg-[#1e1e26] text-sky-600 dark:text-[#00c8ff]' : ''}"
                title="Tùy chọn hồ sơ"
              >
                <DotsThreeVertical size={14} weight="bold" />
              </button>

              {#if activeRowMenu === profile}
                <div class="absolute right-0 top-full pt-1 z-40">
                  <div class="w-38 bg-white dark:bg-[#121216] border border-[#e2e8f0] dark:border-[#262630] rounded-lg shadow-xl py-1 overflow-hidden">
                    <!-- Chỉnh sửa -->
                    <button
                      onclick={() => {
                        activeRowMenu = null;
                        handleOpenRenameModal(profile);
                      }}
                      class="w-full px-3 py-1.5 flex items-center space-x-2 text-xs font-medium text-slate-700 dark:text-[#d4d4d8] hover:bg-sky-50 dark:hover:bg-[#1c1c24] hover:text-sky-600 dark:hover:text-[#00c8ff] cursor-pointer text-left"
                    >
                      <PencilSimple size={13} />
                      <span>{t("editProfile")}</span>
                    </button>

                    <!-- Xem chi tiết -->
                    <button
                      onclick={() => {
                        activeRowMenu = null;
                        handleOpenViewModal(profile);
                      }}
                      class="w-full px-3 py-1.5 flex items-center space-x-2 text-xs font-medium text-slate-700 dark:text-[#d4d4d8] hover:bg-sky-50 dark:hover:bg-[#1c1c24] hover:text-sky-600 dark:hover:text-[#00c8ff] cursor-pointer text-left"
                    >
                      <Eye size={13} />
                      <span>{t("viewProfile")}</span>
                    </button>

                    <!-- Xem nhật ký thay đổi -->
                    <button
                      onclick={() => {
                        activeRowMenu = null;
                        appStore.modal = { type: "changelog", targetProfile: profile };
                      }}
                      class="w-full px-3 py-1.5 flex items-center space-x-2 text-xs font-medium text-slate-700 dark:text-[#d4d4d8] hover:bg-sky-50 dark:hover:bg-[#1c1c24] hover:text-sky-600 dark:hover:text-[#00c8ff] cursor-pointer text-left"
                    >
                      <ClockCounterClockwise size={13} />
                      <span>{t("profileChangelog")}</span>
                    </button>

                    <!-- Xóa hồ sơ -->
                    {#if !isActive}
                      <div class="border-t border-[#e2e8f0] dark:border-[#1e1e24] my-0.5"></div>
                      <button
                        onclick={() => {
                          activeRowMenu = null;
                          handleDelete(profile);
                        }}
                        class="w-full px-3 py-1.5 flex items-center space-x-2 text-xs font-medium text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-500/10 cursor-pointer text-left"
                      >
                        <Trash size={13} />
                        <span>{t("deleteProfile")}</span>
                      </button>
                    {/if}
                  </div>
                </div>
              {/if}
            </div>
          </div>
        </div>
      {/each}

      {#if sortedProfiles.length === 0}
        <div class="flex flex-col items-center justify-center py-12 text-slate-400 dark:text-[#71717a]">
          <MagnifyingGlass size={26} weight="duotone" class="mb-2 text-slate-400" />
          <span class="text-xs font-semibold">{t("noProfilesMatch")}</span>
        </div>
      {/if}
    </div>
  </div>
</div>
