<script>
  import { onMount } from "svelte";
  import { CaretDown, Check } from "phosphor-svelte";

  let {
    options = [],
    value = "",
    onchange = () => {},
    placeholder = "Chọn...",
  } = $props();

  let isOpen = $state(false);
  let dropdownRef = $state(null);

  function toggle() {
    isOpen = !isOpen;
  }

  function selectOption(val) {
    onchange(val);
    isOpen = false;
  }

  function handleOutsideClick(event) {
    if (dropdownRef && !dropdownRef.contains(event.target)) {
      isOpen = false;
    }
  }

  onMount(() => {
    document.addEventListener("click", handleOutsideClick);
    return () => {
      document.removeEventListener("click", handleOutsideClick);
    };
  });
</script>

<div class="relative inline-block text-left z-30" bind:this={dropdownRef}>
  <!-- Trigger Button (Clean in both Light & Dark modes) -->
  <button
    type="button"
    onclick={toggle}
    class="flex items-center justify-between space-x-2 bg-white dark:bg-[#0c0c0e] hover:bg-slate-50 dark:hover:bg-[#141417] border border-slate-300 dark:border-[#222228] hover:border-slate-400 dark:hover:border-[#383842] text-slate-800 dark:text-[#fafafa] px-2.5 py-1.5 rounded-lg text-xs font-semibold shadow-xs transition-colors cursor-pointer focus:outline-none focus:ring-1 focus:ring-sky-500 dark:focus:ring-[#00c8ff]"
  >
    <span class="truncate">{value || placeholder}</span>
    <CaretDown size={11} weight="bold" class="text-slate-400 dark:text-[#a1a1aa] shrink-0 transition-transform duration-200 {isOpen ? 'rotate-180' : ''}" />
  </button>

  <!-- Dropdown Popover (High z-index to never get clipped) -->
  {#if isOpen}
    <div
      class="absolute left-0 mt-1.5 min-w-[160px] w-max bg-white dark:bg-[#0c0c0e] border border-slate-200 dark:border-[#2b2b34] rounded-lg shadow-2xl p-1 z-[999] animate-in fade-in zoom-in-95 duration-100"
    >
      <div class="max-h-48 overflow-y-auto space-y-0.5">
        {#each options as opt}
          {@const optVal = typeof opt === "string" ? opt : opt.value}
          {@const optLabel = typeof opt === "string" ? opt : opt.label}
          {@const isSelected = optVal === value}
          <button
            type="button"
            onclick={() => selectOption(optVal)}
            class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-md text-xs font-medium transition-colors text-left {isSelected
              ? 'bg-sky-50 dark:bg-[#141418] text-sky-600 dark:text-[#00c8ff] font-bold'
              : 'text-slate-700 dark:text-[#d4d4d8] hover:bg-slate-100 dark:hover:bg-[#18181c] hover:text-slate-900 dark:hover:text-white'}"
          >
            <span class="truncate">{optLabel}</span>
            {#if isSelected}
              <Check size={12} weight="bold" class="text-sky-600 dark:text-[#00c8ff] ml-2 shrink-0" />
            {/if}
          </button>
        {/each}
      </div>
    </div>
  {/if}
</div>
