<script>
  import { ICONS } from './icons.js';
  let { name = 'chat', size = 18, stroke = 1.8, class: cls = '' } = $props();
</script>

<svg class="icon {cls}" width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor"
  stroke-width={stroke} stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
  {@html ICONS[name] ?? ICONS.chat}
</svg>

<style>
  .icon { flex: none; display: inline-block; vertical-align: middle; }
</style>
