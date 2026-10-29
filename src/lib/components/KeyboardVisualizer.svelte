<script>
  import { Keyboard, PuzzlePiece, CheckCircle, ArrowRight } from "phosphor-svelte";

  let { shortcuts = [], plugins = [] } = $props();

  let activeModifier = $state("none"); // "none" | "shift" | "ctrl" | "alt" | "ctrl_shift" | "alt_shift"
  let showNumPad = $state(false);
  let selectedShortcut = $state(null);

  // Pre-index plugins by lowercase name for fast lookup
  const pluginMap = $derived(() => {
    const map = new Map();
    for (const p of plugins) {
      map.set(p.name.toLowerCase(), p);
    }
    return map;
  });

  // Helper to map a command string to a plugin icon
  function resolvePluginForCommand(cmd) {
    if (!cmd) return null;
    const lower = cmd.toLowerCase().replace(/\\/g, "/");
    const map = pluginMap();

    // 1. Suforyou
    if (lower.includes("suforyou")) {
      for (const [pname, p] of map.entries()) {
        if (pname.startsWith("s4u_")) {
          const sub = pname.slice(4).replace(/_/g, "");
          if (lower.replace(/[\s_/]/g, "").includes(sub)) {
            return p;
          }
        }
      }
    }

    // 2. Fredo6
    if (lower.includes("fredo") || lower.includes("jointpushpull") || lower.includes("curviloft") || lower.includes("roundcorner") || lower.includes("toposhaper") || lower.includes("visuhole")) {
      for (const [pname, p] of map.entries()) {
        if (pname.includes("fredo") || pname.includes("jointpushpull")) {
          const sub = pname.replace("fredo6_", "").replace(/_/g, "");
          if (lower.replace(/[\s_/]/g, "").includes(sub)) {
            return p;
          }
        }
      }
    }

    // 3. Curic
    if (lower.includes("curic")) {
      for (const [pname, p] of map.entries()) {
        if (pname.startsWith("curic_")) {
          const sub = pname.slice(6).replace(/_/g, "");
          if (lower.replace(/[\s_/]/g, "").includes(sub)) {
            return p;
          }
        }
      }
    }

    // 4. 1001bit
    if (lower.includes("1001bit")) {
      return map.get("1001bit_pro") || null;
    }

    // 5. Mirror
    if (lower.includes("mirror")) {
      return map.get("mirror") || null;
    }

    // 6. Generic lookup against all plugins
    for (const [pname, p] of map.entries()) {
      const clean = pname.replace(/_/g, " ");
      if (lower.includes(clean) || lower.includes(pname)) {
        return p;
      }
    }

    return null;
  }

  // Parse raw shortcut string: "ctrl alt shift key command"
  function parseShortcut(item) {
    const raw = item.raw || "";
    const parts = raw.split(" ");
    let ctrl = false;
    let alt = false;
    let shift = false;
    let baseKey = item.keyDisplay || "";
    let cmd = item.command || "";

    if (parts.length >= 5) {
      ctrl = parts[0] === "1";
      alt = parts[1] === "1";
      shift = parts[2] === "1";
      baseKey = parts[3].toUpperCase();
    } else {
      // Fallback parse from keyDisplay e.g. "Shift + Z"
      const tokens = (item.keyDisplay || "").split("+").map(t => t.trim());
      baseKey = tokens[tokens.length - 1].toUpperCase();
      ctrl = tokens.some(t => t.toLowerCase() === "ctrl");
      alt = tokens.some(t => t.toLowerCase() === "alt");
      shift = tokens.some(t => t.toLowerCase() === "shift");
    }

    const plugin = resolvePluginForCommand(cmd);

    return {
      ...item,
      ctrl,
      alt,
      shift,
      baseKey,
      plugin,
    };
  }

  // Index shortcuts for the current active modifier
  const activeShortcutMap = $derived(() => {
    const map = new Map();
    for (const item of shortcuts) {
      const parsed = parseShortcut(item);
      let match = false;
      if (activeModifier === "none" && !parsed.ctrl && !parsed.alt && !parsed.shift) match = true;
      else if (activeModifier === "shift" && !parsed.ctrl && !parsed.alt && parsed.shift) match = true;
      else if (activeModifier === "ctrl" && parsed.ctrl && !parsed.alt && !parsed.shift) match = true;
      else if (activeModifier === "alt" && !parsed.ctrl && parsed.alt && !parsed.shift) match = true;
      else if (activeModifier === "ctrl_shift" && parsed.ctrl && !parsed.alt && parsed.shift) match = true;
      else if (activeModifier === "alt_shift" && !parsed.ctrl && parsed.alt && parsed.shift) match = true;

      if (match) {
        map.set(parsed.baseKey, parsed);
      }
    }
    return map;
  });

  // Standard ANSI Layout Definition
  const rowFunc = [
    { label: "Esc", key: "ESCAPE" },
    { label: "F1", key: "F1" },
    { label: "F2", key: "F2" },
    { label: "F3", key: "F3" },
    { label: "F4", key: "F4" },
    { label: "F5", key: "F5" },
    { label: "F6", key: "F6" },
    { label: "F7", key: "F7" },
    { label: "F8", key: "F8" },
    { label: "F9", key: "F9" },
    { label: "F10", key: "F10" },
    { label: "F11", key: "F11" },
    { label: "F12", key: "F12" },
  ];

  const rowNumbers = [
    { label: "`", key: "`" },
    { label: "1", key: "1" },
    { label: "2", key: "2" },
    { label: "3", key: "3" },
    { label: "4", key: "4" },
    { label: "5", key: "5" },
    { label: "6", key: "6" },
    { label: "7", key: "7" },
    { label: "8", key: "8" },
    { label: "9", key: "9" },
    { label: "0", key: "0" },
    { label: "-", key: "-" },
    { label: "=", key: "=" },
    { label: "⌫", key: "BACKSPACE", w: "w-[42px]" },
  ];

  const rowQwerty = [
    { label: "Tab", key: "TAB", w: "w-[36px]" },
    { label: "Q", key: "Q" },
    { label: "W", key: "W" },
    { label: "E", key: "E" },
    { label: "R", key: "R" },
    { label: "T", key: "T" },
    { label: "Y", key: "Y" },
    { label: "U", key: "U" },
    { label: "I", key: "I" },
    { label: "O", key: "O" },
    { label: "P", key: "P" },
    { label: "[", key: "[" },
    { label: "]", key: "]" },
    { label: "\\", key: "\\" },
  ];

  const rowHome = [
    { label: "Caps", key: "CAPSLOCK", w: "w-[42px]" },
    { label: "A", key: "A" },
    { label: "S", key: "S" },
    { label: "D", key: "D" },
    { label: "F", key: "F" },
    { label: "G", key: "G" },
    { label: "H", key: "H" },
    { label: "J", key: "J" },
    { label: "K", key: "K" },
    { label: "L", key: "L" },
    { label: ";", key: ";" },
    { label: "'", key: "'" },
    { label: "Enter", key: "RETURN", w: "w-[48px]" },
  ];

  const rowBottom = [
    { label: "Shift", key: "SHIFT", w: "w-[52px]" },
    { label: "Z", key: "Z" },
    { label: "X", key: "X" },
    { label: "C", key: "C" },
    { label: "V", key: "V" },
    { label: "B", key: "B" },
    { label: "N", key: "N" },
    { label: "M", key: "M" },
    { label: ",", key: "," },
    { label: ".", key: "." },
    { label: "/", key: "/" },
    { label: "Shift", key: "RSHIFT", w: "w-[46px]" },
  ];

  const rowSpace = [
    { label: "Ctrl", key: "CTRL", w: "w-[38px]" },
    { label: "Alt", key: "ALT", w: "w-[38px]" },
    { label: "Space", key: "SPACE", w: "flex-1" },
    { label: "Alt", key: "RALT", w: "w-[38px]" },
    { label: "Ctrl", key: "RCTRL", w: "w-[38px]" },
  ];

  const rowNumPad = [
    { label: "Num0", key: "NUMPAD0" },
    { label: "Num1", key: "NUMPAD1" },
    { label: "Num2", key: "NUMPAD2" },
    { label: "Num3", key: "NUMPAD3" },
    { label: "Num4", key: "NUMPAD4" },
    { label: "Num5", key: "NUMPAD5" },
    { label: "Num6", key: "NUMPAD6" },
    { label: "Num7", key: "NUMPAD7" },
    { label: "Num8", key: "NUMPAD8" },
    { label: "Num9", key: "NUMPAD9" },
    { label: "Num+", key: "NUMPAD+" },
    { label: "Num-", key: "NUMPAD-" },
    { label: "Num*", key: "NUMPAD*" },
    { label: "Num/", key: "NUMPAD/" },
  ];

  function handleKeyClick(keyId) {
    const sc = activeShortcutMap().get(keyId.toUpperCase());
    if (sc) {
      selectedShortcut = sc;
    } else {
      selectedShortcut = {
        keyDisplay: formatKeyWithModifier(keyId),
        command: "Chưa gán lệnh",
        isUnassigned: true,
      };
    }
  }

  function formatKeyWithModifier(keyId) {
    const prefix = activeModifier === "shift" ? "Shift + "
      : activeModifier === "ctrl" ? "Ctrl + "
      : activeModifier === "alt" ? "Alt + "
      : activeModifier === "ctrl_shift" ? "Ctrl + Shift + "
      : activeModifier === "alt_shift" ? "Alt + Shift + "
      : "";
    return prefix + keyId;
  }
