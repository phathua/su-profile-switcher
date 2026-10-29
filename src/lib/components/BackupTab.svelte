<script>
  import { appStore, setBusy, t, loadProfileComponents } from "../store.svelte.js";
  import { save, open } from "@tauri-apps/plugin-dialog";
  import { invoke } from "@tauri-apps/api/core";
  import {
    Archive,
    FileArrowDown,
    FileArrowUp,
    FolderOpen,
    CheckCircle,
    FileZip,
    PuzzlePiece,
    Keyboard,
    Toolbox,
    Palette,
    FileText,
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

  let subTab = $state("backup"); // 'backup' | 'import'

  const currentVersion = $derived(appStore.state.selectedVersion || "SketchUp");
  const versionConfig = $derived(
    appStore.state.versions[currentVersion] || {
      activeProfile: "Default",
      profiles: [],
      profilesMetadata: {},
    }
  );
  const activeProfile = $derived(versionConfig.activeProfile || "Default");
  const profilesList = $derived(versionConfig.profiles || []);
  const profilesMetadata = $derived(versionConfig.profilesMetadata || {});

  // Backup selection state
  let backupItems = $state([]);

  // Sync backupItems when profilesList changes
  $effect(() => {
    const list = profilesList;
    const existingMap = new Map(backupItems.map((i) => [i.name, i]));
    backupItems = list.map((name) => {
      const existing = existingMap.get(name);
      return {
        name,
        selected: existing ? existing.selected : true,
        includePlugins: existing ? existing.includePlugins : true,
        includeShortcuts: existing ? existing.includeShortcuts : true,
        includeToolbars: existing ? existing.includeToolbars : true,
        includeMaterials: existing ? existing.includeMaterials : false,
        includeTemplates: existing ? existing.includeTemplates : false,
      };
    });
  });

  function getProfileIcon(name) {
    const meta = profilesMetadata[name];
    const iconKey = meta?.icon || "Folder";
    return iconMap[iconKey] || Folder;
  }

  function getProfileColor(name) {
    const meta = profilesMetadata[name];
    return meta?.color || "#0284c7";
  }

  function toggleAllBackup(checked) {
    backupItems = backupItems.map((item) => ({ ...item, selected: checked }));
  }

  async function handleExportBatchZip() {
    const selected = backupItems.filter((i) => i.selected);
    if (selected.length === 0) {
      alert(t("noProfilesSelected") || "Vui lòng chọn ít nhất một hồ sơ.");
      return;
    }

    const now = new Date();
    const mm = String(now.getMonth() + 1).padStart(2, "0");
    const dd = String(now.getDate()).padStart(2, "0");
    const hh = String(now.getHours()).padStart(2, "0");
    const min = String(now.getMinutes()).padStart(2, "0");
    const defaultName = `SketchUp_${currentVersion.replace(/\s+/g, "_")}_Backup_${mm}${dd}_${hh}${min}.zip`;

    const dest = await save({
      filters: [{ name: "Zip", extensions: ["zip"] }],
      defaultPath: defaultName,
    });

    if (dest) {
      setBusy(true, "Đang đóng gói hồ sơ sao lưu...");
      try {
        const profilesPayload = selected.map((i) => ({
          name: i.name,
          includePlugins: i.includePlugins,
          includeShortcuts: i.includeShortcuts,
          includeToolbars: i.includeToolbars,
          includeMaterials: i.includeMaterials,
          includeTemplates: i.includeTemplates,
        }));

        await invoke("export_batch_backup_zip", {
          version: currentVersion,
          profiles: profilesPayload,
          zipPath: dest,
        });

        alert(`Đã xuất gói sao lưu thành công (${selected.length} hồ sơ)!`);
      } catch (e) {
        alert("Lỗi xuất sao lưu: " + String(e));
      } finally {
        setBusy(false);
      }
    }
  }

  async function handleSelectFolder() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Chọn thư mục sao lưu SketchUp",
      });
      if (selected) {
        await runBatchAnalysis(selected);
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
        title: "Chọn tệp Zip sao lưu SketchUp",
      });
      if (selected) {
        await runBatchAnalysis(selected);
      }
    } catch (e) {
      alert("Lỗi mở file zip: " + String(e));
    }
  }

  async function runBatchAnalysis(sourcePath) {
    setBusy(true, "Đang quét và phân tích dữ liệu sao lưu...");
    try {
      const res = await invoke("analyze_batch_import_source", {
        version: currentVersion,
        sourcePath,
      });

      if (!res.profiles || res.profiles.length === 0) {
        alert("Không tìm thấy hồ sơ SketchUp hợp lệ nào trong nguồn đã chọn.");
        return;
      }

      appStore.modal = {
        type: "batch_import_modal",
        sourcePath,
        analysis: res,
      };
    } catch (e) {
      alert("Lỗi phân tích gói nhập: " + String(e));
    } finally {
      setBusy(false);
    }
  }
</script>

