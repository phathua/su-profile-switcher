<script>
  import {
    appStore,
    renameProfile,
    deleteProfile,
    updateProfileMetadata,
    saveProfileCustom,
    switchProfile,
    t,
  } from "../store.svelte.js";
  import { invoke } from "@tauri-apps/api/core";
  import CustomCheckbox from "./CustomCheckbox.svelte";
  import {
    X,
    Warning,
    Trash,
    CloudArrowDown,
    Plus,
    PencilSimple,
    CheckCircle,
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
    Command,
  } from "phosphor-svelte";

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

  let renameValue = $state("");
  let selectedIcon = $state("Folder");
  let selectedColor = $state("#00c8ff");
  let shortcutValue = $state("");
  let isRecordingShortcut = $state(false);
  let newProfileName = $state("");
  let savePlugins = $state(true);
  let saveShortcuts = $state(true);
  let saveToolbars = $state(false);
  let saveMaterials = $state(false);
  let saveTemplates = $state(false);
  let downloadingUpdate = $state(false);

  $effect(() => {
    if (appStore.modal) {
      if (appStore.modal.type === "rename") {
        renameValue = appStore.modal.oldName;
        const curVer = appStore.state.selectedVersion;
        const verCfg = appStore.state.versions[curVer] || {};
        const meta = (verCfg.profilesMetadata && verCfg.profilesMetadata[appStore.modal.oldName]) || {};
        selectedIcon = meta.icon || "Folder";
        selectedColor = meta.color || "#00c8ff";
        shortcutValue = meta.shortcut || "";
        isRecordingShortcut = false;
      } else if (appStore.modal.type === "create_profile" || appStore.modal.type === "save") {
        const now = new Date();
        const mm = String(now.getMonth() + 1).padStart(2, "0");
        const dd = String(now.getDate()).padStart(2, "0");
        const hh = String(now.getHours()).padStart(2, "0");
        const min = String(now.getMinutes()).padStart(2, "0");
        newProfileName = `Profile_${mm}${dd}_${hh}${min}`;
        savePlugins = true;
        saveShortcuts = true;
        saveToolbars = false;
        saveMaterials = false;
        saveTemplates = false;
      }
    }
  });

  function closeModal() {
    appStore.modal = null;
  }

  async function handleRenameSubmit() {
    if (!renameValue.trim()) return;
    const oldName = appStore.modal.oldName;
    const newName = renameValue.trim();
    if (oldName !== newName) {
      await renameProfile(oldName, newName);
    }
    await updateProfileMetadata(
      newName,
      selectedIcon,
      selectedColor,
      shortcutValue.trim() || null
    );
    closeModal();
  }

  function handleKeyDownShortcut(e) {
    if (!isRecordingShortcut) return;
    e.preventDefault();
    if (["Control", "Shift", "Alt", "Meta"].includes(e.key)) return;

    let parts = [];
    if (e.ctrlKey) parts.push("Ctrl");
    if (e.altKey) parts.push("Alt");
    if (e.shiftKey) parts.push("Shift");

    let key = e.key.toUpperCase();
    if (key === "ESCAPE") {
      shortcutValue = "";
      isRecordingShortcut = false;
      return;
    }
    parts.push(key);
    shortcutValue = parts.join("+");
    isRecordingShortcut = false;
  }

  async function handleCreateProfileSubmit() {
    if (!newProfileName.trim()) return;
    await saveProfileCustom(
      newProfileName.trim(),
      savePlugins,
      saveShortcuts,
      saveToolbars,
      saveMaterials,
      saveTemplates
    );
    closeModal();
  }

  async function handleConfirmSwitchCloseSketchUp() {
    const target = appStore.modal.targetProfile;
    await invoke("close_sketchup");
    await switchProfile(target);
    closeModal();
  }

  async function handleUpdateDoNotRemind() {
    const info = appStore.modal.info;
    await invoke("update_settings", {
      autoCheckUpdates: appStore.state.autoCheckUpdates,
      excludeRender: appStore.state.excludeRenderAndDefaultPlugins,
      language: appStore.language,
      theme: appStore.theme,
      ignoredUpdateVersion: info.latestVersion,
      remindLater: false,
    });
    closeModal();
  }

  async function handleUpdateRemindLater() {
    await invoke("update_settings", {
      autoCheckUpdates: appStore.state.autoCheckUpdates,
      excludeRender: appStore.state.excludeRenderAndDefaultPlugins,
      language: appStore.language,
      theme: appStore.theme,
      ignoredUpdateVersion: null,
      remindLater: true,
    });
    closeModal();
  }

  async function handleUpdateBackground() {
    const info = appStore.modal.info;
    downloadingUpdate = true;
    try {
      await invoke("download_update", {
        url: info.downloadUrl,
        latestVersion: info.latestVersion,
      });
      alert("Đã tải bản cập nhật ngầm!");
    } catch (e) {
      alert("Lỗi tải: " + String(e));
    } finally {
      downloadingUpdate = false;
      closeModal();
    }
  }
