<script>
  import { onMount } from 'svelte';
  import Icon from '../lib/Icon.svelte';
  import { api } from '../lib/api.js';
  import { app, toast, refreshProviders, setPref } from '../lib/state.svelte.js';

  const KINDS = [
    ['openai', 'OpenAI'],
    ['anthropic', 'Anthropic (Claude)'],
    ['gemini', 'Google Gemini'],
    ['openrouter', 'OpenRouter'],
    ['custom', 'Other (OpenAI-compatible)'],
  ];

  const teras = $derived(app.providers.find((p) => p.kind === 'teras'));
  const clouds = $derived(app.providers.filter((p) => p.kind !== 'teras'));
  const cloudOn = $derived(app.prefs.cloud_enabled === '1');
  const cloudAllowed = $derived(app.brand?.policy?.allow_cloud !== false);

  // TERAS form
  let tUrl = $state('');
  let tKey = $state('');
  let tBusy = $state(false);
  let tMsg = $state('');

  // Cloud editor
  let edit = $state(null); // { id, kind, name, base_url, enabled, models, key, available, filter, manual }
  let eBusy = $state(false);

  // Local API
  let api_ = $state({ enabled: false, port: 11435, running: false, allowed: true });
  let keys = $state([]);
  let newKeyName = $state('');
  let shownKey = $state(null);

  onMount(async () => {
    await refreshProviders();
    if (teras) tUrl = teras.base_url;
    api_ = await api.api_settings();
    keys = await api.app_keys_list();
  });

  async function connectTeras() {
    if (!tUrl.trim()) { toast('Enter your TERAS address, for example teras.myorg.gov.my', 'error'); return; }
    tBusy = true; tMsg = '';
    try {
      const r = await api.provider_save({
        provider: { id: teras?.id || '', kind: 'teras', name: teras?.name || 'TERAS', base_url: tUrl, enabled: true, models: teras?.models || [] },
        key: tKey ? tKey : (teras ? null : ''),
      });
      tKey = '';
      await refreshProviders();
      tUrl = r.provider.base_url;
      tMsg = r.error ? r.error : `Connected. ${r.provider.models.length} model${r.provider.models.length === 1 ? '' : 's'} from TERAS can now be used by your assistants.`;
      if (!r.error) toast('Connected to TERAS.', 'ok');
    } catch (e) { tMsg = String(e); }
    finally { tBusy = false; }
  }

  async function terasRefresh() {
    tBusy = true;
    try {
      const list = await api.provider_available({ id: teras.id });
      await api.provider_save({ provider: { ...teras, models: list }, key: null });
      await refreshProviders();
      tMsg = `${list.length} model${list.length === 1 ? '' : 's'} available from TERAS.`;
    } catch (e) { tMsg = String(e); }
    finally { tBusy = false; }
  }

  async function toggleProvider(p) {
    await api.provider_save({ provider: { ...p, enabled: !p.enabled }, key: null });
    await refreshProviders();
  }

  async function removeProvider(p) {
    await api.provider_delete({ id: p.id });
    if (p.kind === 'teras') { tUrl = ''; tMsg = ''; }
    await refreshProviders();
    toast(`${p.name} removed. Assistants that used it now use the default model.`);
  }

  function newCloud() {
    edit = { id: '', kind: 'openai', name: 'OpenAI', base_url: '', enabled: true, models: [], key: '', available: [], filter: '', manual: '', saved: false };
  }

  async function openCloud(p) {
    edit = { ...p, key: '', available: [], filter: '', manual: '', saved: true };
    eBusy = true;
    try { edit.available = await api.provider_available({ id: p.id }); } catch (e) { toast(String(e), 'error'); }
    finally { eBusy = false; }
  }

  async function saveCloud(withKey = true) {
    eBusy = true;
    try {
      const r = await api.provider_save({
        provider: { id: edit.id, kind: edit.kind, name: edit.name, base_url: edit.base_url, enabled: edit.enabled, models: edit.models },
        key: withKey && edit.key ? edit.key : (edit.id ? null : edit.key),
      });
      edit.id = r.provider.id; edit.key = ''; edit.saved = true;
      edit.base_url = r.provider.base_url;
      if (r.available.length) edit.available = r.available;
      if (r.error) toast(r.error, 'error', 8000);
      await refreshProviders();
    } catch (e) { toast(String(e), 'error'); }
    finally { eBusy = false; }
  }

  async function finishCloud() {
    if (!edit.models.length) { toast('Tick at least one model to use, or type a model name.', 'error'); return; }
    await saveCloud(false);
    toast(`${edit.name} saved. Its models can now be chosen as an assistant's base model.`, 'ok');
    edit = null;
  }

  function addManual() {
    const m = edit.manual.trim();
    if (m && !edit.models.includes(m)) edit.models = [...edit.models, m];
    edit.manual = '';
  }

  async function setCloud(on) {
    await setPref('cloud_enabled', on ? '1' : '0');
  }

  async function setApi(enabled) {
    try {
      await api.api_set({ enabled, port: Number(api_.port) || 11435 });
      api_ = await api.api_settings();
      toast(enabled ? `Other apps can now use TUNAS at http://127.0.0.1:${api_.port}/v1` : 'Other apps can no longer use TUNAS.', 'ok');
    } catch (e) { toast(String(e), 'error', 8000); api_ = await api.api_settings(); }
  }

  async function createKey() {
    try {
      const r = await api.app_key_create({ name: newKeyName });
      shownKey = r;
      newKeyName = '';
      keys = await api.app_keys_list();
    } catch (e) { toast(String(e), 'error'); }
  }

  async function deleteKey(k) {
    await api.app_key_delete({ id: k.id });
    keys = await api.app_keys_list();
    toast(`Key "${k.name}" removed. That app can no longer use TUNAS.`);
  }

  async function copy(t) { await navigator.clipboard.writeText(t); toast('Copied.', 'ok', 1500); }

  const filtered = $derived(edit ? edit.available.filter((m) => !edit.filter || m.toLowerCase().includes(edit.filter.toLowerCase())).slice(0, 200) : []);
  const fmt = (d) => (d ? new Date(d).toLocaleString() : 'never');