<div class="flex flex-col flex-1 h-full space-y-2 overflow-hidden">
  <!-- Sub-navigation Toggle -->
  <div class="flex items-center justify-between bg-[#ffffff] dark:bg-[#0c0c0e] p-1 rounded-lg border border-[#e2e8f0] dark:border-[#1e1e24] shrink-0">
    <div class="inline-flex space-x-1">
      <button
        type="button"
        onclick={() => (subTab = "backup")}
        class="flex items-center space-x-1.5 px-3 py-1 rounded-md text-xs font-semibold transition-all cursor-pointer {subTab === 'backup'
          ? 'bg-sky-50 dark:bg-[#18181c] text-sky-600 dark:text-[#00c8ff] border border-sky-200 dark:border-[#00c8ff]/30 shadow-xs'
          : 'text-slate-600 dark:text-[#a1a1aa] hover:text-slate-900 dark:hover:text-white'}"
      >
        <Archive size={12} weight="bold" />
        <span>{t("subTabBackup")}</span>
      </button>

      <button
        type="button"
        onclick={() => (subTab = "import")}
        class="flex items-center space-x-1.5 px-3 py-1 rounded-md text-xs font-semibold transition-all cursor-pointer {subTab === 'import'
          ? 'bg-sky-50 dark:bg-[#18181c] text-sky-600 dark:text-[#00c8ff] border border-sky-200 dark:border-[#00c8ff]/30 shadow-xs'
          : 'text-slate-600 dark:text-[#a1a1aa] hover:text-slate-900 dark:hover:text-white'}"
      >
        <FileArrowUp size={12} weight="bold" />
        <span>{t("subTabImport")}</span>
      </button>
    </div>

    <span class="text-[11px] font-medium text-slate-500 dark:text-[#71717a] pr-2">
      {currentVersion}
    </span>
  </div>

  <!-- Mode 1: Sao lưu & Đóng gói Zip đa hồ sơ -->
  {#if subTab === "backup"}
    <div class="flex-1 bg-[#ffffff] dark:bg-[#0c0c0e] border border-[#e2e8f0] dark:border-[#1e1e24] rounded-lg p-3 flex flex-col overflow-hidden">
      <!-- Toolbar: Select all & Export button -->
      <div class="flex items-center justify-between pb-2 border-b border-[#e2e8f0] dark:border-[#1e1e24] shrink-0">
        <label class="flex items-center space-x-2 cursor-pointer font-semibold text-xs text-slate-700 dark:text-[#d4d4d8]">
          <input
            type="checkbox"
            checked={backupItems.length > 0 && backupItems.every((i) => i.selected)}
            onchange={(e) => toggleAllBackup(e.currentTarget.checked)}
            class="rounded accent-sky-500 w-3.5 h-3.5 cursor-pointer"
          />
          <span>{t("selectAll")} ({backupItems.filter((i) => i.selected).length}/{backupItems.length})</span>
        </label>

        <button
          type="button"
          onclick={handleExportBatchZip}
          disabled={backupItems.filter((i) => i.selected).length === 0}
          class="px-3.5 py-1.5 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-xs font-bold shadow-xs transition-colors flex items-center space-x-1.5 cursor-pointer disabled:opacity-50"
        >
          <FileArrowDown size={14} weight="bold" />
          <span>{t("exportBatchZipBtn")}</span>
        </button>
      </div>

      <!-- Profiles List -->
      <div class="flex-1 overflow-y-auto space-y-2 pt-2 pr-1">
        {#if backupItems.length === 0}
          <div class="h-full flex items-center justify-center text-xs text-slate-400 dark:text-[#71717a]">
            Chưa có hồ sơ nào trong phiên bản này.
          </div>
        {:else}
          {#each backupItems as item}
            {@const IconComponent = getProfileIcon(item.name)}
            {@const profileColor = getProfileColor(item.name)}
            {@const comp = appStore.profileComponents[item.name]}
            <div class="p-2.5 rounded-lg border transition-all {item.selected ? 'bg-[#f8fafc] dark:bg-[#121216] border-sky-500/30 dark:border-sky-500/20 shadow-2xs' : 'bg-slate-50/50 dark:bg-[#0e0e11] border-[#e2e8f0] dark:border-[#1a1a20] opacity-60'} space-y-2">
              <div class="flex items-center justify-between">
                <div class="flex items-center space-x-2.5">
                  <input
                    type="checkbox"
                    bind:checked={item.selected}
                    class="rounded accent-sky-500 w-3.5 h-3.5 cursor-pointer"
                  />
                  <div
                    class="w-6 h-6 rounded-md flex items-center justify-center shrink-0"
                    style="background-color: {profileColor}20; color: {profileColor};"
                  >
                    <IconComponent size={13} weight="duotone" />
                  </div>
                  <span class="text-xs font-bold text-slate-800 dark:text-white">{item.name}</span>
                  {#if item.name === activeProfile}
                    <span class="text-[9px] px-1.5 py-0.2 rounded-full font-bold bg-sky-500/10 text-sky-600 dark:text-[#00c8ff] border border-sky-500/20">
                      {t("activeLabel")}
                    </span>
                  {/if}
                </div>

                <!-- Counts Pills -->
                <div class="flex items-center space-x-2 text-[10px] text-slate-500 dark:text-[#a1a1aa]">
                  {#if comp}
                    <span class="bg-slate-100 dark:bg-[#18181c] px-1.5 py-0.5 rounded border border-slate-200 dark:border-[#222228]">
                      Plugins: {comp.plugins.length}
                    </span>
                    <span class="bg-slate-100 dark:bg-[#18181c] px-1.5 py-0.5 rounded border border-slate-200 dark:border-[#222228]">
                      Phím tắt: {comp.customShortcuts.length}
                    </span>
                  {/if}
                </div>
              </div>

              <!-- Component Checkboxes for this profile -->
              {#if item.selected}
                <div class="flex flex-wrap items-center gap-x-4 gap-y-1.5 pl-6 pt-1 text-[11px] text-slate-600 dark:text-slate-300 border-t border-slate-100 dark:border-[#1a1a20]">
                  <label class="inline-flex items-center gap-1.5 cursor-pointer">
                    <input type="checkbox" bind:checked={item.includePlugins} class="rounded accent-sky-500 w-3 h-3 cursor-pointer" />
                    <PuzzlePiece size={11} class="text-sky-500" />
                    <span>{t("tagPlugins")}</span>
                  </label>
                  <label class="inline-flex items-center gap-1.5 cursor-pointer">
                    <input type="checkbox" bind:checked={item.includeShortcuts} class="rounded accent-sky-500 w-3 h-3 cursor-pointer" />
                    <Keyboard size={11} class="text-sky-500" />
                    <span>{t("tagShortcuts")}</span>
                  </label>
                  <label class="inline-flex items-center gap-1.5 cursor-pointer">
                    <input type="checkbox" bind:checked={item.includeToolbars} class="rounded accent-sky-500 w-3 h-3 cursor-pointer" />
                    <Toolbox size={11} class="text-sky-500" />
                    <span>{t("tagToolbars")}</span>
                  </label>
                  <label class="inline-flex items-center gap-1.5 cursor-pointer">
                    <input type="checkbox" bind:checked={item.includeMaterials} class="rounded accent-sky-500 w-3 h-3 cursor-pointer" />
                    <Palette size={11} class="text-sky-500" />
                    <span>{t("tagMaterials")}</span>
                  </label>
                  <label class="inline-flex items-center gap-1.5 cursor-pointer">
                    <input type="checkbox" bind:checked={item.includeTemplates} class="rounded accent-sky-500 w-3 h-3 cursor-pointer" />
                    <FileText size={11} class="text-sky-500" />
                    <span>{t("tagTemplates")}</span>
                  </label>
                </div>
              {/if}
            </div>
          {/each}
        {/if}
      </div>
    </div>
  {:else}
    <!-- Mode 2: Nhập dữ liệu (Batch Import) -->
    <div class="flex-1 bg-[#ffffff] dark:bg-[#0c0c0e] border border-[#e2e8f0] dark:border-[#1e1e24] rounded-lg p-4 flex flex-col justify-center items-center text-center">
      <div class="max-w-md space-y-4">
        <div>
          <h3 class="text-xs font-bold text-slate-800 dark:text-white uppercase tracking-wider">{t("importHeading")}</h3>
          <p class="text-[11px] text-slate-500 dark:text-[#a1a1aa] mt-1 leading-relaxed">
            Chọn tệp sao lưu (.zip) hoặc thư mục chứa hồ sơ SketchUp. Hệ thống sẽ phân tích và hiển thị danh sách hồ sơ để bạn lựa chọn nhập.
          </p>
        </div>

        <div class="grid grid-cols-2 gap-3 pt-1">
          <button
            type="button"
            onclick={handleSelectFolder}
            class="p-4 bg-[#f8fafc] dark:bg-[#121215] hover:bg-[#f1f5f9] dark:hover:bg-[#18181c] border border-slate-300 dark:border-[#27272f] rounded-xl flex flex-col items-center justify-center space-y-2 cursor-pointer transition-colors"
          >
            <FolderOpen size={26} weight="duotone" class="text-sky-600 dark:text-[#00c8ff]" />
            <span class="text-xs font-bold text-slate-800 dark:text-[#f4f4f6]">{t("chooseFolderBtn")}</span>
          </button>

          <button
            type="button"
            onclick={handleSelectZip}
            class="p-4 bg-[#f8fafc] dark:bg-[#121215] hover:bg-[#f1f5f9] dark:hover:bg-[#18181c] border border-slate-300 dark:border-[#27272f] rounded-xl flex flex-col items-center justify-center space-y-2 cursor-pointer transition-colors"
          >
            <FileZip size={26} weight="duotone" class="text-sky-600 dark:text-[#00c8ff]" />
            <span class="text-xs font-bold text-slate-800 dark:text-[#f4f4f6]">{t("chooseZipBtn")}</span>
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>
