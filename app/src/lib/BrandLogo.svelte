<script>
  import Mark from './Mark.svelte';
  // bg: '' (transparent, default) | 'light' | 'dark'; shape: 'square' | 'circle'
  let { src = '', size = 32, rounded = true, bg = '', shape = 'square' } = $props();
  const pad = $derived(Math.max(2, Math.round(size * 0.14)));
</script>

{#if src && bg}
  <span class="plate {bg}" style="width:{size}px;height:{size}px;padding:{pad}px;border-radius:{shape === 'circle' ? '50%' : Math.round(size * 0.22) + 'px'}">
    <img {src} alt="" />
  </span>
{:else if src}
  <img {src} alt="" style="width:{size}px;height:{size}px" class:rounded />
{:else}
  <Mark {size} />
{/if}

<style>
  img { object-fit: contain; flex: none; }
  img.rounded { border-radius: 8px; }
  .plate { display: inline-grid; place-items: center; flex: none; box-sizing: border-box; }
  .plate img { width: 100%; height: 100%; }
  .plate.light { background: #FFFFFF; box-shadow: 0 0 0 1px rgba(20, 26, 46, .10); }
  .plate.dark { background: #0B1022; box-shadow: 0 0 0 1px rgba(255, 255, 255, .10); }
</style>
