<script>
  import Icon from '../lib/Icon.svelte';
  import { api, fmtBytes } from '../lib/api.js';
  import { app, toast } from '../lib/state.svelte.js';
  import Folders from './Folders.svelte';
  import { kn, refreshKnowledge, newSetFromFolder, addToSet, setLabel, baseName, ensureSearchModel, EMBED_MODEL } from '../lib/knowledge.svelte.js';

  let renaming = $state(null);
  let newName = $state('');
  let confirmDel = $state(null);
  let failed = $state({});
  const dl = $derived(app.progress[EMBED_MODEL]);

  async function rename(s) {
    if (newName.trim()) await api.knowledge_rename({ id: s.id, name: newName });
    renaming = null;
    refreshKnowledge();
  }
  async function del(s) {
    await api.knowledge_delete({ id: s.id });
    confirmDel = null;
    toast(`"${s.name}" deleted. Your files were not touched.`, 'ok');
    refreshKnowledge();
  }
  async function showFailed(s) {
    if (failed[s.id]) { failed = { ...failed, [s.id]: null }; return; }
    failed = { ...failed, [s.id]: await api.knowledge_failed({ id: s.id }) };
  }
  async function reindex(s) {
    await api.knowledge_reindex({ id: s.id, force: false });
    toast('Checking for new and changed files…');
    refreshKnowledge();
  }
  const CLOUD = /(my drive|google ?drive|googledrive|onedrive|dropbox|icloud)/i;
  const onCloud = (s) => s.sources.some((p) => CLOUD.test(p));
  async function removeSource(s, p) {
    await api.knowledge_remove_source({ id: s.id, path: p });
    refreshKnowledge();
  }
</script>

