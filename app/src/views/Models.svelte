<script>
  import { onMount } from 'svelte';
  import Icon from '../lib/Icon.svelte';
  import { api, pickFolder, pickFile, fmtBytes, fmtEta, openUrl } from '../lib/api.js';
  import { speedTest } from '../lib/chat.js';
  import { app, toast, refreshModels, setPref, modelName } from '../lib/state.svelte.js';

  let hw = $state(null);
  let checking = $state(false);
  let testing = $state(false);
  let speed = $state(null);
  let importing = $state(false);
  let report = $state(null);
  let customName = $state('');
  let confirmRemove = $state(null);
  onMount(async () => {
    const ids = app.installed.map((m) => m.id).filter((id) => !id.includes('embed'));
    const vals = await Promise.all(ids.map((id) => api.setting_get({ key: `think:${id}` })));
    const tm = { ...app.thinkModes };
    ids.forEach((id, i) => (tm[id] = vals[i] || 'off'));
    app.thinkModes = tm;
    const lens = await Promise.all(ids.map((id) => api.setting_get({ key: `think_len:${id}` })));
    const tl = { ...app.thinkLens };
    ids.forEach((id, i) => (tl[id] = lens[i] || 'normal'));
    app.thinkLens = tl;
  });
  async function setLen(id, v) {
    app.thinkLens = { ...app.thinkLens, [id]: v };
    await api.setting_set({ key: `think_len:${id}`, value: v });
  }
  async function setThink(id, v) {
    app.thinkModes = { ...app.thinkModes, [id]: v };
    await api.setting_set({ key: `think:${id}`, value: v });
  }

  const chatCatalog = $derived(app.catalog.filter((m) => m.role === 'chat'));
  const customs = $derived(app.installed.filter((m) => m.source === 'custom'));
  const defaultId = $derived(app.prefs.default_model || app.installed.find((m) => !m.id.includes('embed'))?.id || '');

  async function check() {
    checking = true;
    try { hw = await api.hardware_check(); } finally { checking = false; }
  }

  async function runSpeed() {
    testing = true;
    speed = null;
    try {
      const st = await speedTest(defaultId);
      speed = { ...st, model: app.engine?.model_id || defaultId, engine: app.engine?.engine };
    } catch (e) {
      toast(String(e.message || e), 'error', 8000);
    } finally { testing = false; }
  }

  function verdictFor(s) {
    if (!s) return '';
    if (s.tps >= 12) return 'Fast: comfortable for everyday use.';
    if (s.tps >= 6) return 'Good: readable as it writes.';
    if (s.tps >= 3) return 'Slow but usable. A smaller model will feel quicker.';
    return 'Very slow. Use the smallest model on this machine.';
  }

  async function importFolder() {
    const dir = await pickFolder('Choose the folder or USB stick with TUNAS models');
    if (!dir) return;
    importing = true;
    report = null;
    try {
      report = await api.model_import_folder({ path: dir });
      await refreshModels();
      if (!report.imported.length && !report.failed.length && !report.skipped.length) toast('No model files were found in that folder.');
    } catch (e) { toast(String(e), 'error'); }
    finally { importing = false; }
  }

  async function addCustom() {
    const f = await pickFile([{ name: 'GGUF model', extensions: ['gguf'] }], 'Choose a GGUF model file');
    if (!f) return;
    importing = true;
    try {
      const m = await api.model_add_custom({ path: f, name: customName });
      customName = '';
      await refreshModels();
      toast(`${m.name} added.`, 'ok');
    } catch (e) { toast(String(e), 'error'); }
    finally { importing = false; }
  }

  async function exportModels() {
    const dir = await pickFolder('Choose a folder or USB stick to copy the models to');
    if (!dir) return;
    importing = true;
    try {
      const done = await api.models_export({ path: dir });
      toast(done.length ? `Copied: ${done.join(', ')}.` : 'There are no catalogue models to copy yet.', 'ok', 6000);
    } catch (e) { toast(String(e), 'error'); }
    finally { importing = false; }
  }

  async function remove(id) {
    confirmRemove = null;
    try {
      await api.model_remove({ id });
      if (app.prefs.default_model === id) await setPref('default_model', '');
      await refreshModels();
      toast('Model removed.');
    } catch (e) { toast(String(e), 'error'); }
  }

  const tierLabel = { fast: 'Fast', better: 'Better quality', balanced: 'Balanced', tiny: 'Very small', regional: 'Regional languages' };

  onMount(check);
</script>