</script>

<div class="flex flex-col space-y-2 select-none">
  <!-- Modifier Selection Header -->
  <div class="flex items-center justify-between pb-1 text-xs">
    <div class="flex items-center space-x-1 overflow-x-auto py-0.5">
      <button
        type="button"
        onclick={() => (activeModifier = "none")}
        class="px-2 py-0.5 rounded text-[11px] font-semibold cursor-pointer transition-colors {activeModifier === 'none'
          ? 'bg-sky-500/20 text-sky-600 dark:text-[#00c8ff] border border-sky-500/40'
          : 'bg-[#f1f5f9] dark:bg-[#16161b] text-[#64748b] dark:text-[#a1a1aa] hover:bg-slate-200 dark:hover:bg-[#202028]'}"
      >
        Không mod
      </button>

      <button
        type="button"
        onclick={() => (activeModifier = "shift")}
        class="px-2 py-0.5 rounded text-[11px] font-semibold cursor-pointer transition-colors {activeModifier === 'shift'
          ? 'bg-sky-500/20 text-sky-600 dark:text-[#00c8ff] border border-sky-500/40'
          : 'bg-[#f1f5f9] dark:bg-[#16161b] text-[#64748b] dark:text-[#a1a1aa] hover:bg-slate-200 dark:hover:bg-[#202028]'}"
      >
        Shift
      </button>

      <button
        type="button"
        onclick={() => (activeModifier = "ctrl")}
        class="px-2 py-0.5 rounded text-[11px] font-semibold cursor-pointer transition-colors {activeModifier === 'ctrl'
          ? 'bg-sky-500/20 text-sky-600 dark:text-[#00c8ff] border border-sky-500/40'
          : 'bg-[#f1f5f9] dark:bg-[#16161b] text-[#64748b] dark:text-[#a1a1aa] hover:bg-slate-200 dark:hover:bg-[#202028]'}"
      >
        Ctrl
      </button>

      <button
        type="button"
        onclick={() => (activeModifier = "alt")}
        class="px-2 py-0.5 rounded text-[11px] font-semibold cursor-pointer transition-colors {activeModifier === 'alt'
          ? 'bg-sky-500/20 text-sky-600 dark:text-[#00c8ff] border border-sky-500/40'
          : 'bg-[#f1f5f9] dark:bg-[#16161b] text-[#64748b] dark:text-[#a1a1aa] hover:bg-slate-200 dark:hover:bg-[#202028]'}"
      >
        Alt
      </button>

      <button
        type="button"
        onclick={() => (activeModifier = "ctrl_shift")}
        class="px-2 py-0.5 rounded text-[11px] font-semibold cursor-pointer transition-colors {activeModifier === 'ctrl_shift'
          ? 'bg-sky-500/20 text-sky-600 dark:text-[#00c8ff] border border-sky-500/40'
          : 'bg-[#f1f5f9] dark:bg-[#16161b] text-[#64748b] dark:text-[#a1a1aa] hover:bg-slate-200 dark:hover:bg-[#202028]'}"
      >
        Ctrl+Shift
      </button>
    </div>

    <button
      type="button"
      onclick={() => (showNumPad = !showNumPad)}
      class="px-2 py-0.5 rounded text-[10px] font-medium cursor-pointer border {showNumPad
        ? 'bg-indigo-500/15 text-indigo-500 border-indigo-500/30'
        : 'bg-[#f1f5f9] dark:bg-[#16161b] text-[#64748b] dark:text-[#a1a1aa] border-transparent'}"
    >
      {showNumPad ? "Ẩn NumPad" : "Hiện NumPad"}
    </button>
  </div>

  <!-- Keyboard Layout Board -->
  <div class="bg-[#f1f5f9] dark:bg-[#0d0d10] p-2 rounded-xl border border-[#cbd5e1] dark:border-[#1e1e24] space-y-1 overflow-x-auto shadow-inner">
    <!-- Function Row -->
    <div class="flex space-x-1 justify-between">
      {#each rowFunc as k}
        {@const sc = activeShortcutMap().get(k.key.toUpperCase())}
        <button
          type="button"
          onclick={() => handleKeyClick(k.key)}
          title={sc ? `${k.label}: ${sc.command}` : k.label}
          class="h-6 flex-1 min-w-[28px] max-w-[40px] rounded flex flex-col items-center justify-center text-[9px] font-mono transition-all cursor-pointer border {sc
            ? 'bg-sky-500/20 text-sky-600 dark:text-[#00c8ff] border-sky-500/50 shadow-xs'
            : 'bg-[#ffffff] dark:bg-[#141418] text-[#64748b] dark:text-[#71717a] border-[#e2e8f0] dark:border-[#22222a] hover:border-slate-400 dark:hover:border-slate-600'}"
        >
          <span>{k.label}</span>
          {#if sc?.plugin?.iconData}
            <img src={sc.plugin.iconData} alt="" class="w-2.5 h-2.5 object-contain" />
          {/if}
        </button>
      {/each}
    </div>

    <!-- Numbers Row -->
    <div class="flex space-x-1">
      {#each rowNumbers as k}
        {@const sc = activeShortcutMap().get(k.key.toUpperCase())}
        <button
          type="button"
          onclick={() => handleKeyClick(k.key)}
          title={sc ? `${k.label}: ${sc.command}` : k.label}
          class="h-7 {k.w || 'flex-1 min-w-[26px]'} rounded flex flex-col items-center justify-center text-[10px] font-mono transition-all cursor-pointer border {sc
            ? 'bg-sky-500/20 text-sky-600 dark:text-[#00c8ff] border-sky-500/50 shadow-xs'
            : 'bg-[#ffffff] dark:bg-[#141418] text-[#334155] dark:text-[#a1a1aa] border-[#e2e8f0] dark:border-[#22222a] hover:border-slate-400 dark:hover:border-slate-600'}"
        >
          <span>{k.label}</span>
          {#if sc?.plugin?.iconData}
            <img src={sc.plugin.iconData} alt="" class="w-3 h-3 object-contain" />
          {/if}
        </button>
      {/each}
    </div>

    <!-- QWERTY Row -->
    <div class="flex space-x-1">
      {#each rowQwerty as k}
        {@const sc = activeShortcutMap().get(k.key.toUpperCase())}
        <button
          type="button"
          onclick={() => handleKeyClick(k.key)}
          title={sc ? `${k.label}: ${sc.command}` : k.label}
          class="h-7 {k.w || 'flex-1 min-w-[26px]'} rounded flex flex-col items-center justify-center text-[10px] font-mono transition-all cursor-pointer border {sc
            ? 'bg-sky-500/20 text-sky-600 dark:text-[#00c8ff] border-sky-500/50 shadow-xs font-bold'
            : 'bg-[#ffffff] dark:bg-[#141418] text-[#334155] dark:text-[#a1a1aa] border-[#e2e8f0] dark:border-[#22222a] hover:border-slate-400 dark:hover:border-slate-600'}"
        >
          <span>{k.label}</span>
          {#if sc?.plugin?.iconData}
            <img src={sc.plugin.iconData} alt="" class="w-3 h-3 object-contain" />
          {/if}
        </button>
      {/each}
    </div>

    <!-- Home Row (ASDF) -->
    <div class="flex space-x-1">
      {#each rowHome as k}
        {@const sc = activeShortcutMap().get(k.key.toUpperCase())}
        <button
          type="button"
          onclick={() => handleKeyClick(k.key)}
          title={sc ? `${k.label}: ${sc.command}` : k.label}
          class="h-7 {k.w || 'flex-1 min-w-[26px]'} rounded flex flex-col items-center justify-center text-[10px] font-mono transition-all cursor-pointer border {sc
            ? 'bg-sky-500/20 text-sky-600 dark:text-[#00c8ff] border-sky-500/50 shadow-xs font-bold'
            : 'bg-[#ffffff] dark:bg-[#141418] text-[#334155] dark:text-[#a1a1aa] border-[#e2e8f0] dark:border-[#22222a] hover:border-slate-400 dark:hover:border-slate-600'}"
        >
          <span>{k.label}</span>
          {#if sc?.plugin?.iconData}
            <img src={sc.plugin.iconData} alt="" class="w-3 h-3 object-contain" />
          {/if}
        </button>
      {/each}
    </div>

    <!-- Bottom Row (ZXCV) -->
    <div class="flex space-x-1">
      {#each rowBottom as k}
        {@const sc = activeShortcutMap().get(k.key.toUpperCase())}
        <button
          type="button"
          onclick={() => handleKeyClick(k.key)}
          title={sc ? `${k.label}: ${sc.command}` : k.label}
          class="h-7 {k.w || 'flex-1 min-w-[26px]'} rounded flex flex-col items-center justify-center text-[10px] font-mono transition-all cursor-pointer border {sc
            ? 'bg-sky-500/20 text-sky-600 dark:text-[#00c8ff] border-sky-500/50 shadow-xs font-bold'
            : 'bg-[#ffffff] dark:bg-[#141418] text-[#334155] dark:text-[#a1a1aa] border-[#e2e8f0] dark:border-[#22222a] hover:border-slate-400 dark:hover:border-slate-600'}"
        >
          <span>{k.label}</span>
          {#if sc?.plugin?.iconData}
            <img src={sc.plugin.iconData} alt="" class="w-3 h-3 object-contain" />
          {/if}
        </button>
      {/each}
    </div>

    <!-- Space Row -->
    <div class="flex space-x-1">
      {#each rowSpace as k}
        {@const sc = activeShortcutMap().get(k.key.toUpperCase())}
        <button
          type="button"
          onclick={() => handleKeyClick(k.key)}
          title={sc ? `${k.label}: ${sc.command}` : k.label}
          class="h-7 {k.w} rounded flex flex-col items-center justify-center text-[10px] font-mono transition-all cursor-pointer border {sc
            ? 'bg-sky-500/20 text-sky-600 dark:text-[#00c8ff] border-sky-500/50 shadow-xs font-bold'
            : 'bg-[#ffffff] dark:bg-[#141418] text-[#334155] dark:text-[#a1a1aa] border-[#e2e8f0] dark:border-[#22222a] hover:border-slate-400 dark:hover:border-slate-600'}"
        >
          <span>{k.label}</span>
          {#if sc?.plugin?.iconData}
            <img src={sc.plugin.iconData} alt="" class="w-3 h-3 object-contain" />
          {/if}
        </button>
      {/each}
    </div>

    <!-- Optional NumPad Row -->
    {#if showNumPad}
      <div class="flex space-x-1 pt-1 border-t border-[#e2e8f0] dark:border-[#1e1e24]">
        {#each rowNumPad as k}
          {@const sc = activeShortcutMap().get(k.key.toUpperCase())}
          <button
            type="button"
            onclick={() => handleKeyClick(k.key)}
            title={sc ? `${k.label}: ${sc.command}` : k.label}
            class="h-7 flex-1 min-w-[24px] rounded flex flex-col items-center justify-center text-[9px] font-mono transition-all cursor-pointer border {sc
              ? 'bg-sky-500/20 text-sky-600 dark:text-[#00c8ff] border-sky-500/50 shadow-xs font-bold'
              : 'bg-[#ffffff] dark:bg-[#141418] text-[#334155] dark:text-[#a1a1aa] border-[#e2e8f0] dark:border-[#22222a] hover:border-slate-400 dark:hover:border-slate-600'}"
          >
            <span>{k.label}</span>
            {#if sc?.plugin?.iconData}
              <img src={sc.plugin.iconData} alt="" class="w-3 h-3 object-contain" />
            {/if}
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Key Inspection Detail Panel -->
  {#if selectedShortcut}
    <div class="p-2 rounded-lg bg-[#f8fafc] dark:bg-[#121216] border border-[#e2e8f0] dark:border-[#1e1e24] flex items-center justify-between text-xs animate-in fade-in duration-100">
      <div class="flex items-center space-x-2 truncate flex-1 mr-2">
        <span class="px-2 py-0.5 rounded bg-sky-500/15 text-sky-600 dark:text-[#00c8ff] font-mono font-bold text-[10px] border border-sky-500/30 shrink-0">
          {selectedShortcut.keyDisplay}
        </span>
        {#if selectedShortcut.plugin}
          {#if selectedShortcut.plugin.iconData}
            <img src={selectedShortcut.plugin.iconData} alt="" class="w-4 h-4 object-contain rounded shrink-0" />
          {:else}
            <PuzzlePiece size={13} weight="bold" class="text-sky-600 dark:text-[#00c8ff] shrink-0" />
          {/if}
          <span class="text-[10px] text-[#64748b] dark:text-[#a1a1aa] font-semibold shrink-0">
            [{selectedShortcut.plugin.name}]
          </span>
        {/if}
        <span class="text-xs text-[#0f172a] dark:text-[#fafafa] truncate font-medium" title={selectedShortcut.command}>
          {selectedShortcut.command}
        </span>
      </div>

      {#if selectedShortcut.isUnassigned}
        <span class="text-[9px] px-1.5 py-0.5 rounded bg-slate-500/10 text-slate-500 border border-slate-500/20 shrink-0 font-medium">
          Chưa gán
        </span>
      {:else if selectedShortcut.originalKey}
        <span class="text-[9px] px-1.5 py-0.5 rounded bg-indigo-500/10 text-indigo-600 dark:text-indigo-400 border border-indigo-500/20 shrink-0 font-medium">
          Đổi phím (Gốc: {selectedShortcut.originalKey})
        </span>
      {:else if selectedShortcut.isCustom}
        <span class="text-[9px] px-1.5 py-0.5 rounded bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/20 shrink-0 font-medium">
          Thêm mới
        </span>
      {:else}
        <span class="text-[9px] px-1.5 py-0.5 rounded bg-slate-500/10 text-slate-500 border border-slate-500/20 shrink-0 font-medium">
          Mặc định
        </span>
      {/if}
    </div>
  {:else}
    <div class="text-[10px] text-center text-[#94a3b8] dark:text-[#71717a] py-1">
      Nhấp vào bất kỳ phím nào trên bàn phím để xem chi tiết lệnh và tiện ích gán
    </div>
  {/if}
</div>