<div class="page">
  <div class="page-head">
    <div class="grow">
      <h1>Knowledge</h1>
      <p>{app.info?.os === 'android' ? 'Your own documents that your assistants can answer from. TUNAS keeps a copy of the documents you pick on this device. Nothing is sent anywhere.' : 'Folders of your own documents that your assistants can answer from. TUNAS reads them where they are on this machine. Nothing is copied or sent anywhere.'}</p>
    </div>
    <button class="btn primary" onclick={() => newSetFromFolder()}><Icon name="plus" size={16} /> {app.info?.os === 'android' ? 'New from documents' : 'New from folder'}</button>
  </div>

  {#if !kn.model_ready && kn.sets.length}
    <div class="card note">
      <Icon name="download" size={18} />
      <div class="grow">
        <b>{dl ? 'Downloading the search model' : 'The search model is needed to read your files'}</b>
        <div class="small mute">This is TUNAS's reading model (about 640 MB, once), not your files: your files are read where they are. After this download everything works offline, and your knowledge sets start reading by themselves.</div>
        {#if dl}<div class="bar"><i style="width:{Math.round((dl.done / (dl.total || 1)) * 100)}%"></i></div>{/if}
      </div>
      {#if !dl}<button class="btn sm primary" onclick={ensureSearchModel}>Download</button>{/if}
    </div>
  {/if}

  {#if !kn.sets.length}
    <div class="empty card">
      <Icon name="book" size={30} />
      <h3>No knowledge yet</h3>
      <p class="mute">Choose a folder, for example your circulars, SOPs, lecture notes or a tender folder. Then add it to an assistant in <b>My assistants</b>, or pick it in a chat with the <b>#</b> button.</p>
      <button class="btn primary" onclick={() => newSetFromFolder()}><Icon name="folder" size={16} /> Choose a folder</button>
    </div>
  {/if}

  <div class="list">
    {#each kn.sets as s (s.id)}
      <div class="card set">
        <div class="row top">
          <span class="ico"><Icon name="book" size={18} /></span>
          <div class="grow">
            {#if renaming === s.id}
              <div class="row"><input type="text" bind:value={newName} onkeydown={(e) => e.key === 'Enter' && rename(s)} /><button class="btn sm primary" onclick={() => rename(s)}>Save</button><button class="btn sm" onclick={() => (renaming = null)}>Cancel</button></div>
            {:else}
              <b>{s.name}</b>
            {/if}
            <div class="small mute">
              <span class="state {s.state}">{setLabel(s)}</span>
              {#if s.bytes} · {fmtBytes(s.bytes)}{/if}
              {#if s.current && s.state === 'indexing'} · <span class="faint">{s.current}</span>{/if}
            </div>
            {#if s.state === 'indexing' && s.files}
              <div class="bar"><i style="width:{Math.round(((s.done + s.failed) / s.files) * 100)}%"></i></div>
            {/if}
          </div>
          <div class="acts">
            <button class="icon-btn" title="Rename" onclick={() => { renaming = s.id; newName = s.name; }}><Icon name="edit" size={15} /></button>
            <button class="icon-btn" title="Check for new and changed files" onclick={() => reindex(s)}><Icon name="refresh" size={15} /></button>
            {#if s.state === 'indexing' || s.state === 'paused'}
              <button class="btn sm" onclick={() => api.knowledge_pause({ id: s.id, paused: !s.paused }).then(refreshKnowledge)}>{s.paused ? 'Resume' : 'Pause'}</button>
            {/if}
            {#if confirmDel === s.id}
              <button class="btn sm danger" onclick={() => del(s)}>Delete</button>
              <button class="btn sm" onclick={() => (confirmDel = null)}>Keep</button>
            {:else}
              <button class="icon-btn" title="Delete this knowledge set (your files stay)" onclick={() => (confirmDel = s.id)}><Icon name="trash" size={15} /></button>
            {/if}
          </div>
        </div>

        <div class="sources">
          {#each s.sources as p}
            <div class="src">
              <Icon name="folder" size={14} />
              <span class="grow ellipsis" title={p}>{p}</span>
              <button class="link small" onclick={() => removeSource(s, p)}>Remove</button>
            </div>
          {/each}
          <div class="row small" style="gap:12px;margin-top:6px">
            {#if app.info?.os !== 'android'}<button class="link" onclick={() => addToSet(s.id, 'folder')}><Icon name="plus" size={13} /> Add folder</button>{/if}
            <button class="link" onclick={() => addToSet(s.id, 'files')}><Icon name="plus" size={13} /> {app.info?.os === 'android' ? 'Add documents' : 'Add files'}</button>
          </div>
        </div>

        {#if onCloud(s)}
          <div class="cloudnote small mute"><Icon name="info" size={13} /> This folder is in a cloud drive. The first time TUNAS reads a file, the drive app may download it to this machine. To make reading faster, mark the folder "Available offline".</div>
        {/if}
        <div class="foot small">
          <span class="mute">Used by: {s.assistants.length ? s.assistants.join(', ') : 'no assistant yet (pick it in a chat with #, or add it in My assistants)'}</span>
          {#if s.failed}
            <button class="link warn" onclick={() => showFailed(s)}><Icon name="warn" size={13} /> {s.failed} file{s.failed === 1 ? '' : 's'} could not be read</button>
          {/if}
        </div>
        {#if failed[s.id]}
          <div class="failed small">
            {#each failed[s.id] as f}<div><b title={f.path}>{baseName(f.path)}</b> <span class="mute">{f.error}</span></div>{/each}
          </div>
        {/if}
      </div>
    {/each}
  </div>

  {#if kn.sets.length}
    <p class="small faint" style="margin-top:18px">
      Reading runs in the background at low priority and waits while an answer is being written.
      Share knowledge with your whole team with TERAS.
    </p>
  {/if}
  {#if app.info?.os !== 'android'}<Folders embedded={true} />{/if}
</div>

<style>
  .list { display: flex; flex-direction: column; gap: 12px; }
  .set { padding: 16px 18px; }
  .top { align-items: flex-start; gap: 12px; }
  .ico { width: 34px; height: 34px; border-radius: 10px; display: grid; place-items: center; background: var(--soft); color: var(--brand1); flex: none; }
  .acts { display: flex; gap: 4px; align-items: center; }
  .state.ready { color: var(--ok, #2f855a); }
  .state.indexing { color: var(--brand1); }
  .state.waiting_model, .state.paused { color: var(--warn, #b7791f); }
  .bar { height: 5px; border-radius: 9px; background: var(--soft); margin-top: 8px; overflow: hidden; max-width: 420px; }
  .bar i { display: block; height: 100%; background: linear-gradient(90deg, var(--brand1), var(--brand2)); transition: width .4s; }
  .sources { margin: 12px 0 0 46px; }
  .src { display: flex; gap: 8px; align-items: center; font-size: 13px; color: var(--mute); padding: 3px 0; }
  .foot { margin: 10px 0 0 46px; display: flex; gap: 14px; flex-wrap: wrap; align-items: center; }
  .failed { margin: 8px 0 0 46px; padding: 10px 12px; background: var(--soft); border-radius: 10px; max-height: 220px; overflow-y: auto; display: flex; flex-direction: column; gap: 4px; }
  .link { color: var(--brand1); display: inline-flex; gap: 4px; align-items: center; font-weight: 600; }
  .link.warn { color: var(--warn, #b7791f); }
  .cloudnote { margin: 8px 0 0 46px; display: flex; gap: 6px; align-items: flex-start; }
  .note { display: flex; gap: 12px; align-items: center; padding: 14px 16px; margin-bottom: 14px; border-color: var(--brand1); }
  .empty { text-align: center; padding: 40px 30px; display: flex; flex-direction: column; align-items: center; gap: 10px; }
  .empty p { max-width: 520px; }
</style>
