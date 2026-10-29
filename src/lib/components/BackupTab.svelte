<script>
  import { appStore, saveProfile, setBusy, t, loadProfileComponents } from "../store.svelte.js";
  import { save, open } from "@tauri-apps/plugin-dialog";
  import { invoke } from "@tauri-apps/api/core";
  import CustomCheckbox from "./CustomCheckbox.svelte";
  import CustomDropdown from "./CustomDropdown.svelte";
  import {
    FloppyDisk,
    Warning,
    Archive,
    FileArrowDown,
    FileArrowUp,
    FolderOpen,
    CheckCircle,
    FileZip,
    ArrowRight,
    PuzzlePiece,
    Keyboard,
    Toolbox,
    Palette,
    FileText,
  } from "phosphor-svelte";

  let subTab = $state("backup"); // 'backup' | 'import'

  // Backup state
  let newProfileName = $state("");
  let savePlugins = $state(true);
  let saveShortcuts = $state(true);

  // Import state
  let importPath = $state("");
  let isAnalyzing = $state(false);
  let analysis = $state(null);
  let importVersion = $state("");
  let importProfileName = $state("");
  let importPlugins = $state(true);
  let importShortcuts = $state(true);
  let importToolbars = $state(true);
  let importMaterials = $state(false);
  let importTemplates = $state(false);
  let isImporting = $state(false);

  const currentVersion = $derived(appStore.state.selectedVersion || "SketchUp");
  const matchingProfile = $derived(appStore.matchingProfile);

  $effect(() => {
    const now = new Date();
    const mm = String(now.getMonth() + 1).padStart(2, "0");
    const dd = String(now.getDate()).padStart(2, "0");
    const hh = String(now.getHours()).padStart(2, "0");
    const min = String(now.getMinutes()).padStart(2, "0");
    newProfileName = `Profile_${mm}${dd}_${hh}${min}`;
  });

  async function handleSave() {
    if (!newProfileName.trim()) {
      alert("Vui lòng nhập tên.");
      return;
    }
    if (!savePlugins && !saveShortcuts) {
      alert("Chọn ít nhất một mục.");
      return;
    }

    await saveProfile(newProfileName.trim(), savePlugins, saveShortcuts);
    appStore.activeTab = "profiles";
  }

  async function handleExportZip() {
    const cfg = appStore.state.versions[currentVersion];
    const active = cfg ? cfg.activeProfile : "Default";
    const dest = await save({
      filters: [{ name: "Zip", extensions: ["zip"] }],
      defaultPath: `SketchUp_${currentVersion.replace(/\s+/g, "_")}_${active}.zip`,
    });

    if (dest) {
      setBusy(true, "Đang đóng gói tệp Zip...");
      try {
        await invoke("export_profile_zip", {
          version: currentVersion,
          profileName: active,
          zipPath: dest,
        });
        alert("Đã xuất zip thành công!");
      } catch (e) {
        alert(String(e));
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
        title: "Chọn thư mục dữ liệu SketchUp",
      });
      if (selected) {
        await runAnalysis(selected);
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
        await runAnalysis(selected);
      }
    } catch (e) {
      alert("Lỗi mở file zip: " + String(e));
    }
  }

  async function runAnalysis(sourcePath) {
    importPath = sourcePath;
    isAnalyzing = true;
    analysis = null;
    setBusy(true, "Đang quét và phân tích dữ liệu...");
    try {
      const res = await invoke("analyze_import_source", { sourcePath });
      analysis = res;
      importVersion = res.detectedVersion || currentVersion;
      importProfileName = res.suggestedProfileName || "Imported_Profile";
      importPlugins = res.hasPlugins;
      importShortcuts = res.hasShortcuts;
      importToolbars = res.hasToolbars;
      importMaterials = res.hasMaterials;
      importTemplates = res.hasTemplates;
    } catch (e) {
      alert("Lỗi phân tích nguồn: " + String(e));
    } finally {
      isAnalyzing = false;
      setBusy(false);
    }
  }

  async function handleExecuteImport() {
    if (!analysis || !importPath) return;
    if (!importProfileName.trim()) {
      alert("Vui lòng đặt tên cho hồ sơ nhập.");
      return;
    }
    isImporting = true;
    setBusy(true, "Đang nhập dữ liệu cấu hình...");
    try {
      const s = await invoke("import_profile", {
        version: importVersion || currentVersion,
        profileName: importProfileName.trim(),
        sourcePath: importPath,
        importPlugins,
        importShortcuts,
        importToolbars,
        importMaterials,
        importTemplates,
        applyImmediately: false,
      });
      appStore.state = s;
      await loadProfileComponents();
      appStore.modal = {
        type: "confirm_apply_imported",
        profileName: importProfileName.trim(),
        version: importVersion || currentVersion,
      };
    } catch (e) {
      alert("Lỗi nhập hồ sơ: " + String(e));
    } finally {
      isImporting = false;
      setBusy(false);
    }
  }