</script>

{#if appStore.modal}
  <!-- Overlay Backdrop -->
  <div class="fixed inset-0 bg-black/70 backdrop-blur-xs z-50 flex items-center justify-center p-4">
    <!-- Modal Container -->
    <div class="bg-[#ffffff] dark:bg-[#0a0a0d] border border-[#cbd5e1] dark:border-[#222228] rounded-xl shadow-2xl w-full max-w-sm overflow-hidden animate-in fade-in duration-100">
      
      <!-- 1. Create Profile Modal -->
      {#if appStore.modal.type === "create_profile" || appStore.modal.type === "save"}
        <div class="p-4 space-y-3">
          <div class="flex items-center justify-between pb-2 border-b border-[#e2e8f0] dark:border-[#1e1e24]">
            <div class="flex items-center space-x-2">
              <Plus size={15} weight="bold" class="text-[#0284c7] dark:text-[#00c8ff]" />
              <span class="text-xs font-bold text-[#0f172a] dark:text-[#fafafa]">{t("createProfileHeading")}</span>
            </div>
            <button onclick={closeModal} class="text-[#64748b] dark:text-[#a1a1aa] hover:text-[#0f172a] dark:hover:text-white cursor-pointer">
              <X size={14} weight="bold" />
            </button>
          </div>

          <div class="space-y-2">
            <div>
              <label class="block text-[11px] font-semibold text-[#64748b] dark:text-[#a1a1aa] mb-1" for="modalProfName">
                {t("profileNameLabel")}
              </label>
              <input
                id="modalProfName"
                type="text"
                bind:value={newProfileName}
                class="w-full bg-[#f8fafc] dark:bg-[#141417] border border-[#cbd5e1] dark:border-[#2b2b34] rounded-lg px-2.5 py-1.5 text-xs text-[#0f172a] dark:text-[#ffffff] focus:outline-none focus:border-[#00c8ff]"
              />
            </div>

            <!-- Custom Rounded Checkboxes with Icons -->
            <div class="grid grid-cols-2 gap-2 pt-1">
              <CustomCheckbox bind:checked={savePlugins} label={t("savePluginsLabel")} icon={PuzzlePiece} />
              <CustomCheckbox bind:checked={saveShortcuts} label={t("saveShortcutsLabel")} icon={Keyboard} />
              <CustomCheckbox bind:checked={saveToolbars} label={t("saveToolbarsLabel")} icon={Toolbox} />
              <CustomCheckbox bind:checked={saveMaterials} label={t("saveMaterialsLabel")} icon={Palette} />
              <CustomCheckbox bind:checked={saveTemplates} label={t("saveTemplatesLabel")} icon={FileText} />
            </div>
          </div>

          <div class="flex items-center justify-end space-x-2 pt-2 border-t border-[#e2e8f0] dark:border-[#1e1e24]">
            <button
              onclick={closeModal}
              class="px-3 py-1.5 bg-[#f1f5f9] dark:bg-[#151518] hover:bg-[#e2e8f0] dark:hover:bg-[#202026] text-[#64748b] dark:text-[#a1a1aa] rounded-lg text-xs font-semibold cursor-pointer"
            >
              {t("cancelBtn")}
            </button>
            <button
              onclick={handleCreateProfileSubmit}
              class="px-3 py-1.5 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-xs font-bold shadow-xs cursor-pointer"
            >
              {t("createBtn")}
            </button>
          </div>
        </div>

      <!-- 2. Customize Profile Modal (Rename + Icon + Color + Shortcut) -->
      {:else if appStore.modal.type === "rename"}
        <div class="p-4 space-y-3">
          <div class="flex items-center justify-between pb-2 border-b border-[#e2e8f0] dark:border-[#1e1e24]">
            <div class="flex items-center space-x-2">
              <PencilSimple size={15} weight="bold" class="text-[#0284c7] dark:text-[#00c8ff]" />
              <span class="text-xs font-bold text-[#0f172a] dark:text-[#fafafa]">{t("editProfileHeading")}</span>
            </div>
            <button onclick={closeModal} class="text-[#64748b] dark:text-[#a1a1aa] hover:text-[#0f172a] dark:hover:text-white cursor-pointer">
              <X size={14} weight="bold" />
            </button>
          </div>

          <div class="space-y-2.5">
            <div>
              <label class="block text-[11px] font-semibold text-[#64748b] dark:text-[#a1a1aa] mb-1" for="renameInput">
                {t("profileNameLabel")}
              </label>
              <input
                id="renameInput"
                type="text"
                bind:value={renameValue}
                class="w-full bg-[#f8fafc] dark:bg-[#141417] border border-[#cbd5e1] dark:border-[#2b2b34] rounded-lg px-2.5 py-1.5 text-xs text-[#0f172a] dark:text-[#ffffff] focus:outline-none focus:border-[#00c8ff]"
              />
            </div>

            <!-- Icon Picker -->
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

            <!-- Shortcut Recorder -->
            <div>
              <label class="block text-[11px] font-semibold text-[#64748b] dark:text-[#a1a1aa] mb-1" for="shortcutBtn">
                {t("shortcutLabel")}
              </label>
              <div class="flex items-center space-x-2">
                <button
                  id="shortcutBtn"
                  type="button"
                  onkeydown={handleKeyDownShortcut}
                  onclick={() => (isRecordingShortcut = true)}
                  class="flex-1 text-left px-2.5 py-1.5 rounded-lg border text-xs font-mono transition-colors cursor-pointer {isRecordingShortcut
                    ? 'border-sky-500 bg-sky-500/10 text-sky-600 dark:text-[#00c8ff] animate-pulse'
                    : 'border-[#cbd5e1] dark:border-[#2b2b34] bg-[#f8fafc] dark:bg-[#141417] text-[#0f172a] dark:text-[#ffffff]'}"
                >
                  {#if isRecordingShortcut}
                    Bấm tổ hợp phím (ví dụ: Ctrl+Alt+1)...
                  {:else if shortcutValue}
                    {shortcutValue}
                  {:else}
                    <span class="text-slate-400 dark:text-[#71717a] font-sans">Bấm để gán phím tắt</span>
                  {/if}
                </button>
                {#if shortcutValue}
                  <button
                    type="button"
                    onclick={() => (shortcutValue = "")}
                    class="p-1 text-slate-400 hover:text-rose-500 cursor-pointer"
                    title="Xóa phím tắt"
                  >
                    <X size={13} weight="bold" />
                  </button>
                {/if}
              </div>
            </div>
          </div>

          <div class="flex items-center justify-end space-x-2 pt-2 border-t border-[#e2e8f0] dark:border-[#1e1e24]">
            <button
              onclick={closeModal}
              class="px-3 py-1.5 bg-[#f1f5f9] dark:bg-[#151518] hover:bg-[#e2e8f0] dark:hover:bg-[#202026] text-[#64748b] dark:text-[#a1a1aa] rounded-lg text-xs font-semibold cursor-pointer"
            >
              {t("cancelBtn")}
            </button>
            <button
              onclick={handleRenameSubmit}
              class="px-3 py-1.5 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-xs font-bold shadow-xs cursor-pointer"
            >
              {t("saveEditBtn")}
            </button>
          </div>
        </div>

      <!-- 3. Confirm Switch & Close SketchUp -->
      {:else if appStore.modal.type === "confirm_switch"}
        <div class="p-4 space-y-3">
          <div class="flex items-center space-x-2.5 text-[#f59e0b]">
            <Warning size={20} weight="fill" />
            <span class="text-xs font-bold text-[#0f172a] dark:text-[#fafafa]">{t("confirmSwitchHeading")}</span>
          </div>

          <p class="text-xs text-[#64748b] dark:text-[#a1a1aa] leading-relaxed">
            {t("confirmSwitchMsg")}
          </p>

          <div class="flex items-center justify-end space-x-2 pt-2 border-t border-[#e2e8f0] dark:border-[#1e1e24]">
            <button
              onclick={closeModal}
              class="px-3 py-1.5 bg-[#f1f5f9] dark:bg-[#151518] hover:bg-[#e2e8f0] dark:hover:bg-[#202026] text-[#64748b] dark:text-[#a1a1aa] rounded-lg text-xs font-semibold cursor-pointer"
            >
              {t("cancelSwitchBtn")}
            </button>
            <button
              onclick={async () => {
                const target = appStore.modal.targetProfile;
                closeModal();
                await switchProfile(target);
              }}
              class="px-3 py-1.5 bg-slate-200 dark:bg-[#1f1f26] hover:bg-slate-300 dark:hover:bg-[#282832] text-slate-800 dark:text-slate-200 rounded-lg text-xs font-bold cursor-pointer"
            >
              {t("continueSwitchBtn")}
            </button>
            <button
              onclick={handleConfirmSwitchCloseSketchUp}
              class="px-3 py-1.5 bg-rose-600 hover:bg-rose-500 text-white rounded-lg text-xs font-bold shadow-xs cursor-pointer"
            >
              {t("closeAndSwitchBtn")}
            </button>
          </div>
        </div>

      <!-- 4. Update Available Modal -->
      {:else if appStore.modal.type === "update"}
        <div class="p-4 space-y-3">
          <div class="flex items-center space-x-2 text-[#0284c7] dark:text-[#00c8ff]">
            <CloudArrowDown size={20} weight="bold" />
            <span class="text-xs font-bold text-[#0f172a] dark:text-[#fafafa]">{t("updateHeading")}</span>
          </div>

          <div class="text-xs text-[#64748b] dark:text-[#a1a1aa] space-y-1">
            <div>
              <span class="font-semibold">{t("versionLabel")}</span>
              <span class="text-[#0284c7] dark:text-[#00c8ff] font-bold ml-1">{appStore.modal.info.latestVersion}</span>
            </div>
            <p class="text-[11px] bg-[#f8fafc] dark:bg-[#121216] p-2 rounded border border-[#cbd5e1] dark:border-[#222228] text-[#334155] dark:text-[#d4d4d8] max-h-24 overflow-y-auto">
              {appStore.modal.info.releaseNotes || "Cập nhật hiệu năng và giao diện."}
            </p>
          </div>

          <div class="flex items-center justify-between pt-2 border-t border-[#e2e8f0] dark:border-[#1e1e24]">
            <div class="flex space-x-1">
              <button
                onclick={handleUpdateDoNotRemind}
                class="px-2 py-1 text-[11px] text-[#94a3b8] hover:text-[#0f172a] dark:hover:text-white cursor-pointer"
              >
                {t("ignoreUpdateBtn")}
              </button>
              <button
                onclick={handleUpdateRemindLater}
                class="px-2 py-1 text-[11px] text-[#94a3b8] hover:text-[#0f172a] dark:hover:text-white cursor-pointer"
              >
                {t("remindLaterBtn")}
              </button>
            </div>

            <button
              onclick={handleUpdateBackground}
              disabled={downloadingUpdate}
              class="px-3 py-1.5 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-xs font-bold shadow-xs cursor-pointer disabled:opacity-50"
            >
              {downloadingUpdate ? "Đang tải..." : t("downloadBgBtn")}
            </button>
          </div>
        </div>

      <!-- 5. Confirm Apply Imported Profile Modal -->
      {:else if appStore.modal.type === "confirm_apply_imported"}
        <div class="p-4 space-y-3">
          <div class="flex items-center space-x-2 text-[#0284c7] dark:text-[#00c8ff]">
            <CheckCircle size={18} weight="fill" />
            <span class="text-xs font-bold text-[#0f172a] dark:text-[#fafafa]">Nhập thành công</span>
          </div>

          <p class="text-xs text-[#64748b] dark:text-[#a1a1aa] leading-relaxed">
            Đã nhập hồ sơ <strong class="text-[#0f172a] dark:text-white">'{appStore.modal.profileName}'</strong>. Bạn có muốn kích hoạt ngay không?
          </p>

          <div class="flex items-center justify-end space-x-2 pt-2 border-t border-[#e2e8f0] dark:border-[#1e1e24]">
            <button
              onclick={() => {
                closeModal();
                appStore.activeTab = "profiles";
              }}
              class="px-3 py-1.5 bg-[#f1f5f9] dark:bg-[#151518] hover:bg-[#e2e8f0] dark:hover:bg-[#202026] text-[#64748b] dark:text-[#a1a1aa] rounded-lg text-xs font-semibold cursor-pointer"
            >
              Để sau
            </button>
            <button
              onclick={async () => {
                const target = appStore.modal.profileName;
                closeModal();
                if (appStore.sketchupRunning) {
                  appStore.modal = { type: "confirm_switch", targetProfile: target };
                } else {
                  await switchProfile(target);
                  appStore.activeTab = "profiles";
                }
              }}
              class="px-3.5 py-1.5 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-xs font-bold shadow-xs cursor-pointer"
            >
              Kích hoạt ngay
            </button>
          </div>
        </div>

      <!-- 6. Confirm Delete Profile Modal -->
      {:else if appStore.modal.type === "confirm_delete"}
        <div class="p-4 space-y-3">
          <div class="flex items-center space-x-2 text-rose-500 dark:text-rose-400">
            <Trash size={18} weight="bold" />
            <span class="text-xs font-bold text-[#0f172a] dark:text-[#fafafa]">{t("confirmDeleteHeading")}</span>
          </div>

          <p class="text-xs text-[#64748b] dark:text-[#a1a1aa] leading-relaxed">
            {t("confirmDeleteMsg")} <strong class="text-rose-600 dark:text-rose-400">'{appStore.modal.profileName}'</strong>?
          </p>

          <div class="flex items-center justify-end space-x-2 pt-2 border-t border-[#e2e8f0] dark:border-[#1e1e24]">
            <button
              onclick={closeModal}
              class="px-3 py-1.5 bg-[#f1f5f9] dark:bg-[#151518] hover:bg-[#e2e8f0] dark:hover:bg-[#202026] text-[#64748b] dark:text-[#a1a1aa] rounded-lg text-xs font-semibold cursor-pointer"
            >
              {t("cancelBtn")}
            </button>
            <button
              onclick={async () => {
                const target = appStore.modal.profileName;
                closeModal();
                await deleteProfile(target);
              }}
              class="px-3.5 py-1.5 bg-rose-600 hover:bg-rose-500 text-white rounded-lg text-xs font-bold shadow-xs cursor-pointer transition-colors"
            >
              {t("deleteBtn")}
            </button>
          </div>
        </div>
      {/if}

    </div>
  </div>
{/if}