</script>

<div class="page">
  <div class="page-head"><div class="grow"><h1>Connections</h1><p>Use your organisation's TERAS or a cloud AI provider, and let other apps on this machine use TUNAS.</p></div></div>

  <!-- TERAS -->
  <div class="section-title">Your organisation's TERAS</div>
  <div class="card">
    <div class="row" style="align-items:flex-start;gap:14px">
      <div class="badge"><Icon name="server" size={22} /></div>
      <div class="grow col" style="gap:10px">
        <div>
          <b>Connect to TERAS</b>
          <div class="small mute">TERAS is the S.A.L.T CI core your organisation runs on its own servers. Once connected, its models appear as base models for your assistants. Messages go only to your organisation's server.</div>
        </div>
        {#if teras}
          <div class="row conn">
            <span class="pill" class:ok={teras.enabled}>{teras.enabled ? 'Connected' : 'Switched off'}</span>
            <span class="small ellipsis grow" title={teras.base_url}>{teras.base_url} · {teras.models.length} model{teras.models.length === 1 ? '' : 's'}{teras.has_key ? '' : ' · no key'}</span>
            <button class="btn sm" disabled={tBusy} onclick={terasRefresh}><Icon name="refresh" size={14} /> Refresh models</button>
            <button class="btn sm" onclick={() => toggleProvider(teras)}>{teras.enabled ? 'Switch off' : 'Switch on'}</button>
            <button class="btn sm ghost danger" onclick={() => removeProvider(teras)}><Icon name="trash" size={14} /></button>
          </div>
          {#if teras.models.length}<div class="chips">{#each teras.models as m}<span class="pill">{m}</span>{/each}</div>{/if}
        {/if}
        <div class="grid2">
          <label class="field">TERAS address <span class="hint">As given by your IT department, for example teras.myorg.gov.my</span>
            <input type="text" bind:value={tUrl} placeholder="https://teras.myorg.gov.my" />
          </label>
          <label class="field">TERAS key <span class="hint">{teras?.has_key ? 'Saved. Leave empty to keep it.' : 'Your personal API key from TERAS.'}</span>
            <input type="password" bind:value={tKey} placeholder={teras?.has_key ? '••••••••••••' : 'Paste the key'} autocomplete="off" />
          </label>
        </div>
        <div class="row">
          <button class="btn primary sm" disabled={tBusy} onclick={connectTeras}>{tBusy ? 'Connecting…' : teras ? 'Save and reconnect' : 'Connect'}</button>
          {#if tMsg}<span class="small" class:mute={!tMsg.includes('Could not') && !tMsg.includes('refused')} style={tMsg.includes('Could not') || tMsg.includes('refused') ? 'color:var(--err)' : ''}>{tMsg}</span>{/if}
        </div>
      </div>
    </div>
  </div>

  <!-- Cloud -->
  <div class="section-title">Cloud AI providers</div>
  <div class="card">
    <div class="row" style="align-items:flex-start;gap:14px">
      <div class="badge warn"><Icon name="globe" size={22} /></div>
      <div class="grow col" style="gap:10px">
        <div class="row" style="align-items:flex-start">
          <div class="grow">
            <b>Allow cloud providers</b>
            <div class="small mute">Off by default. When an assistant uses a cloud model, what you type in that chat, and any files you add, are sent to that provider over the internet. Keys are kept protected by {app.info?.os === 'macos' ? 'the macOS Keychain' : 'Windows'} on this machine.</div>
          </div>
          <label class="switch" title={cloudAllowed ? '' : 'Not allowed by your organisation'}>
            <input type="checkbox" checked={cloudOn} disabled={!cloudAllowed} onchange={(e) => setCloud(e.target.checked)} /><span></span>
          </label>
        </div>
        {#if !cloudAllowed}<div class="small" style="color:var(--warn)"><Icon name="lock" size={13} /> Your organisation does not allow cloud providers on this machine.</div>{/if}
        {#if cloudOn && cloudAllowed}
          {#each clouds as p (p.id)}
            <div class="row conn">
              <span class="pill" class:ok={p.enabled}>{p.enabled ? 'On' : 'Off'}</span>
              <b>{p.name}</b>
              <span class="small faint ellipsis grow">{p.models.length} model{p.models.length === 1 ? '' : 's'} · {p.base_url}</span>
              <button class="btn sm" onclick={() => openCloud(p)}>Models &amp; key</button>
              <button class="btn sm" onclick={() => toggleProvider(p)}>{p.enabled ? 'Switch off' : 'Switch on'}</button>
              <button class="btn sm ghost danger" onclick={() => removeProvider(p)}><Icon name="trash" size={14} /></button>
            </div>
          {/each}
          <div><button class="btn sm" onclick={newCloud}><Icon name="plus" size={14} /> Add a provider</button></div>
        {/if}
      </div>
    </div>
  </div>

  <!-- Local API -->
  {#if app.info?.os !== 'android'}
  <div class="section-title">Apps that use TUNAS</div>
  <div class="card">
    <div class="row" style="align-items:flex-start;gap:14px">
      <div class="badge"><Icon name="code" size={22} /></div>
      <div class="grow col" style="gap:10px">
        <div class="row" style="align-items:flex-start">
          <div class="grow">
            <b>Let other apps use TUNAS</b>
            <div class="small mute">Other apps on this machine, such as S.A.L.T NEXUS, can use TUNAS as their AI engine through an OpenAI-compatible API. They see your assistants and models by name. Only this machine can connect, and each app needs a key.</div>
          </div>
          <label class="switch" title={api_.allowed ? '' : 'Not allowed by your organisation'}>
            <input type="checkbox" checked={api_.enabled} disabled={!api_.allowed} onchange={(e) => setApi(e.target.checked)} /><span></span>
          </label>
        </div>
        {#if !api_.allowed}<div class="small" style="color:var(--warn)"><Icon name="lock" size={13} /> Your organisation does not allow other apps to use TUNAS on this machine.</div>{/if}
        {#if api_.enabled && api_.allowed}
          <div class="row conn">
            <span class="pill" class:ok={api_.running} class:err={!api_.running}>{api_.running ? 'Running' : 'Not running'}</span>
            <span class="small">Address for apps:</span>
            <code>http://127.0.0.1:{api_.port}/v1</code>
            <button class="icon-btn" title="Copy" onclick={() => copy(`http://127.0.0.1:${api_.port}/v1`)}><Icon name="copy" size={14} /></button>
            <span class="spacer"></span>
            <span class="small faint">Port</span>
            <input type="text" class="port" bind:value={api_.port} onchange={() => setApi(true)} />
          </div>

          <div class="small" style="font-weight:600;margin-top:4px">App keys</div>
          {#each keys as k (k.id)}
            <div class="row conn">
              <Icon name="lock" size={14} />
              <b>{k.name}</b>
              <code class="faint">{k.hint}</code>
              <span class="small faint grow">Used {k.uses} time{k.uses === 1 ? '' : 's'} · last {fmt(k.last_used)}</span>
              <button class="btn sm ghost danger" onclick={() => deleteKey(k)}>Remove</button>
            </div>
          {:else}
            <div class="small faint">No keys yet. Make one for each app that should use TUNAS.</div>
          {/each}
          <div class="row">
            <input type="text" bind:value={newKeyName} placeholder="Which app? e.g. NEXUS desktop" style="max-width:300px" onkeydown={(e) => e.key === 'Enter' && createKey()} />
            <button class="btn sm primary" onclick={createKey}><Icon name="plus" size={14} /> Make key</button>
          </div>
          {#if shownKey}
            <div class="newkey">
              <div class="small"><b>Key for {shownKey.entry.name}.</b> Copy it now and paste it into the app. TUNAS will not show it again.</div>
              <div class="row"><code class="grow">{shownKey.key}</code><button class="btn sm" onclick={() => copy(shownKey.key)}><Icon name="copy" size={14} /> Copy</button><button class="btn sm ghost" onclick={() => (shownKey = null)}>Done</button></div>
            </div>
          {/if}
          <details class="small mute">
            <summary>How an app connects</summary>
            <div style="margin-top:6px">Use the address above as the OpenAI base URL and the key as the API key. <code>GET /v1/models</code> lists your assistants (for example <code>surat-rasmi</code>), local models (<code>qwen3.5-4b</code>) and connected provider models (<code>teras/model-name</code>). <code>POST /v1/chat/completions</code> answers, with streaming. Asking for an assistant applies its persona.</div>
          </details>
        {/if}
      </div>
    </div>
  </div>
  {/if}
</div>

{#if edit}
  <div class="overlay" role="presentation" onclick={(e) => e.target === e.currentTarget && (edit = null)}>
    <div class="drawer">
      <div class="row head"><h2 class="grow">{edit.saved ? edit.name : 'Add a cloud provider'}</h2><button class="icon-btn" aria-label="Close" onclick={() => (edit = null)}><Icon name="x" /></button></div>
      <div class="body col" style="gap:16px">
        <label class="field">Provider
          <select bind:value={edit.kind} disabled={edit.saved} onchange={() => (edit.name = KINDS.find((k) => k[0] === edit.kind)[1].replace(' (OpenAI-compatible)', ''))}>
            {#each KINDS as [k, l]}<option value={k}>{l}</option>{/each}
          </select>
        </label>
        <label class="field">Name<input type="text" bind:value={edit.name} maxlength="40" /></label>
        {#if edit.kind === 'custom'}
          <label class="field">Address (OpenAI-compatible base URL)<input type="text" bind:value={edit.base_url} placeholder="https://api.example.com/v1" /></label>
        {/if}
        <label class="field">API key <span class="hint">{edit.saved && edit.has_key ? 'Saved. Enter a new one only to replace it.' : `Your own key from the provider. It is stored protected by ${app.info?.os === 'macos' ? 'the macOS Keychain' : 'Windows'} on this machine.`}</span>
          <input type="password" bind:value={edit.key} autocomplete="off" placeholder={edit.saved && edit.has_key ? '••••••••••••' : 'Paste the key'} />
        </label>
        <div><button class="btn sm" disabled={eBusy} onclick={() => saveCloud(true)}>{eBusy ? 'Checking…' : edit.saved ? 'Save key and reload models' : 'Save and load models'}</button></div>

        {#if edit.saved}
          <div class="field">Models to use
            <span class="hint">Tick the models your assistants may use. They appear as base models in My assistants.</span>
            {#if edit.models.length}<div class="chips">{#each edit.models as m}<button class="pill brand" title="Remove" onclick={() => (edit.models = edit.models.filter((x) => x !== m))}>{m} <Icon name="x" size={11} /></button>{/each}</div>{/if}
            {#if edit.available.length}
              <input type="search" placeholder="Search {edit.available.length} models" bind:value={edit.filter} />
              <div class="mlist">
                {#each filtered as m}
                  <label class="chk"><input type="checkbox" checked={edit.models.includes(m)} onchange={(e) => (edit.models = e.target.checked ? [...edit.models, m] : edit.models.filter((x) => x !== m))} /> {m}</label>
                {/each}
              </div>
            {/if}
            <div class="row"><input type="text" bind:value={edit.manual} placeholder="Or type a model name" onkeydown={(e) => e.key === 'Enter' && addManual()} /><button class="btn sm" onclick={addManual}>Add</button></div>
          </div>
        {/if}
      </div>
      <div class="row foot">
        <div class="spacer"></div>
        <button class="btn" onclick={() => (edit = null)}>Cancel</button>
        <button class="btn primary" disabled={!edit.saved || eBusy} onclick={finishCloud}>Save</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .page { max-width: 1040px; }
  .badge { width: 44px; height: 44px; border-radius: 12px; display: grid; place-items: center; color: #fff; background: var(--grad); flex: none; }
  .badge.warn { background: linear-gradient(120deg, #D99A00, #F05A28); }
  .grid2 { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
  .conn { padding: 8px 10px; border: 1px solid var(--line); border-radius: 10px; background: var(--bg); flex-wrap: wrap; }
  .chips { display: flex; flex-wrap: wrap; gap: 6px; }
  code { font-family: 'Cascadia Mono', Consolas, monospace; font-size: 12.5px; background: var(--soft); padding: 2px 6px; border-radius: 6px; overflow-wrap: anywhere; }
  .port { width: 90px; padding: 4px 8px; }
  .newkey { padding: 12px; border-radius: 10px; border: 1px solid color-mix(in srgb, var(--ok) 40%, var(--line)); background: color-mix(in srgb, var(--ok) 6%, transparent); display: flex; flex-direction: column; gap: 8px; }
  details summary { cursor: pointer; }
  .overlay { position: fixed; inset: 0; background: rgba(11, 16, 34, .4); z-index: 40; display: flex; justify-content: flex-end; }
  .drawer { width: 560px; max-width: 100%; height: 100%; background: var(--panel); display: flex; flex-direction: column; box-shadow: var(--shadow); }
  .head { padding: 18px 22px; border-bottom: 1px solid var(--line); }
  .body { flex: 1; overflow-y: auto; padding: 20px 22px; }
  .foot { padding: 14px 22px; border-top: 1px solid var(--line); }
  .mlist { max-height: 260px; overflow-y: auto; border: 1px solid var(--line); border-radius: 9px; padding: 6px 10px; display: flex; flex-direction: column; gap: 4px; font-weight: 400; }
  .chk { display: flex; align-items: center; gap: 8px; font-size: 13px; cursor: pointer; }
  @media (max-width: 760px) { .grid2 { grid-template-columns: 1fr; } }
</style>
