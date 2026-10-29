<script>
  import { Check } from "phosphor-svelte";

  let {
    checked = $bindable(false),
    label = "",
    id = "",
    disabled = false,
    icon: Icon = null,
    onchange = null,
  } = $props();

  function toggle() {
    if (!disabled) {
      checked = !checked;
      if (onchange) {
        onchange(checked);
      }
    }
  }
</script>

<button
  type="button"
  {id}
  onclick={toggle}
  {disabled}
  class="flex items-center space-x-2 group cursor-pointer text-left select-none disabled:opacity-50 disabled:cursor-not-allowed"
>
  <!-- Circular Checkbox UI (Perfect Circle) -->
  <div
    class="w-4 h-4 rounded-full flex items-center justify-center transition-all duration-150 shrink-0 border {checked
      ? 'bg-[#00c8ff] border-[#00c8ff] text-[#050505] shadow-xs'
      : 'bg-white dark:bg-[#121215] border-slate-300 dark:border-[#3a3a46] group-hover:border-slate-400 dark:group-hover:border-[#5a5a68]'}"
  >
    {#if checked}
      <Check size={10} weight="bold" />
    {/if}
  </div>

  {#if Icon}
    <Icon size={13} weight="bold" class="text-sky-600 dark:text-[#00c8ff] shrink-0" />
  {/if}

  {#if label}
    <span class="text-xs font-semibold text-slate-700 dark:text-[#d4d4d8] group-hover:text-slate-900 dark:group-hover:text-white transition-colors">
      {label}
    </span>
  {/if}
</button>
