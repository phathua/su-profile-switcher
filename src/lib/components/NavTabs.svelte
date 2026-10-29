<script>
  import { appStore, t } from "../store.svelte.js";
  import { Folder, FloppyDisk, GearSix } from "phosphor-svelte";

  const tabs = [
    { id: "profiles", key: "tabProfiles", icon: Folder },
    { id: "backup", key: "tabBackup", icon: FloppyDisk },
    { id: "settings", key: "tabSettings", icon: GearSix },
  ];
</script>

<div class="bg-[#f1f5f9] dark:bg-[#0c0c0e] p-0.5 rounded-lg border border-[#e2e8f0] dark:border-[#1e1e24] inline-flex space-x-1">
  {#each tabs as tab}
    {@const isActive = appStore.activeTab === tab.id}
    <button
      onclick={() => (appStore.activeTab = tab.id)}
      class="flex items-center space-x-1.5 px-3 py-1.5 rounded-md text-xs font-semibold transition-all {isActive
        ? 'bg-[#ffffff] dark:bg-[#18181c] text-[#0284c7] dark:text-[#00c8ff] border border-[#cbd5e1] dark:border-[#00c8ff]/30 shadow-xs'
        : 'text-[#64748b] dark:text-[#a1a1aa] hover:text-[#0f172a] dark:hover:text-[#f4f4f6] hover:bg-[#e2e8f0] dark:hover:bg-[#121215] border border-transparent'}"
    >
      <tab.icon size={13} weight={isActive ? 'fill' : 'regular'} />
      <span>{t(tab.key)}</span>
    </button>
  {/each}
</div>
