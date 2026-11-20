<script>
  import { onMount } from 'svelte';
  import Icon from '../lib/Icon.svelte';
  import { api, pickFolder } from '../lib/api.js';
  import { toast } from '../lib/state.svelte.js';
  import { askConsent, refreshKnowledge } from '../lib/knowledge.svelte.js';

  let list = $state([]);
  let confirm = $state(null);
  async function load() { list = await api.folders_list(); }
  onMount(load);

  async function add() {
    const p = await pickFolder('Choose a folder TUNAS may read');
    if (p && (await askConsent([p]))) load();
  }
  async function revoke(g) {
    await api.folder_revoke({ path: g.path });
    confirm = null;
    toast('TUNAS will no longer read this place. Its search index was deleted; your files are untouched.', 'ok', 6000);
    load();
    refreshKnowledge();
  }
</script>

<div class="page">
  <div class="page-head">
    <div class="grow">
      <h1>Folders</h1>
      <p>The places on this machine TUNAS is allowed to read for Knowledge. TUNAS reads files where they are; it never copies, changes or deletes them.</p>
    </div>
    <button class="btn" onclick={add}><Icon name="plus" size={16} /> Allow a folder</button>
  </div>

  {#if !list.length}
    <div class="card" style="padding:22px"><p class="mute">No folders yet. When you make a knowledge set, TUNAS asks before reading the folder, and it appears here.</p></div>
  {/if}

  <div class="card listc">
    {#each list as g (g.path)}
      <div class="row item">
        <Icon name={g.is_dir ? 'folder' : 'file'} size={18} />
        <div class="grow" style="min-width:0">
          <div class="ellipsis" title={g.path}><b>{g.path}</b></div>
          <div class="small mute">
            {#if !g.exists}<span style="color:var(--warn)">Not found (moved, renamed or on a drive that is not connected).</span>{/if}
            {g.used_by.length ? `Used by: ${g.used_by.join(', ')}` : 'Not used by a knowledge set'}
          </div>
        </div>
        {#if confirm === g.path}
          <button class="btn sm danger" onclick={() => revoke(g)}>Stop reading</button>
          <button class="btn sm" onclick={() => (confirm = null)}>Keep</button>
        {:else}
          <button class="btn sm" onclick={() => (confirm = g.path)}>Remove</button>
        {/if}
      </div>
    {/each}
  </div>
  <p class="small faint" style="margin-top:14px">Removing a place also removes it from every knowledge set that uses it.</p>
</div>

<style>
  .listc { padding: 4px 0; }
  .listc:empty { display: none; }
  .item { padding: 12px 16px; gap: 12px; border-bottom: 1px solid var(--line); }
  .item:last-child { border-bottom: 0; }
</style>
