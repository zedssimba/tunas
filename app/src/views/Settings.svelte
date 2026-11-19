<script>
  import Icon from '../lib/Icon.svelte';
  import { api } from '../lib/api.js';
  import { openPath } from '@tauri-apps/plugin-opener';
  import { app, toast, setPref, setTheme, modelName } from '../lib/state.svelte.js';

  const chatModels = $derived(app.installed.filter((m) => !m.id.includes('embed')));

  async function setEngine(v) {
    await setPref('engine_pref', v);
    await api.engine_stop();
    toast('Saved. The engine will restart with the next message.', 'ok');
  }

  async function setDefault(v) {
    await setPref('default_model', v);
    toast('Default model saved.', 'ok');
  }
</script>

<div class="page">
  <div class="page-head"><div class="grow"><h1>Settings</h1><p>How TUNAS runs on this notebook.</p></div></div>

  <div class="stack">
    <div class="card set">
      <div class="grow"><b>Default model</b><div class="small mute">Used by assistants that don't choose their own model.</div></div>
      <select value={app.prefs.default_model} onchange={(e) => setDefault(e.target.value)} style="width:260px">
        <option value="">Automatic (smallest installed)</option>
        {#each chatModels as m}<option value={m.id}>{m.name}</option>{/each}
      </select>
    </div>

    <div class="card set">
      <div class="grow">
        <b>Run AI on</b>
        <div class="small mute">Automatic uses the graphics when they can help and falls back to the processor. On some notebooks with built-in graphics, the processor is faster: try both with <i>Test speed</i> on the Models page.</div>
      </div>
      <div class="seg">
        {#each [['auto', 'Automatic'], ['vulkan', 'Graphics'], ['cpu', 'Processor']] as [v, l]}
          <button class:sel={app.prefs.engine_pref === v} disabled={v === 'vulkan' && !app.info?.engines?.includes('vulkan')} onclick={() => setEngine(v)}>{l}</button>
        {/each}
      </div>
    </div>

    <div class="card set">
      <div class="grow"><b>Appearance</b><div class="small mute">Light or dark.</div></div>
      <div class="seg">
        <button class:sel={app.theme === 'light'} onclick={() => setTheme('light')}><Icon name="sun" size={14} /> Light</button>
        <button class:sel={app.theme === 'dark'} onclick={() => setTheme('dark')}><Icon name="moon" size={14} /> Dark</button>
      </div>
    </div>

    <div class="card set">
      <div class="grow">
        <b>Engine</b>
        <div class="small mute">
          {#if app.engine?.state === 'ready'}Running {modelName(app.engine.model_id)} on the {app.engine.engine === 'vulkan' ? 'graphics' : 'processor'}, context {app.engine.ctx?.toLocaleString()} tokens.
          {:else if app.engine?.state === 'error'}Problem: {app.engine.message}
          {:else}Not running. It starts with your next message.{/if}
        </div>
      </div>
      <button class="btn sm" onclick={() => api.engine_stop()}>Stop engine</button>
    </div>

    <div class="card set">
      <div class="grow">
        <b>Privacy</b>
        <div class="small mute">Chats, assistants and models are stored only on this notebook. TUNAS sends nothing about you or your chats anywhere. It goes online only when you download a model.</div>
      </div>
      <span class="pill ok"><Icon name="lock" size={12} /> On this device</span>
    </div>

    <div class="card set">
      <div class="grow">
        <b>Data folder</b>
        <div class="small mute ellipsis" title={app.info?.data_dir}>{app.info?.data_dir}</div>
      </div>
      <button class="btn sm" onclick={() => openPath(app.info?.data_dir).catch((e) => toast(String(e), 'error'))}><Icon name="folder" size={14} /> Open</button>
    </div>
  </div>
</div>

<style>
  .stack { display: flex; flex-direction: column; gap: 10px; max-width: 900px; }
  .set { display: flex; align-items: center; gap: 18px; padding: 16px 18px; }
  .seg { display: inline-flex; border: 1px solid var(--line); border-radius: 10px; padding: 3px; gap: 2px; background: var(--bg); flex: none; }
  .seg button { display: inline-flex; align-items: center; gap: 6px; padding: 6px 12px; border-radius: 7px; font-weight: 600; font-size: 13px; color: var(--mute); }
  .seg button.sel { background: var(--panel); color: var(--ink); box-shadow: 0 1px 3px rgba(0,0,0,.08); }
</style>