</script>

<div class="space-y-2 h-[255px] flex flex-col justify-between">
  <!-- Sub-navigation Toggle -->
  <div class="flex items-center justify-between bg-[#ffffff] dark:bg-[#0c0c0e] p-1 rounded-lg border border-[#e2e8f0] dark:border-[#1e1e24]">
    <div class="inline-flex space-x-1">
      <button
        onclick={() => (subTab = "backup")}
        class="flex items-center space-x-1.5 px-3 py-1 rounded-md text-xs font-semibold transition-all {subTab === 'backup'
          ? 'bg-sky-50 dark:bg-[#18181c] text-sky-600 dark:text-[#00c8ff] border border-sky-200 dark:border-[#00c8ff]/30 shadow-xs'
          : 'text-slate-600 dark:text-[#a1a1aa] hover:text-slate-900 dark:hover:text-white'}"
      >
        <FloppyDisk size={12} weight="bold" />
        <span>{t("subTabBackup")}</span>
      </button>

      <button
        onclick={() => (subTab = "import")}
        class="flex items-center space-x-1.5 px-3 py-1 rounded-md text-xs font-semibold transition-all {subTab === 'import'
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

  <!-- Mode 1: Sao lưu & Đóng gói Zip -->
  {#if subTab === "backup"}
    <div class="grid grid-cols-2 gap-2.5 flex-1">
      <!-- Left Column: Lưu cấu hình -->
      <div class="bg-[#ffffff] dark:bg-[#0c0c0e] border border-[#e2e8f0] dark:border-[#1e1e24] rounded-lg p-3 flex flex-col justify-between">
        <div class="space-y-2">
          <div class="flex items-center space-x-2 pb-1.5 border-b border-[#e2e8f0] dark:border-[#1a1a20]">
            <FloppyDisk size={14} weight="bold" class="text-sky-600 dark:text-[#00c8ff]" />
            <span class="text-xs font-bold text-slate-800 dark:text-[#fafafa]">{t("saveHeading")}</span>
          </div>

          <!-- Profile Name Input & Inline Duplicate Warning -->
          <div>
            <div class="flex items-center justify-between mb-1">
              <label class="text-[11px] font-semibold text-slate-700 dark:text-[#a1a1aa]" for="newProfName">
                {t("profileNameLabel")}
              </label>
              {#if matchingProfile}
                <span class="inline-flex items-center space-x-1 text-[10px] font-semibold text-amber-600 dark:text-amber-400 bg-amber-500/10 px-1.5 py-0.5 rounded-full border border-amber-500/20">
                  <Warning size={11} weight="fill" class="shrink-0" />
                  <span>Trùng: {matchingProfile}</span>
                </span>
              {/if}
            </div>
            <input
              id="newProfName"
              type="text"
              bind:value={newProfileName}
              class="w-full bg-[#f8fafc] dark:bg-[#141417] border border-slate-300 dark:border-[#27272f] rounded-lg px-2.5 py-1.5 text-xs text-slate-900 dark:text-[#ffffff] focus:outline-none focus:border-sky-500 dark:focus:border-[#00c8ff]"
            />
          </div>

          <!-- Selective Checkboxes (Horizontal & Circular with Icons) -->
          <div class="flex flex-wrap items-center gap-x-4 gap-y-2 pt-1">
            <CustomCheckbox bind:checked={savePlugins} label={t("savePluginsLabel")} icon={PuzzlePiece} />
            <CustomCheckbox bind:checked={saveShortcuts} label={t("saveShortcutsLabel")} icon={Keyboard} />
          </div>
        </div>

        <!-- Sticky Bottom Bar (Button aligned to the right) -->
        <div class="pt-2 border-t border-slate-200 dark:border-[#1a1a20] flex items-center justify-end mt-auto">
          <button
            onclick={handleSave}
            class="px-4 py-1.5 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-xs font-bold shadow-xs transition-colors flex items-center space-x-1 cursor-pointer"
          >
            <CheckCircle size={13} weight="bold" />
            <span>{t("saveProfileBtn")}</span>
          </button>
        </div>
      </div>

      <!-- Right Column: Đóng gói Zip -->
      <div class="bg-[#ffffff] dark:bg-[#0c0c0e] border border-[#e2e8f0] dark:border-[#1e1e24] rounded-lg p-3 flex flex-col justify-between">
        <div class="space-y-2">
          <div class="flex items-center space-x-2 pb-1.5 border-b border-[#e2e8f0] dark:border-[#1a1a20]">
            <Archive size={14} weight="bold" class="text-sky-600 dark:text-[#00c8ff]" />
            <span class="text-xs font-bold text-slate-800 dark:text-[#fafafa]">{t("exportHeading")}</span>
          </div>

          <div class="bg-[#f8fafc] dark:bg-[#141417] p-2.5 rounded-lg border border-[#e2e8f0] dark:border-[#27272f] text-xs text-slate-600 dark:text-[#a1a1aa] space-y-1.5">
            <div class="flex justify-between">
              <span>{t("targetVersionLabel")}</span>
              <span class="text-slate-800 dark:text-white font-semibold">{currentVersion}</span>
            </div>
            <div class="flex justify-between">
              <span>{t("exportProfileLabel")}</span>
              <span class="text-sky-600 dark:text-[#00c8ff] font-bold">{appStore.state.versions[currentVersion]?.activeProfile || "Default"}</span>
            </div>
          </div>
        </div>

        <!-- Sticky Bottom Bar (Button aligned to the right) -->
        <div class="pt-2 border-t border-slate-200 dark:border-[#1a1a20] flex items-center justify-end mt-auto">
          <button
            onclick={handleExportZip}
            class="px-4 py-1.5 bg-slate-100 hover:bg-slate-200 dark:bg-[#18181c] dark:hover:bg-[#232329] text-slate-800 dark:text-[#f4f4f6] rounded-lg text-xs font-bold border border-slate-300 dark:border-[#27272f] transition-colors flex items-center space-x-1.5 cursor-pointer"
          >
            <FileArrowDown size={14} weight="bold" class="text-sky-600 dark:text-[#00c8ff]" />
            <span>{t("exportZipBtn")}</span>
          </button>
        </div>
      </div>
    </div>
  {:else}
    <!-- Mode 2: Nhập thư mục hoặc Zip -->
    <div class="bg-[#ffffff] dark:bg-[#0c0c0e] border border-[#e2e8f0] dark:border-[#1e1e24] rounded-lg p-3 flex-1 flex flex-col justify-between relative">
      {#if !analysis}
        <!-- Step 1: Browse Buttons -->
        <div class="space-y-3">
          <div class="flex items-center space-x-2 pb-1.5 border-b border-[#e2e8f0] dark:border-[#1a1a20]">
            <FileArrowUp size={14} weight="bold" class="text-sky-600 dark:text-[#00c8ff]" />
            <span class="text-xs font-bold text-slate-800 dark:text-[#fafafa]">{t("importHeading")}</span>
          </div>

          <div class="grid grid-cols-2 gap-2.5 pt-1">
            <button
              onclick={handleSelectFolder}
              class="p-4 bg-[#f8fafc] dark:bg-[#121215] hover:bg-[#f1f5f9] dark:hover:bg-[#18181c] border border-slate-300 dark:border-[#27272f] rounded-lg flex flex-col items-center justify-center space-y-2 cursor-pointer transition-colors"
            >
              <FolderOpen size={24} weight="duotone" class="text-sky-600 dark:text-[#00c8ff]" />
              <span class="text-xs font-bold text-slate-800 dark:text-[#f4f4f6]">{t("chooseFolderBtn")}</span>
            </button>

            <button
              onclick={handleSelectZip}
              class="p-4 bg-[#f8fafc] dark:bg-[#121215] hover:bg-[#f1f5f9] dark:hover:bg-[#18181c] border border-slate-300 dark:border-[#27272f] rounded-lg flex flex-col items-center justify-center space-y-2 cursor-pointer transition-colors"
            >
              <FileZip size={24} weight="duotone" class="text-sky-600 dark:text-[#00c8ff]" />
              <span class="text-xs font-bold text-slate-800 dark:text-[#f4f4f6]">{t("chooseZipBtn")}</span>
            </button>
          </div>

          {#if isAnalyzing}
            <div class="text-center text-xs font-medium text-sky-600 dark:text-[#00c8ff] animate-pulse">
              Đang phân tích cấu hình...
            </div>
          {/if}
        </div>
      {:else}
        <!-- Step 2: Analysis Results (No scrollbars, clean layout) -->
        <div class="space-y-2.5 flex-1 relative">
          <div class="flex items-center justify-between pb-1.5 border-b border-[#e2e8f0] dark:border-[#1a1a20]">
            <div class="flex items-center space-x-2">
              <FileArrowUp size={14} weight="bold" class="text-sky-600 dark:text-[#00c8ff]" />
              <span class="text-xs font-bold text-slate-800 dark:text-[#fafafa]">{t("detectedHeading")}</span>
            </div>
            <button
              onclick={() => (analysis = null)}
              class="text-[11px] font-semibold text-slate-500 hover:text-slate-800 dark:text-[#a1a1aa] dark:hover:text-white underline cursor-pointer"
            >
              Chọn lại
            </button>
          </div>

          <!-- Configuration Fields -->
          <div class="grid grid-cols-2 gap-2.5 text-xs">
            <div>
              <label class="block text-[10px] font-bold text-slate-600 dark:text-[#a1a1aa] mb-1" for="importProfName">
                {t("profileNameLabel")}
              </label>
              <input
                id="importProfName"
                type="text"
                bind:value={importProfileName}
                class="w-full bg-[#f8fafc] dark:bg-[#141417] border border-slate-300 dark:border-[#27272f] rounded-lg px-2.5 py-1.5 text-xs text-slate-900 dark:text-[#ffffff] focus:outline-none focus:border-sky-500 dark:focus:border-[#00c8ff]"
              />
            </div>

            <div class="relative z-30">
              <span class="block text-[10px] font-bold text-slate-600 dark:text-[#a1a1aa] mb-1">
                {t("targetVersionLabel")}
              </span>
              <CustomDropdown
                options={appStore.installedVersions}
                value={importVersion}
                onchange={(v) => (importVersion = v)}
              />
            </div>
          </div>

          <!-- Detected Components Toggles (Horizontal & Circular with Icons) -->
          <div class="flex flex-wrap items-center gap-x-3 gap-y-2 pt-1">
            {#if analysis.hasPlugins}
              <CustomCheckbox bind:checked={importPlugins} label={`Plugins (${analysis.pluginCount})`} icon={PuzzlePiece} />
            {/if}
            {#if analysis.hasShortcuts}
              <CustomCheckbox bind:checked={importShortcuts} label={t("tagShortcuts")} icon={Keyboard} />
            {/if}
            {#if analysis.hasToolbars}
              <CustomCheckbox bind:checked={importToolbars} label={t("tagToolbars")} icon={Toolbox} />
            {/if}
            {#if analysis.hasMaterials}
              <CustomCheckbox bind:checked={importMaterials} label={t("tagMaterials")} icon={Palette} />
            {/if}
            {#if analysis.hasTemplates}
              <CustomCheckbox bind:checked={importTemplates} label={t("tagTemplates")} icon={FileText} />
            {/if}
          </div>
        </div>

        <!-- Sticky Bottom Bar (Button aligned to the right) -->
        <div class="pt-2 border-t border-slate-200 dark:border-[#1a1a20] flex items-center justify-end mt-auto">
          <button
            onclick={handleExecuteImport}
            disabled={isImporting}
            class="px-4 py-1.5 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-xs font-bold shadow-xs transition-colors flex items-center space-x-1.5 cursor-pointer disabled:opacity-50"
          >
            <ArrowRight size={13} weight="bold" />
            <span>{isImporting ? "Đang nhập..." : t("startImportBtn")}</span>
          </button>
        </div>
      {/if}
    </div>
  {/if}
</div>
