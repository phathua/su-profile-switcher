<script>
  import {
    appStore,
    renameProfile,
    deleteProfile,
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
  } from "phosphor-svelte";

  let renameValue = $state("");
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
    await renameProfile(appStore.modal.oldName, renameValue.trim());
    closeModal();
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

      <!-- 2. Rename Modal -->
      {:else if appStore.modal.type === "rename"}
        <div class="p-4 space-y-3">
          <div class="flex items-center justify-between pb-2 border-b border-[#e2e8f0] dark:border-[#1e1e24]">
            <div class="flex items-center space-x-2">
              <PencilSimple size={15} weight="bold" class="text-[#0284c7] dark:text-[#00c8ff]" />
              <span class="text-xs font-bold text-[#0f172a] dark:text-[#fafafa]">{t("renameHeading")}</span>
            </div>
            <button onclick={closeModal} class="text-[#64748b] dark:text-[#a1a1aa] hover:text-[#0f172a] dark:hover:text-white cursor-pointer">
              <X size={14} weight="bold" />
            </button>
          </div>

          <div>
            <label class="block text-[11px] font-semibold text-[#64748b] dark:text-[#a1a1aa] mb-1" for="renameInput">
              Tên hồ sơ mới:
            </label>
            <input
              id="renameInput"
              type="text"
              bind:value={renameValue}
              class="w-full bg-[#f8fafc] dark:bg-[#141417] border border-[#cbd5e1] dark:border-[#2b2b34] rounded-lg px-2.5 py-1.5 text-xs text-[#0f172a] dark:text-[#ffffff] focus:outline-none focus:border-[#00c8ff]"
            />
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
              {t("renameBtn")}
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
              {t("cancelBtn")}
            </button>
            <button
              onclick={handleConfirmSwitchCloseSketchUp}
              class="px-3 py-1.5 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-xs font-bold shadow-xs cursor-pointer"
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