<div class="page">
  <div class="page-head">
    <div class="grow">
      <h1>Models</h1>
      <p>Models run on this machine, fully offline. Download them once, or bring them on a USB stick.</p>
    </div>
  </div>

  <div class="card nb">
    <div class="row" style="margin-bottom:14px">
      <Icon name="cpu" size={20} />
      <h2 class="grow">Check my machine</h2>
      <button class="btn sm" onclick={check} disabled={checking}><Icon name="refresh" size={14} /> {checking ? 'Checking…' : 'Check again'}</button>
    </div>
    {#if hw}
      <div class="specs">
        <div class="spec"><Icon name="memory" /><div><b>{hw.ram_gb} GB</b><span>Memory ({hw.ram_free_gb} GB free now)</span></div></div>
        <div class="spec"><Icon name="cpu" /><div><b class="ellipsis" title={hw.cpu_name}>{hw.cpu_name || 'Processor'}</b><span>{hw.cpu_cores} cores, {hw.cpu_threads} threads</span></div></div>
        <div class="spec"><Icon name="chip" /><div><b class="ellipsis">{hw.gpus.map((g) => g.name).join(', ') || 'No graphics found'}</b><span>{hw.vulkan_devices.length ? 'Usable for AI' : 'Processor will be used'}</span></div></div>
        <div class="spec"><Icon name="disk" /><div><b>{hw.disk_free_gb} GB</b><span>Free disk space</span></div></div>
      </div>
      <div class="verdict">
        <Icon name="info" size={16} />
        <div class="grow">
          <div>{hw.verdict}</div>
          {#each hw.notes as n}<div class="small mute">{n}</div>{/each}
        </div>
      </div>
      <div class="row" style="margin-top:12px">
        <button class="btn sm" onclick={runSpeed} disabled={testing || !defaultId}><Icon name="speed" size={14} /> {testing ? 'Testing…' : 'Test speed'}</button>
        {#if speed}
          <span class="small">
            <b>{modelName(speed.model)}</b> on the {speed.engine === 'vulkan' ? 'graphics' : 'processor'}:
            first word in {(speed.first_ms / 1000).toFixed(1)} s, {speed.tps} tokens/s. <span class="mute">{verdictFor(speed)}</span>
          </span>
        {:else if !defaultId}
          <span class="small faint">Install a model first.</span>
        {/if}
      </div>
    {:else}
      <p class="mute">Reading your machine…</p>
    {/if}
  </div>

  <div class="section-title">Catalogue</div>
  <div class="list">
    {#each chatCatalog as m (m.id)}
      {@const p = app.progress[m.id]}
      {@const rec = hw?.recommended_models?.includes(m.id)}
      <div class="card tight model">
        <div class="row" style="align-items:flex-start">
          <div class="grow">
            <div class="row" style="flex-wrap:wrap">
              <b>{m.name}</b>
              <span class="pill">{tierLabel[m.tier] || m.tier}</span>
              {#if rec}<span class="pill brand">Recommended for this machine</span>{/if}
              {#if m.installed && defaultId === m.id}<span class="pill gold">Default</span>{/if}
              {#if hw && m.min_ram_gb > hw.ram_gb + 0.5}<span class="pill warn">Needs {m.min_ram_gb} GB</span>{/if}
            </div>
            <div class="mute small" style="margin-top:4px">{m.description}</div>
            <div class="faint small" style="margin-top:4px">
              {m.family} · {m.params} · {fmtBytes(m.size)} · Licence: {m.license}
              · <button class="link" onclick={() => openUrl(m.source_page)}>Source</button>
            </div>
            {#if m.license_note}<div class="small" style="color:var(--warn);margin-top:4px">{m.license_note}</div>{/if}
          </div>
          <div class="acts">
            {#if p && !['done', 'error', 'cancelled'].includes(p.phase)}
              <button class="btn sm" onclick={() => api.model_cancel({ id: m.id })}>Cancel</button>
            {:else if m.installed}
              {#if !m.id.includes('embed')}<select class="thinksel" title="Thinking before answering" value={app.thinkModes[m.id] || 'off'} onchange={(e) => setThink(m.id, e.currentTarget.value)}><option value="off">Thinking off</option><option value="hidden">Thinking, hidden</option><option value="shown">Thinking, shown</option></select>{#if (app.thinkModes[m.id] || 'off') !== 'off'}<select class="thinksel" title="How long to think" value={app.thinkLens[m.id] || 'normal'} onchange={(e) => setLen(m.id, e.currentTarget.value)}><option value="short">Short thinking</option><option value="normal">Normal thinking</option><option value="long">Long thinking</option></select>{/if}{/if}
              {#if defaultId !== m.id}<button class="btn sm" onclick={() => setPref('default_model', m.id)}>Make default</button>{/if}
              {#if confirmRemove === m.id}
                <button class="btn sm danger" onclick={() => remove(m.id)}>Remove</button>
                <button class="btn sm ghost" onclick={() => (confirmRemove = null)}>Keep</button>
              {:else}
                <button class="icon-btn" title="Remove from this machine" onclick={() => (confirmRemove = m.id)}><Icon name="trash" size={16} /></button>
              {/if}
            {:else}
              <button class="btn sm primary" onclick={() => api.model_download({ id: m.id })}><Icon name="download" size={14} /> Download</button>
            {/if}
          </div>
        </div>
        {#if p && !['done', 'error', 'cancelled'].includes(p.phase)}
          <div class="prog">
            <div class="bar"><i style="width:{p.total ? (100 * p.received) / p.total : 0}%"></i></div>
            <div class="small mute row">
              <span>{p.phase === 'verifying' ? 'Checking the file…' : p.phase === 'copying' ? 'Copying…' : 'Downloading…'}</span>
              <span>{fmtBytes(p.received)} of {fmtBytes(p.total)}</span>
              {#if p.bytes_per_sec && p.phase !== 'verifying'}<span>{fmtBytes(p.bytes_per_sec)}/s · {fmtEta(p.total - p.received, p.bytes_per_sec)}</span>{/if}
              {#if p.message && p.phase === 'downloading'}<span class="faint">from {p.message}</span>{/if}
            </div>
          </div>
        {/if}
      </div>
    {/each}
  </div>

  {#if customs.length}
    <div class="section-title">Your own models</div>
    <div class="list">
      {#each customs as m (m.id)}
        <div class="card tight row">
          <Icon name="models" />
          <div class="grow"><b>{m.name}</b><div class="small faint">{fmtBytes(m.size)} · Added by you</div></div>
          <select class="thinksel" title="Thinking before answering" value={app.thinkModes[m.id] || 'off'} onchange={(e) => setThink(m.id, e.currentTarget.value)}><option value="off">Thinking off</option><option value="hidden">Thinking, hidden</option><option value="shown">Thinking, shown</option></select>{#if (app.thinkModes[m.id] || 'off') !== 'off'}<select class="thinksel" title="How long to think" value={app.thinkLens[m.id] || 'normal'} onchange={(e) => setLen(m.id, e.currentTarget.value)}><option value="short">Short thinking</option><option value="normal">Normal thinking</option><option value="long">Long thinking</option></select>{/if}
          {#if defaultId !== m.id}<button class="btn sm" onclick={() => setPref('default_model', m.id)}>Make default</button>{:else}<span class="pill gold">Default</span>{/if}
          {#if confirmRemove === m.id}
            <button class="btn sm danger" onclick={() => remove(m.id)}>Remove</button>
          {:else}
            <button class="icon-btn" title="Remove" onclick={() => (confirmRemove = m.id)}><Icon name="trash" size={16} /></button>
          {/if}
        </div>
      {/each}
    </div>
  {/if}

  <div class="section-title">Offline and advanced</div>
  <div class="tri">
    <div class="card tight col">
      <div class="row"><Icon name="folder" /><b>Import from a folder or USB stick</b></div>
      <p class="small mute">For machines without internet. TUNAS finds catalogue models in the folder, checks each file and copies it in.</p>
      <button class="btn sm" disabled={importing} onclick={importFolder}>Choose folder…</button>
    </div>
    <div class="card tight col">
      <div class="row"><Icon name="upload" /><b>Prepare a USB stick</b></div>
      <p class="small mute">Copy the models on this machine to a folder, to install them on other machines without downloading.</p>
      <button class="btn sm" disabled={importing} onclick={exportModels}>Copy models to…</button>
    </div>
    <div class="card tight col">
      <div class="row"><Icon name="file" /><b>Add your own model (GGUF)</b></div>
      <p class="small mute">Any GGUF model file. You are responsible for its licence; large models may be slow on this machine.</p>
      <div class="row"><input type="text" placeholder="Name (optional)" bind:value={customName} /><button class="btn sm" disabled={importing} onclick={addCustom}>Choose file…</button></div>
    </div>
  </div>

  {#if report}
    <div class="card tight" style="margin-top:12px">
      <b>Import result</b>
      {#each report.imported as x}<div class="small" style="color:var(--ok)">Installed: {x}</div>{/each}
      {#each report.skipped as x}<div class="small mute">Skipped: {x}</div>{/each}
      {#each report.failed as x}<div class="small" style="color:var(--err)">Failed: {x}</div>{/each}
    </div>
  {/if}
</div>

<style>
  .thinksel { width: auto; padding: 5px 8px; font-size: 12.5px; }
  .specs { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; }
  .spec { display: flex; gap: 10px; align-items: flex-start; padding: 12px; border-radius: 11px; background: var(--soft); min-width: 0; color: var(--mute); }
  .spec div { display: flex; flex-direction: column; min-width: 0; }
  .spec b { color: var(--ink); }
  .spec span { font-size: 12px; }
  .verdict { display: flex; gap: 10px; margin-top: 12px; padding: 12px; border-radius: 11px; border: 1px solid color-mix(in srgb, var(--brand1) 25%, var(--line)); background: color-mix(in srgb, var(--brand1) 5%, transparent); }
  .list { display: flex; flex-direction: column; gap: 10px; }
  .acts { display: flex; gap: 6px; align-items: center; flex: none; }
  .prog { margin-top: 12px; display: flex; flex-direction: column; gap: 6px; }
  .prog .row { gap: 14px; }
  .tri { display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px; }
  .tri .col { gap: 10px; }
  .tri p { flex: 1; }
  .link { color: var(--brand1); padding: 0; }
  .link:hover { text-decoration: underline; }
  @media (max-width: 1100px) { .specs { grid-template-columns: 1fr 1fr; } .tri { grid-template-columns: 1fr; } }
</style>
