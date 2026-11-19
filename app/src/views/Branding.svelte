<script>
  import Icon from '../lib/Icon.svelte';
  import BrandLogo from '../lib/BrandLogo.svelte';
  import { api, pickFile, saveFile } from '../lib/api.js';
  import { app, toast, refreshAssistants, assistantName } from '../lib/state.svelte.js';

  const LAYER_LABEL = { personal: 'My own brand', org: 'Organisation or department', oem: 'OEM or reseller' };
  const PRESETS = [
    ['#2F74D8', '#6D4FE0'], ['#0F8F9C', '#2F74D8'], ['#6D4FE0', '#D9443A'], ['#D99A00', '#F05A28'],
    ['#0B1022', '#2F74D8'], ['#8A1538', '#D99A00'], ['#14532D', '#0F8F9C'], ['#545B72', '#141A2E'],
  ];

  let layer = $state(initialLayer());
  let form = $state(load(initialLayer()));
  let logoPath = $state(null); // newly chosen file
  let logoCheck = $state(null);
  let removeLogo = $state(false);
  let saving = $state(false);
  let exp = $state({ open: false, layer: 'org', assistants: [], policy: false, issuer: '',
    pol: { allow_change_org_brand: false, allow_personal_brand: true, allow_create_assistants: true, allow_cloud: true, allow_knowledge_with_cloud: true } });

  function initialLayer() {
    const ls = app.brand?.layers || [];
    const org = ls.find((l) => l.layer === 'org');
    if (org?.brand && org.editable) return 'org';
    return ls.find((l) => l.layer === 'personal')?.editable ? 'personal' : 'org';
  }

  function lv(l) { return app.brand?.layers?.find((x) => x.layer === l); }

  function load(l) {
    const b = lv(l)?.brand || {};
    return {
      name: b.name || '', assistant_name: b.assistant_name || '', welcome: b.welcome || '',
      color1: b.color1 || '', color2: b.color2 || '', letterhead_org: b.letterhead_org || '',
      letterhead_address: b.letterhead_address || '', issuer: b.issuer || '', logo_file: b.logo_file || '', updated: '',
    };
  }

  function selectLayer(l) {
    layer = l;
    form = load(l);
    logoPath = null; logoCheck = null; removeLogo = false;
  }

  const current = $derived(lv(layer));
  const editable = $derived(!!current?.editable);
  const previewLogo = $derived(removeLogo ? '' : logoCheck?.ok ? logoCheck.preview_data_url : current?.logo_data_url || '');
  const previewName = $derived(form.name.trim() || effectiveBelow('name') || 'TUNAS');
  const c1 = $derived(form.color1 || effectiveBelow('color1') || '#2F74D8');
  const c2 = $derived(form.color2 || effectiveBelow('color2') || '#6D4FE0');
  const previewAssistant = $derived(form.assistant_name.trim() || effectiveBelow('assistant_name') || assistantName(app.assistants.find((a) => a.id === 'starter-general')) || 'General Assistant');
  const previewWelcome = $derived(form.welcome.trim() || effectiveBelow('welcome') || `Hello, I'm ${previewAssistant}.`);

  // Value from less specific layers, for the preview.
  function effectiveBelow(key) {
    const order = ['personal', 'org', 'oem'];
    for (const l of order.slice(order.indexOf(layer) + 1)) {
      const v = lv(l)?.brand?.[key];
      if (v) return v;
    }
    return '';
  }

  async function chooseLogo() {
    const f = await pickFile([{ name: 'Logo', extensions: ['png', 'svg', 'jpg', 'jpeg'] }], 'Choose a logo (PNG or SVG)');
    if (!f) return;
    const c = await api.brand_check_logo({ path: f });
    if (!c.ok) { toast(c.warnings.join(' '), 'error'); return; }
    logoPath = f;
    logoCheck = c;
    removeLogo = false;
  }

  async function save() {
    if (!form.name.trim() && !logoPath && !current?.logo_data_url) { toast('Add at least a name or a logo.', 'error'); return; }
    saving = true;
    try {
      await api.brand_save({ layer, brand: $state.snapshot(form), logoPath, removeLogo });
      logoPath = null; logoCheck = null; removeLogo = false;
      form = load(layer);
      toast('Branding saved. The title, icon and shortcuts now show it.', 'ok', 5000);
    } catch (e) { toast(String(e), 'error'); }
    finally { saving = false; }
  }

  async function clear() {
    try {
      await api.brand_clear({ layer });
      form = load(layer);
      logoPath = null; logoCheck = null;
      toast('Brand removed from this layer.');
    } catch (e) { toast(String(e), 'error'); }
  }

  async function importPack() {
    const f = await pickFile([{ name: 'TUNAS branding pack', extensions: ['tunasbrand'] }], 'Choose a branding pack');
    if (!f) return;
    try {
      const r = await api.brand_import_pack({ path: f });
      await refreshAssistants();
      selectLayer('org');
      toast(`Branding "${r.brand_name || 'pack'}" applied${r.assistants ? ` with ${r.assistants} assistant(s)` : ''}.`, 'ok');
    } catch (e) { toast(String(e), 'error', 8000); }
  }

  async function exportPack() {
    const b = lv(exp.layer)?.brand;
    const base = (b?.name || 'branding').replace(/[\\/:*?"<>|]/g, '').trim() || 'branding';
    const path = await saveFile(`${base}.tunasbrand`, [{ name: 'TUNAS branding pack', extensions: ['tunasbrand'] }], 'Save branding pack');
    if (!path) return;
    try {
      await api.brand_export_pack({
        path, layer: exp.layer, assistantIds: exp.assistants, issuer: exp.issuer,
        policy: exp.policy ? $state.snapshot(exp.pol) : null,
      });
      exp.open = false;
      toast('Branding pack saved. Hand it out on a USB stick or by link; people double-click it to apply.', 'ok', 7000);
    } catch (e) { toast(String(e), 'error'); }
  }

</script>

<div class="page">
  <div class="page-head">
    <div class="grow">
      <h1>Branding</h1>
      <p>Put your organisation's or your own name and logo on TUNAS. It shows in the title bar, the icon, the welcome screen and the assistant's name.</p>
    </div>
    <button class="btn" onclick={importPack}><Icon name="download" size={15} /> Import branding pack</button>
  </div>

  <div class="layers">
    {#each ['personal', 'org', 'oem'] as l}
      {@const v = lv(l)}
      <button class="layer" class:sel={layer === l} onclick={() => selectLayer(l)}>
        <BrandLogo src={v?.logo_data_url} size={28} />
        <div class="grow" style="text-align:left">
          <div class="small faint">{LAYER_LABEL[l]}</div>
          <div class="ellipsis"><b>{v?.brand?.name || (v?.brand ? '(logo only)' : 'Not set')}</b></div>
        </div>
        {#if !v?.editable}<Icon name="lock" size={14} class="faint" />{/if}
      </button>
    {/each}
    <div class="layer salt">
      <div class="grow" style="text-align:left">
        <div class="small faint">Always shown</div>
        <div class="salt-credit" style="font-size:13px">Powered by <b>S.A.L.T CI</b></div>
      </div>
    </div>
  </div>
  <p class="small faint" style="margin:8px 2px 0">The most specific brand is shown first: your own, then your organisation's, then the OEM's. All of them are credited on the About page.</p>

  <div class="split">
    <div class="card editor">
      {#if !editable}
        <div class="locked">
          <Icon name="lock" size={18} />
          <div>
            {#if layer === 'oem'}This brand comes with the notebook from the maker or reseller and is set at installation.
            {:else if current?.machine}This brand is managed by your organisation's IT department.
            {:else}Your organisation does not allow this brand to be changed.{/if}
          </div>
        </div>
      {/if}
      <fieldset disabled={!editable}>
        <div class="step"><span class="n">1</span>
          <label class="field grow">Name
            <input type="text" bind:value={form.name} maxlength="48" placeholder={layer === 'personal' ? 'e.g. Pembantu AI Kak Mela' : 'e.g. AI Fakulti Kejuruteraan'} />
          </label>
        </div>

        <div class="step"><span class="n">2</span>
          <div class="field grow">Logo
            <span class="hint">PNG or SVG. A square logo works best for the icon.</span>
            <div class="row" style="margin-top:4px">
              <div class="logo-box"><BrandLogo src={previewLogo} size={52} rounded={false} /></div>
              <button class="btn sm" onclick={chooseLogo}><Icon name="upload" size={14} /> Choose logo…</button>
              {#if previewLogo}<button class="btn sm ghost" onclick={() => { removeLogo = true; logoPath = null; logoCheck = null; }}>Remove</button>{/if}
            </div>
            {#each logoCheck?.warnings || [] as w}<div class="small" style="color:var(--warn)"><Icon name="warn" size={13} /> {w}</div>{/each}
          </div>
        </div>

        <div class="step"><span class="n">3</span>
          <div class="field grow">Colour
            <div class="presets">
              {#each PRESETS as [a, b]}
                <button class="preset" class:sel={form.color1 === a && form.color2 === b} style="background:linear-gradient(120deg,{a},{b})"
                  aria-label="{a} to {b}" onclick={() => { form.color1 = a; form.color2 = b; }}></button>
              {/each}
              <label class="custom" title="Choose your own two colours">
                <input type="color" bind:value={form.color1} /><input type="color" bind:value={form.color2} />
              </label>
            </div>
          </div>
        </div>

        <div class="step"><span class="n">4</span>
          <label class="field grow">Welcome line <span class="hint">Optional. Shown on the welcome screen.</span>
            <input type="text" bind:value={form.welcome} maxlength="90" placeholder="e.g. Selamat datang ke AI Fakulti Kejuruteraan" />
          </label>
        </div>

        <details>
          <summary class="small">More options</summary>
          <label class="field" style="margin-top:10px">Assistant's name <span class="hint">Optional. The name of the main assistant, if different from the brand name.</span>
            <input type="text" bind:value={form.assistant_name} maxlength="48" placeholder="e.g. Pembantu AI FKE" />
          </label>
        </details>
      </fieldset>

      <div class="row" style="margin-top:18px">
        {#if editable && current?.brand}<button class="btn ghost danger sm" onclick={clear}><Icon name="trash" size={14} /> Remove this brand</button>{/if}
        <div class="spacer"></div>
        <button class="btn primary" disabled={!editable || saving} onclick={save}>{saving ? 'Saving…' : 'Save branding'}</button>
      </div>
    </div>

    <div class="preview-col">
      <div class="small faint" style="margin-bottom:8px">Live preview</div>
      <div class="win">
        <div class="titlebar">
          <span class="tb-ic"><BrandLogo src={previewLogo} size={16} rounded={false} /></span>
          <span class="ellipsis">{previewName}</span>
          <span class="spacer"></span><span class="dots"><i></i><i></i><i></i></span>
        </div>
        <div class="win-body">
          <div class="mini-side">
            <div class="row"><BrandLogo src={previewLogo} size={22} /><b class="ellipsis small">{previewName}</b></div>
            <div class="mini-new" style="background:linear-gradient(120deg,{c1},{c2})">New chat</div>
            <div class="mini-line"></div><div class="mini-line short"></div><div class="mini-line"></div>
            <div class="spacer"></div>
            <div class="salt-credit" style="font-size:9px">Powered by <b>S.A.L.T CI</b></div>
          </div>
          <div class="mini-main">
            <BrandLogo src={previewLogo} size={40} />
            <div class="mini-welcome">{previewWelcome}</div>
            <div class="mini-input"><span class="faint">Message {previewAssistant}…</span><span class="mini-send" style="background:linear-gradient(120deg,{c1},{c2})"></span></div>
          </div>
        </div>
      </div>

      <div class="small faint" style="margin:16px 0 8px">Desktop and Start menu</div>
      <div class="desk">
        <div class="dicon"><div class="dic"><BrandLogo src={previewLogo} size={44} rounded={false} /></div><span>{previewName}</span></div>
        <div class="taskbar"><BrandLogo src={previewLogo} size={22} rounded={false} /></div>
      </div>
    </div>
  </div>

  <div class="section-title">Branding packs</div>
  <div class="card tight row pack">
    <Icon name="brush" size={20} />
    <div class="grow">
      <b>Share your brand in one file</b>
      <div class="small mute">Make a .tunasbrand file with the name, logo and colours, and optionally assistants and rules. At a workshop, everyone double-clicks it and the whole faculty is branded in one step.</div>
    </div>
    <button class="btn" onclick={() => { exp.open = !exp.open; exp.layer = lv('org')?.brand ? 'org' : 'personal'; }}>Make a pack…</button>
  </div>

  {#if exp.open}
    <div class="card" style="margin-top:10px">
      <div class="grid2">
        <label class="field">Brand to include
          <select bind:value={exp.layer}>
            {#each ['org', 'personal', 'oem'] as l}{#if lv(l)?.brand}<option value={l}>{LAYER_LABEL[l]}: {lv(l).brand.name || 'logo only'}</option>{/if}{/each}
          </select>
        </label>
        <label class="field">Made by <span class="hint">Shown to people who import it.</span>
          <input type="text" bind:value={exp.issuer} placeholder="e.g. Unit ICT, Fakulti Kejuruteraan" />
        </label>
      </div>
      <div class="field" style="margin-top:14px">Assistants to include
        <div class="checks">
          {#each app.assistants as a}
            <label class="chk"><input type="checkbox" value={a.id} bind:group={exp.assistants} /> {assistantName(a)}</label>
          {/each}
        </div>
      </div>
      <label class="chk" style="margin-top:14px"><input type="checkbox" bind:checked={exp.policy} /> <b>Include organisation rules</b></label>
      {#if exp.policy}
        <div class="checks rules">
          <label class="chk"><input type="checkbox" bind:checked={exp.pol.allow_change_org_brand} /> People may change the organisation brand</label>
          <label class="chk"><input type="checkbox" bind:checked={exp.pol.allow_personal_brand} /> People may add their own brand on top</label>
          <label class="chk"><input type="checkbox" bind:checked={exp.pol.allow_create_assistants} /> People may create their own assistants</label>
        </div>
        <p class="small faint">Rules in a pack apply to people who import it. For notebooks your IT department manages, rules can also be set machine-wide so users cannot change them.</p>
      {/if}
      <div class="row" style="justify-content:flex-end;margin-top:14px">
        <button class="btn" onclick={() => (exp.open = false)}>Cancel</button>
        <button class="btn primary" disabled={!lv(exp.layer)?.brand} onclick={exportPack}>Save pack…</button>
      </div>
    </div>
  {/if}
</div>

<style>
  .layers { display: grid; grid-template-columns: repeat(4, 1fr); gap: 10px; }
  .layer { display: flex; align-items: center; gap: 10px; padding: 10px 12px; border-radius: 12px; border: 1px solid var(--line); background: var(--panel); min-width: 0; }
  .layer.sel { border-color: var(--brand1); box-shadow: 0 0 0 3px color-mix(in srgb, var(--brand1) 15%, transparent); }
  .layer.salt { background: var(--soft); }
  .split { display: grid; grid-template-columns: minmax(380px, 1fr) 420px; gap: 18px; margin-top: 18px; align-items: start; }
  fieldset { border: 0; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 18px; min-width: 0; }
  fieldset:disabled { opacity: .55; }
  .locked { display: flex; gap: 10px; align-items: center; padding: 12px; margin-bottom: 16px; border-radius: 10px; background: var(--soft); color: var(--mute); }
  .step { display: flex; gap: 12px; align-items: flex-start; }
  .n { width: 24px; height: 24px; border-radius: 50%; display: grid; place-items: center; font-size: 12px; font-weight: 700; color: #fff; background: var(--grad); flex: none; margin-top: 1px; }
  .logo-box { width: 72px; height: 72px; border-radius: 12px; border: 1px dashed var(--line); display: grid; place-items: center; background: repeating-conic-gradient(var(--soft) 0 25%, transparent 0 50%) 0 0 / 14px 14px; }
  .presets { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 4px; }
  .preset { width: 40px; height: 28px; border-radius: 8px; border: 2px solid transparent; outline: 2px solid transparent; }
  .preset.sel { outline-color: var(--ink); border-color: var(--panel); }
  .custom { display: inline-flex; gap: 2px; align-items: center; padding: 0 4px; border: 1px solid var(--line); border-radius: 8px; height: 28px; cursor: pointer; }
  .custom input { width: 22px; height: 20px; border: 0; padding: 0; background: none; cursor: pointer; }
  details summary { cursor: pointer; color: var(--mute); }
  .preview-col { position: sticky; top: 0; }
  .win { border-radius: 12px; overflow: hidden; border: 1px solid var(--line); box-shadow: var(--shadow); background: var(--panel); }
  .titlebar { display: flex; align-items: center; gap: 8px; height: 30px; padding: 0 10px; font-size: 12px; background: var(--soft); border-bottom: 1px solid var(--line); }
  .tb-ic { display: inline-grid; place-items: center; }
  .dots { display: flex; gap: 10px; }
  .dots i { width: 9px; height: 1.5px; background: var(--faint); }
  .win-body { display: flex; height: 230px; }
  .mini-side { width: 132px; border-right: 1px solid var(--line); padding: 10px 8px; display: flex; flex-direction: column; gap: 8px; }
  .mini-new { color: #fff; font-size: 10px; font-weight: 600; text-align: center; padding: 5px; border-radius: 6px; }
  .mini-line { height: 7px; border-radius: 4px; background: var(--soft); }
  .mini-line.short { width: 60%; }
  .mini-main { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 10px; padding: 12px; text-align: center; background: var(--bg); }
  .mini-welcome { font-weight: 700; font-size: 14px; line-height: 1.3; }
  .mini-input { display: flex; align-items: center; justify-content: space-between; width: 100%; font-size: 10px; padding: 6px 6px 6px 10px; border: 1px solid var(--line); border-radius: 9px; background: var(--panel); }
  .mini-send { width: 18px; height: 18px; border-radius: 6px; }
  .desk { display: flex; align-items: flex-end; justify-content: space-between; height: 130px; padding: 14px; border-radius: 12px; background: linear-gradient(135deg, #1B2A5C, #3B2C7A 60%, #7A5A2A); }
  .dicon { display: flex; flex-direction: column; align-items: center; gap: 5px; width: 92px; color: #fff; font-size: 11px; text-align: center; text-shadow: 0 1px 2px rgba(0,0,0,.6); align-self: flex-start; }
  .dic { width: 54px; height: 54px; display: grid; place-items: center; }
  .dicon span { overflow: hidden; display: -webkit-box; -webkit-line-clamp: 2; line-clamp: 2; -webkit-box-orient: vertical; }
  .taskbar { display: grid; place-items: center; width: 40px; height: 36px; border-radius: 7px; background: rgba(255,255,255,.14); border-bottom: 2px solid #8fb4ff; }
  .pack { gap: 14px; }
  .grid2 { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; }
  .checks { display: grid; grid-template-columns: repeat(3, 1fr); gap: 6px 14px; font-weight: 400; margin-top: 4px; }
  .checks.rules { grid-template-columns: 1fr; margin: 8px 0 6px 24px; }
  .chk { display: flex; align-items: center; gap: 8px; font-size: 13px; cursor: pointer; }
  @media (max-width: 1150px) { .split { grid-template-columns: 1fr; } .preview-col { position: static; } .layers { grid-template-columns: 1fr 1fr; } }
</style>
