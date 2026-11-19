<script>
  import { onMount } from 'svelte';
  import { api, listen } from './lib/api.js';
  import {
    app, toast, refreshBrand, refreshAssistants, refreshModels, refreshChats, loadPrefs, initEvents,
  } from './lib/state.svelte.js';
  import Sidebar from './views/Sidebar.svelte';
  import Chat from './views/Chat.svelte';
  import Assistants from './views/Assistants.svelte';
  import Models from './views/Models.svelte';
  import Branding from './views/Branding.svelte';
  import Settings from './views/Settings.svelte';
  import About from './views/About.svelte';
  import Setup from './views/Setup.svelte';
  import Icon from './lib/Icon.svelte';

  let ready = $state(false);
  let packPrompt = $state(null);

  async function offerPack(path) {
    packPrompt = path;
  }

  async function importPack() {
    const path = packPrompt;
    packPrompt = null;
    try {
      const r = await api.brand_import_pack({ path });
      await refreshAssistants();
      toast(`Branding "${r.brand_name || 'pack'}" applied${r.assistants ? ` with ${r.assistants} assistant${r.assistants > 1 ? 's' : ''}` : ''}.`, 'ok');
    } catch (e) {
      toast(String(e), 'error', 8000);
    }
  }

  onMount(async () => {
    await initEvents();
    app.info = await api.app_info();
    await Promise.all([refreshBrand(), loadPrefs(), refreshAssistants(), refreshModels(), refreshChats()]);
    app.engine = await api.engine_status();
    if (!app.info.setup_done) app.view = 'setup';
    ready = true;
    const pending = await api.take_pending_pack();
    if (pending) offerPack(pending);
    await listen('open-pack', (e) => offerPack(e.payload));
  });
</script>

{#if ready}
  {#if app.view === 'setup'}
    <Setup />
  {:else}
    <div class="shell">
      <Sidebar />
      <main>
        {#if app.view === 'chat'}<Chat />
        {:else if app.view === 'assistants'}<Assistants />
        {:else if app.view === 'models'}<Models />
        {:else if app.view === 'branding'}<Branding />
        {:else if app.view === 'settings'}<Settings />
        {:else if app.view === 'about'}<About />
        {/if}
      </main>
    </div>
  {/if}
{/if}

{#if packPrompt}
  <div class="overlay">
    <div class="dialog card">
      <div class="row"><Icon name="brush" size={20} /><h2>Apply a branding pack?</h2></div>
      <p class="mute" style="margin:10px 0 4px">This will set your organisation's name and logo in TUNAS, and may add assistants.</p>
      <p class="small faint ellipsis" title={packPrompt}>{packPrompt}</p>
      <div class="row" style="justify-content:flex-end;margin-top:16px">
        <button class="btn" onclick={() => (packPrompt = null)}>Cancel</button>
        <button class="btn primary" onclick={importPack}>Apply</button>
      </div>
    </div>
  </div>
{/if}

<div class="toasts">
  {#each app.toasts as t (t.id)}
    <div class="toast {t.kind}">
      <Icon name={t.kind === 'error' ? 'warn' : t.kind === 'ok' ? 'check' : 'info'} size={16} />
      <span>{t.text}</span>
    </div>
  {/each}
</div>

<style>
  .shell { display: flex; height: 100%; }
  main { flex: 1; min-width: 0; height: 100%; background: var(--bg); }
  .overlay { position: fixed; inset: 0; background: rgba(11, 16, 34, .45); display: grid; place-items: center; z-index: 50; }
  .dialog { width: 460px; max-width: calc(100vw - 40px); box-shadow: var(--shadow); }
  .toasts { position: fixed; right: 18px; bottom: 18px; display: flex; flex-direction: column; gap: 8px; z-index: 60; }
  .toast {
    display: flex; gap: 8px; align-items: flex-start; max-width: 420px; padding: 10px 14px; border-radius: 10px;
    background: var(--panel); border: 1px solid var(--line); box-shadow: var(--shadow); font-size: 13px;
  }
  .toast.ok :global(.icon) { color: var(--ok); }
  .toast.error :global(.icon) { color: var(--err); }
  .toast :global(.icon) { margin-top: 2px; }
</style>
