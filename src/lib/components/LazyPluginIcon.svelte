<script>
  import { PuzzlePiece } from "phosphor-svelte";

  let { iconData = null, name = "", size = 16 } = $props();

  let isVisible = $state(false);
  let el = $state(null);

  $effect(() => {
    if (!el) return;
    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          isVisible = entry.isIntersecting;
        }
      },
      {
        rootMargin: "60px 0px 60px 0px",
        threshold: 0.01,
      }
    );

    observer.observe(el);

    return () => {
      observer.disconnect();
    };
  });
</script>

<div
  bind:this={el}
  class="w-4 h-4 shrink-0 flex items-center justify-center overflow-hidden"
  style="min-width: {size}px; min-height: {size}px;"
>
  {#if isVisible && iconData}
    <img
      src={iconData}
      alt={name}
      class="w-full h-full object-contain rounded shrink-0 select-none pointer-events-none"
      decoding="async"
    />
  {:else}
    <PuzzlePiece size={13} weight="bold" class="text-sky-600 dark:text-[#00c8ff] shrink-0 opacity-70" />
  {/if}
</div>
