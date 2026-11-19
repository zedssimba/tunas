<script>
  import { onMount } from 'svelte';
  import Icon from '../lib/Icon.svelte';
  import BrandLogo from '../lib/BrandLogo.svelte';
  import SaltBrandStrip from '../lib/SaltBrandStrip.svelte';
  import { api, pickFile, pickFolder, fmtBytes, fmtEta } from '../lib/api.js';
  import { app, toast, refreshModels, refreshAssistants, setPref } from '../lib/state.svelte.js';

  let step = $state(0);
  let hw = $state(null);
  let picked = $state([]);
  let source = $state('online');
  let started = $state(false);
  let importing = $state(false);
  let importReport = $state(null);
  let brandFor = $state('org');
  let bname = $state('');
  let logoPath = $state(null);
  let logoPreview = $state('');
  let logoWarn = $state([]);
  let savingBrand = $state(false);

  const steps = ['Welcome', 'Your notebook', 'Models', 'Branding', 'Ready'];
  const bundled = $derived(app.catalog.filter((m) => m.role === 'chat' && m.bundle));
  const totalSize = $derived(bundled.filter((m) => picked.includes(m.id)).reduce((a, m) => a + m.size, 0));
  const hasModel = $derived(app.installed.some((m) => !m.id.includes('embed')));
  const oem = $derived(app.brand?.layers?.find((l) => l.layer === 'oem')?.brand);
  const active = $derived(Object.values(app.progress).filter((p) => !['done', 'error', 'cancelled'].includes(p.phase)));

  onMount(async () => {
    await refreshModels();
    hw = await api.hardware_check();
    // Pre-tick: the recommended bundled models, at most two.
    picked = hw.recommended_models.filter((id) => app.catalog.some((m) => m.id === id && m.bundle && !m.installed)).slice(0, 2);
  });

  function next() { step = Math.min(step + 1, steps.length - 1); }
  function back() { step = Math.max(step - 1, 0); }

  async function startModels() {
    if (source === 'online') {
      for (const id of picked) await api.model_download({ id });
      started = true;
      if (picked.length && !app.prefs.default_model) await setPref('default_model', picked[0]);
      next();
    } else {
      const dir = await pickFolder('Choose the folder or USB stick with TUNAS models');
      if (!dir) return;
      importing = true;
      try {
        importReport = await api.model_import_folder({ path: dir });
        await refreshModels();
        if (importReport.imported.length) { started = true; next(); }
        else toast('No TUNAS models were found in that folder.', 'error');
      } catch (e) { toast(String(e), 'error'); }
      finally { importing = false; }
    }
  }

  async function chooseLogo() {
    const f = await pickFile([{ name: 'Logo', extensions: ['png', 'svg', 'jpg', 'jpeg'] }], 'Choose a logo');
    if (!f) return;
    const c = await api.brand_check_logo({ path: f });
    if (!c.ok) { toast(c.warnings.join(' '), 'error'); return; }
    logoPath = f; logoPreview = c.preview_data_url; logoWarn = c.warnings;
  }

  async function saveBrand() {
    if (!bname.trim() && !logoPath) { next(); return; }
    savingBrand = true;
    try {
      await api.brand_save({ layer: brandFor, brand: { name: bname.trim() }, logoPath, removeLogo: false });
      next();
    } catch (e) { toast(String(e), 'error'); }
    finally { savingBrand = false; }
  }

  async function importPack() {
    const f = await pickFile([{ name: 'TUNAS branding pack', extensions: ['tunasbrand'] }], 'Choose a branding pack');
    if (!f) return;
    try {
      const r = await api.brand_import_pack({ path: f });
      await refreshAssistants();
      toast(`Branding "${r.brand_name}" applied.`, 'ok');
      next();
    } catch (e) { toast(String(e), 'error'); }
  }

  async function finish() {
    await api.setting_set({ key: 'setup_done', value: '1' });
    app.info.setup_done = true;
    app.view = 'chat';
  }
</script>

