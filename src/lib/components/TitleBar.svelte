<script>
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { Minus, X, ArrowClockwise, Sparkle } from "phosphor-svelte";
  import { appStore, t, checkForUpdates, applyUpdateAndRestart } from "../store.svelte.js";
  import { APP_VERSION } from "../version.js";

  const appWindow = getCurrentWindow();

  async function minimize() {
    await appWindow.minimize();
  }

  // NOTE: Tuyệt đối không thêm nút hoặc tính năng phóng to (maximize) cửa sổ vào đây

  async function close() {
    await appWindow.hide();
  }
</script>

<div
  data-tauri-drag-region
  class="h-9 bg-[#ffffff] dark:bg-[#050507] border-b border-[#e2e8f0] dark:border-[#18181b] px-3.5 flex items-center justify-between select-none cursor-grab active:cursor-grabbing"
>
  <!-- App Branding & Version Pill -->
  <div class="flex items-center space-x-2.5 pointer-events-none">
    <img src="/icon.svg" alt="logo" class="w-4 h-4 object-contain" />
    <span class="font-bold text-xs text-slate-800 dark:text-[#fafafa] tracking-wide">{t("appTitle")}</span>
    <span class="px-2 py-0.5 rounded-full text-[10px] font-bold bg-sky-100 dark:bg-[#00c8ff]/15 text-sky-700 dark:text-[#00c8ff] border border-sky-300 dark:border-[#00c8ff]/30 tracking-tight">
      v{APP_VERSION}
    </span>
  </div>

  <!-- Window Controls & Update Button -->
  <div class="flex items-center space-x-1">
    <!-- Update Action (Pill when update available, sleek icon when normal) -->
    {#if appStore.updateInfo && appStore.updateInfo.hasUpdate}
      <button
        onclick={applyUpdateAndRestart}
        class="flex items-center space-x-1.5 px-2.5 py-0.5 rounded-full bg-sky-600 hover:bg-sky-500 text-white text-[10px] font-bold shadow-xs transition-all cursor-pointer mr-1 animate-pulse"
        title="Có bản mới v{appStore.updateInfo.latestVersion}. Bấm để tải và tự khởi động lại."
      >
        <Sparkle size={11} weight="fill" />
        <span>Cập nhật ngay</span>
      </button>
    {:else}
      <button
        onclick={() => checkForUpdates(false)}
        class="w-6.5 h-6.5 flex items-center justify-center rounded hover:bg-[#e2e8f0] dark:hover:bg-[#18181b] text-slate-500 dark:text-[#a1a1aa] hover:text-slate-800 dark:hover:text-white transition-colors cursor-pointer mr-0.5"
        title="Kiểm tra bản cập nhật mới"
      >
        <ArrowClockwise
          size={12}
          weight="bold"
          class={appStore.isCheckingUpdate ? "animate-spin text-sky-600 dark:text-[#00c8ff]" : ""}
        />
      </button>
    {/if}

    <button
      onclick={minimize}
      class="w-6.5 h-6.5 flex items-center justify-center rounded hover:bg-[#e2e8f0] dark:hover:bg-[#18181b] text-[#64748b] dark:text-[#a1a1aa] hover:text-[#0f172a] dark:hover:text-[#ffffff] transition-colors cursor-pointer"
      title="Thu nhỏ"
    >
      <Minus size={12} weight="bold" />
    </button>
    <!-- NOTE: Tuyệt đối không tự thêm nút phóng to (maximize) tại đây -->
    <button
      onclick={close}
      class="w-6.5 h-6.5 flex items-center justify-center rounded hover:bg-[#ef4444]/20 text-[#64748b] dark:text-[#a1a1aa] hover:text-[#ef4444] transition-colors cursor-pointer"
      title="Đóng xuống khay"
    >
      <X size={12} weight="bold" />
    </button>
  </div>
</div>
