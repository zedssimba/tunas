<script>
  import Mark from './Mark.svelte';
  let { src = '', size = 32, rounded = true } = $props();
</script>

{#if src}
  <img {src} alt="" style="width:{size}px;height:{size}px" class:rounded />
{:else}
  <Mark {size} />
{/if}

<style>
  img { object-fit: contain; flex: none; }
  img.rounded { border-radius: 8px; }
</style>