<div class="setup">
  <div class="panel card">
    <div class="steps">
      {#each steps as s, i}
        <div class="st" class:on={i === step} class:done={i < step}><span>{i < step ? '✓' : i + 1}</span>{s}</div>
      {/each}
    </div>

    <div class="content">
      {#if step === 0}
        <div class="center">
          <BrandLogo src={app.brand?.logo_data_url} size={84} />
          <h1>{oem?.name ? `Welcome to ${app.brand.effective_name}` : 'Welcome to TUNAS'}</h1>
          <p class="mute lead">Your own AI, running privately on this notebook. It works offline, reads your documents and speaks English and Bahasa Melayu.</p>
          <p class="mute small">This takes a few minutes: we check your notebook, get one or two AI models, and you can add your organisation's name and logo.</p>
          <button class="btn primary big" onclick={next}>Get started <Icon name="send" size={16} /></button>
        </div>
      {:else if step === 1}
        <h2>Your notebook</h2>
        {#if hw}
          <div class="specs">
            <div><Icon name="memory" /><b>{hw.ram_gb} GB</b><span>Memory</span></div>
            <div><Icon name="cpu" /><b class="ellipsis" title={hw.cpu_name}>{hw.cpu_cores} cores</b><span class="ellipsis">{hw.cpu_name}</span></div>
            <div><Icon name="chip" /><b>{hw.vulkan_devices.length ? 'Graphics' : 'Processor'}</b><span>Used for AI</span></div>
            <div><Icon name="disk" /><b>{hw.disk_free_gb} GB</b><span>Free space</span></div>
          </div>
          <div class="verdict"><Icon name="check" size={18} /><div><b>{hw.verdict}</b>{#each hw.notes as n}<div class="small mute">{n}</div>{/each}</div></div>
        {:else}
          <p class="mute" style="margin-top:20px">Checking memory, processor and graphics (up to 15 seconds)…</p>
        {/if}
        <div class="nav"><button class="btn" onclick={back}>Back</button><div class="spacer"></div><button class="btn primary" disabled={!hw} onclick={next}>Next</button></div>
      {:else if step === 2}
        <h2>Choose your models</h2>
        <p class="mute">We've ticked the best ones for this notebook. You can add or remove models later.</p>
        <div class="mlist">
          {#each bundled as m (m.id)}
            <label class="mrow" class:dim={m.installed}>
              <input type="checkbox" value={m.id} bind:group={picked} disabled={m.installed} />
              <div class="grow">
                <div class="row"><b>{m.name}</b>{#if hw?.recommended_models?.[0] === m.id}<span class="pill brand">Best for this notebook</span>{/if}{#if m.installed}<span class="pill ok">Installed</span>{/if}</div>
                <div class="small mute">{m.description}</div>
              </div>
              <span class="small faint">{fmtBytes(m.size)}</span>
            </label>
          {/each}
        </div>
        <div class="src">
          <label class="srcopt" class:sel={source === 'online'}><input type="radio" bind:group={source} value="online" /><Icon name="download" /><div><b>Download</b><div class="small mute">{fmtBytes(totalSize)} from the internet. You can keep going while it downloads.</div></div></label>
          <label class="srcopt" class:sel={source === 'folder'}><input type="radio" bind:group={source} value="folder" /><Icon name="folder" /><div><b>From a folder or USB stick</b><div class="small mute">No internet needed. Checked and copied in.</div></div></label>
        </div>
        {#if importReport}{#each importReport.failed as f}<div class="small" style="color:var(--err)">{f}</div>{/each}{/if}
        <div class="nav">
          <button class="btn" onclick={back}>Back</button><div class="spacer"></div>
          {#if hasModel}<button class="btn ghost" onclick={next}>Skip</button>{/if}
          <button class="btn primary" disabled={importing || (source === 'online' && !picked.length)} onclick={startModels}>
            {importing ? 'Copying…' : source === 'online' ? 'Download and continue' : 'Choose folder…'}
          </button>
        </div>
      {:else if step === 3}
        <h2>Make it yours <span class="faint" style="font-weight:500">(optional)</span></h2>
        <p class="mute">Put your organisation's or your own name and logo on the app. You can change this any time in Branding.</p>
        <div class="brandgrid">
          <div class="col" style="gap:14px">
            <div class="seg">
              <button class:sel={brandFor === 'org'} onclick={() => (brandFor = 'org')}>For my organisation</button>
              <button class:sel={brandFor === 'personal'} onclick={() => (brandFor = 'personal')}>Just for me</button>
            </div>
            <label class="field">Name<input type="text" bind:value={bname} maxlength="48" placeholder={brandFor === 'org' ? 'e.g. AI Fakulti Kejuruteraan' : 'e.g. Pembantu AI Kak Mela'} /></label>
            <div class="field">Logo
              <div class="row"><button class="btn sm" onclick={chooseLogo}><Icon name="upload" size={14} /> Choose logo…</button>{#if logoPath}<span class="small faint ellipsis">{logoPath.split(/[\\/]/).pop()}</span>{/if}</div>
              {#each logoWarn as w}<div class="small" style="color:var(--warn)">{w}</div>{/each}
            </div>
            <button class="btn sm ghost" style="align-self:flex-start" onclick={importPack}><Icon name="download" size={14} /> I have a branding pack</button>
          </div>
          <div class="mini">
            <div class="mt"><BrandLogo src={logoPreview || app.brand?.logo_data_url} size={14} rounded={false} /><span class="ellipsis">{bname || app.brand?.effective_name || 'TUNAS'}</span></div>
            <div class="mb"><BrandLogo src={logoPreview || app.brand?.logo_data_url} size={46} /><b>Hello, I'm {bname || app.brand?.effective_assistant_name || 'your assistant'}.</b><span class="salt-credit" style="font-size:10px">Powered by <b>S.A.L.T CI</b></span></div>
          </div>
        </div>
        <div class="nav"><button class="btn" onclick={back}>Back</button><div class="spacer"></div><button class="btn ghost" onclick={next}>Skip</button><button class="btn primary" disabled={savingBrand} onclick={saveBrand}>Save and continue</button></div>
      {:else}
        <div class="center">
          <BrandLogo src={app.brand?.logo_data_url} size={72} />
          <h1>{hasModel ? "You're ready" : 'Almost ready'}</h1>
          {#if active.length}
            <div class="dl">
              {#each active as p (p.id)}
                <div class="small row"><b class="grow" style="text-align:left">{app.catalog.find((m) => m.id === p.id)?.name}</b><span class="mute">{fmtBytes(p.received)} / {fmtBytes(p.total)} · {fmtEta(p.total - p.received, p.bytes_per_sec)}</span></div>
                <div class="bar"><i style="width:{p.total ? (100 * p.received) / p.total : 0}%"></i></div>
              {/each}
              <p class="small faint">Downloads continue in the background. You can start chatting as soon as one model is ready.</p>
            </div>
          {:else if hasModel}
            <p class="mute lead">Ask anything, drop in a document, or make your own assistants. Everything stays on this notebook.</p>
          {:else}
            <p class="mute lead">No model is installed yet. You can download or import one from the Models page.</p>
          {/if}
          <button class="btn primary big" onclick={finish}>{hasModel ? 'Start chatting' : 'Open TUNAS'}</button>
        </div>
      {/if}
    </div>
    <SaltBrandStrip compact />
  </div>
</div>

<style>
  .setup { height: 100%; display: grid; place-items: center; padding: 24px; background:
    radial-gradient(1200px 600px at 10% 0%, color-mix(in srgb, var(--brand1) 14%, transparent), transparent),
    radial-gradient(900px 500px at 100% 100%, color-mix(in srgb, #E3C254 16%, transparent), transparent), var(--bg); overflow-y: auto; }
  .panel { width: 860px; max-width: 100%; min-height: 560px; display: flex; flex-direction: column; padding: 0; overflow: hidden; box-shadow: var(--shadow); }
  .steps { display: flex; gap: 4px; padding: 16px 22px; border-bottom: 1px solid var(--line); background: var(--soft); }
  .st { display: flex; align-items: center; gap: 7px; font-size: 12.5px; color: var(--faint); font-weight: 600; flex: 1; }
  .st span { width: 22px; height: 22px; border-radius: 50%; display: grid; place-items: center; font-size: 11px; background: var(--line); color: var(--mute); }
  .st.on { color: var(--ink); }
  .st.on span, .st.done span { background: var(--grad); color: #fff; }
  .content { flex: 1; padding: 28px 34px; display: flex; flex-direction: column; gap: 12px; }
  .center { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; text-align: center; gap: 14px; }
  .lead { font-size: 15px; max-width: 560px; }
  .big { height: 44px; padding: 0 22px; font-size: 15px; margin-top: 8px; }
  .nav { display: flex; gap: 8px; margin-top: auto; padding-top: 16px; }
  .specs { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; margin-top: 10px; }
  .specs > div { display: flex; flex-direction: column; gap: 3px; padding: 14px; border-radius: 12px; background: var(--soft); color: var(--mute); min-width: 0; }
  .specs b { color: var(--ink); font-size: 16px; }
  .specs span { font-size: 12px; }
  .verdict { display: flex; gap: 10px; padding: 14px; border-radius: 12px; border: 1px solid color-mix(in srgb, var(--ok) 35%, var(--line)); background: color-mix(in srgb, var(--ok) 6%, transparent); margin-top: 6px; }
  .verdict :global(.icon) { color: var(--ok); margin-top: 1px; }
  .mlist { display: flex; flex-direction: column; gap: 8px; }
  .mrow { display: flex; align-items: center; gap: 12px; padding: 12px 14px; border: 1px solid var(--line); border-radius: 12px; cursor: pointer; }
  .mrow.dim { opacity: .7; }
  .src { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-top: 6px; }
  .srcopt { display: flex; gap: 10px; align-items: flex-start; padding: 12px 14px; border: 1px solid var(--line); border-radius: 12px; cursor: pointer; }
  .srcopt.sel { border-color: var(--brand1); background: color-mix(in srgb, var(--brand1) 5%, transparent); }
  .srcopt input { margin-top: 3px; }
  .brandgrid { display: grid; grid-template-columns: 1fr 300px; gap: 22px; margin-top: 6px; }
  .seg { display: inline-flex; border: 1px solid var(--line); border-radius: 10px; padding: 3px; gap: 2px; background: var(--bg); align-self: flex-start; }
  .seg button { padding: 6px 12px; border-radius: 7px; font-weight: 600; font-size: 13px; color: var(--mute); }
  .seg button.sel { background: var(--panel); color: var(--ink); box-shadow: 0 1px 3px rgba(0,0,0,.08); }
  .mini { border: 1px solid var(--line); border-radius: 12px; overflow: hidden; box-shadow: var(--shadow); align-self: start; }
  .mt { display: flex; align-items: center; gap: 6px; padding: 6px 10px; font-size: 11px; background: var(--soft); border-bottom: 1px solid var(--line); }
  .mb { display: flex; flex-direction: column; align-items: center; gap: 8px; padding: 22px 14px; text-align: center; background: var(--bg); }
  .dl { width: 100%; max-width: 520px; display: flex; flex-direction: column; gap: 6px; }
</style>
