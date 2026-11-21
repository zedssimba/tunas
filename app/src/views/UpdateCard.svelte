<script>
  import Icon from '../lib/Icon.svelte';
  import { upd, updateReady, later, whatsNew, updateNow } from '../lib/update.svelte.js';
  import { fmtBytes } from '../lib/api.js';
</script>

{#if updateReady() || upd.phase}
  <div class="upd">
    <Icon name="download" size={16} />
    <div class="grow">
      <b>TUNAS {upd.info.latest} is ready</b>
      {#if upd.phase === 'downloading'}
        <span class="small mute"> · downloading {fmtBytes(upd.done)}{upd.total ? ` of ${fmtBytes(upd.total)}` : ''}</span>
      {:else if upd.phase === 'installing'}
        <span class="small mute"> · opening the installer…</span>
      {:else}
        <span class="small mute"> · you have {upd.info.current}</span>
      {/if}
    </div>
    {#if !upd.phase}
      {#if upd.info.notes_url}<button class="btn sm ghost" onclick={whatsNew}>What's new</button>{/if}
      <button class="btn sm ghost" onclick={later}>Later</button>
      <button class="btn sm primary" onclick={updateNow}>Update</button>
    {/if}
  </div>
{/if}

<style>
  .upd { display: flex; align-items: center; gap: 10px; padding: 8px 14px; background: color-mix(in srgb, var(--brand1) 9%, var(--panel)); border-bottom: 1px solid color-mix(in srgb, var(--brand1) 25%, var(--line)); color: var(--brand1); flex: none; }
  .upd b { color: var(--ink); font-weight: 600; }
  @media (max-width: 760px) { .upd { flex-wrap: wrap; padding: 8px 10px; } .upd .grow { flex-basis: 70%; } }
</style>
