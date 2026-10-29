<script>
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { appStore, initAppStore, saveProfileCustom, t } from "../store.svelte.js";
  import CustomCheckbox from "./CustomCheckbox.svelte";
  import {
    X,
    FloppyDisk,
    PuzzlePiece,
    Keyboard,
    Toolbox,
    Palette,
    FileText,
    CheckCircle,
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

  const appWindow = getCurrentWindow();

  const availableIcons = [
    { name: "Folder", icon: Folder },
    { name: "Briefcase", icon: Briefcase },
    { name: "PaintBrush", icon: PaintBrush },
    { name: "Cube", icon: Cube },
    { name: "Compass", icon: Compass },
    { name: "Lightning", icon: Lightning },
    { name: "Star", icon: Star },
    { name: "Sparkle", icon: Sparkle },
    { name: "Cpu", icon: Cpu },
    { name: "GameController", icon: GameController },
  ];

  const presetColors = [
    "#00c8ff",
    "#10b981",
    "#f59e0b",
    "#ef4444",
    "#a855f7",
    "#64748b",
  ];

  let saveName = $state("");
  let selectedIcon = $state("Folder");
  let selectedColor = $state("#00c8ff");
  let savePlugins = $state(true);
  let saveShortcuts = $state(true);
  let saveToolbars = $state(false);
  let saveMaterials = $state(false);
  let saveTemplates = $state(false);
  let isSaving = $state(false);
  let ready = $state(false);

  const currentVersion = $derived(appStore.state.selectedVersion || "SketchUp");

  onMount(async () => {
    await initAppStore();
    ready = true;
    const now = new Date();
    const mm = String(now.getMonth() + 1).padStart(2, "0");
    const dd = String(now.getDate()).padStart(2, "0");
    const hh = String(now.getHours()).padStart(2, "0");
    const min = String(now.getMinutes()).padStart(2, "0");
    saveName = `Profile_${mm}${dd}_${hh}${min}`;
  });

  async function handleClose() {
    await appWindow.close();
  }

  function handleKeydown(e) {
    if (e.key === "Escape") {
      handleClose();
    }
  }

  async function handleSave() {
    if (!saveName.trim() || isSaving) return;
    isSaving = true;
    try {
      await saveProfileCustom(
        saveName.trim(),
        savePlugins,
        saveShortcuts,
        saveToolbars,
        saveMaterials,
        saveTemplates,
        selectedIcon,
        selectedColor
      );
      await appWindow.close();
    } catch (e) {
      alert("Lỗi lưu hồ sơ: " + String(e));
    } finally {
      isSaving = false;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="h-screen w-screen flex flex-col justify-between bg-[#ffffff] dark:bg-[#09090b] text-[#0f172a] dark:text-[#fafafa] font-sans border border-[#cbd5e1] dark:border-[#222228] select-none p-3.5 overflow-hidden shadow-2xl">
  <!-- Draggable Header -->
  <div data-tauri-drag-region class="flex items-center justify-between pb-2 border-b border-[#e2e8f0] dark:border-[#1e1e24] cursor-grab active:cursor-grabbing">
    <div class="flex items-center space-x-2 pointer-events-none">
      <FloppyDisk size={14} weight="bold" class="text-[#0284c7] dark:text-[#00c8ff]" />
      <span class="text-xs font-bold">{t("saveHeading")}</span>
      <span class="text-[10px] text-[#64748b] dark:text-[#a1a1aa] bg-[#f1f5f9] dark:bg-[#151518] px-1.5 py-0.5 rounded border border-[#cbd5e1] dark:border-[#222228]">
        {currentVersion}
      </span>
    </div>
    <button
      onclick={handleClose}
      class="w-6 h-6 flex items-center justify-center rounded hover:bg-[#fee2e2] dark:hover:bg-[#ef4444]/20 text-[#64748b] dark:text-[#a1a1aa] hover:text-[#ef4444] transition-colors cursor-pointer"
      title="Đóng"
    >
      <X size={13} weight="bold" />
    </button>
  </div>

  <!-- Form Fields -->
  <div class="space-y-2.5 py-1 flex-1 overflow-y-auto">
    <div>
      <label class="block text-[11px] font-semibold text-[#64748b] dark:text-[#a1a1aa] mb-1" for="quickSaveName">
        {t("profileNameLabel")}
      </label>
      <input
        id="quickSaveName"
        type="text"
        bind:value={saveName}
        class="w-full bg-[#f8fafc] dark:bg-[#141417] border border-[#cbd5e1] dark:border-[#2b2b34] rounded-lg px-2.5 py-1.5 text-xs text-[#0f172a] dark:text-[#ffffff] focus:outline-none focus:border-[#00c8ff]"
      />
    </div>

    <!-- Icon Selection -->
    <div>
      <span class="block text-[11px] font-semibold text-[#64748b] dark:text-[#a1a1aa] mb-1">
        {t("iconLabel")}
      </span>
      <div class="flex items-center gap-1.5 flex-wrap">
        {#each availableIcons as item}
          <button
            type="button"
            onclick={() => (selectedIcon = item.name)}
            class="w-7 h-7 rounded-lg flex items-center justify-center transition-all cursor-pointer {selectedIcon === item.name
              ? 'bg-sky-500/20 text-sky-600 dark:text-[#00c8ff] ring-2 ring-sky-500'
              : 'bg-slate-100 dark:bg-[#16161b] text-slate-600 dark:text-[#a1a1aa] hover:bg-slate-200 dark:hover:bg-[#202026]'}"
            title={item.name}
          >
            <item.icon size={14} weight={selectedIcon === item.name ? 'fill' : 'regular'} />
          </button>
        {/each}
      </div>
    </div>

    <!-- Color Palette -->
    <div>
      <span class="block text-[11px] font-semibold text-[#64748b] dark:text-[#a1a1aa] mb-1">
        {t("colorLabel")}
      </span>
      <div class="flex items-center space-x-2">
        {#each presetColors as col}
          <button
            type="button"
            onclick={() => (selectedColor = col)}
            class="w-6 h-6 rounded-full transition-transform cursor-pointer flex items-center justify-center {selectedColor === col ? 'scale-110 ring-2 ring-white/60 shadow-xs' : 'hover:scale-105'}"
            style="background-color: {col};"
          >
            {#if selectedColor === col}
              <CheckCircle size={12} weight="fill" class="text-white" />
            {/if}
          </button>
        {/each}
      </div>
    </div>

    <!-- Checkboxes (2 columns with Icons) -->
    <div class="grid grid-cols-2 gap-2 pt-1 border-t border-[#e2e8f0] dark:border-[#1e1e24]">
      <CustomCheckbox bind:checked={savePlugins} label={t("savePluginsLabel")} icon={PuzzlePiece} />
      <CustomCheckbox bind:checked={saveShortcuts} label={t("saveShortcutsLabel")} icon={Keyboard} />
      <CustomCheckbox bind:checked={saveToolbars} label={t("saveToolbarsLabel")} icon={Toolbox} />
      <CustomCheckbox bind:checked={saveMaterials} label={t("saveMaterialsLabel")} icon={Palette} />
      <CustomCheckbox bind:checked={saveTemplates} label={t("saveTemplatesLabel")} icon={FileText} />
    </div>
  </div>

  <!-- Primary Action Only (Single Dismissal Rule) -->
  <div class="flex items-center justify-end pt-2 border-t border-[#e2e8f0] dark:border-[#1e1e24]">
    <button
      onclick={handleSave}
      disabled={isSaving || !ready || !saveName.trim()}
      class="w-full py-2 bg-sky-600 hover:bg-sky-500 disabled:opacity-40 text-white rounded-lg text-xs font-bold shadow-xs cursor-pointer transition-colors"
    >
      {isSaving ? "Đang lưu hồ sơ..." : t("saveProfileBtn")}
    </button>
  </div>
</div>
