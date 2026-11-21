<script>
  import { onMount } from 'svelte';
  import { api, listen } from './lib/api.js';
  import {
    app, toast, refreshBrand, refreshAssistants, refreshModels, refreshChats, refreshProviders, loadPrefs, initEvents,
  } from './lib/state.svelte.js';
  import Sidebar from './views/Sidebar.svelte';
  import Chat from './views/Chat.svelte';
  import Assistants from './views/Assistants.svelte';
  import Knowledge from './views/Knowledge.svelte';
  import { kn, initKnowledgeEvents } from './lib/knowledge.svelte.js';
  import Setup from './views/Setup.svelte';
  import SettingsShell from './views/SettingsShell.svelte';
  import Icon from './lib/Icon.svelte';
  import UpdateCard from './views/UpdateCard.svelte';
  import { startUpdateChecks } from './lib/update.svelte.js';

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
    initKnowledgeEvents();
    app.info = await api.app_info();
    for (let i = 0; i < 50 && !app.gateway.port; i++) { app.gateway = await api.gateway_info(); if (!app.gateway.port) await new Promise((r) => setTimeout(r, 100)); }
    await Promise.all([refreshBrand(), loadPrefs(), refreshAssistants(), refreshModels(), refreshChats(), refreshProviders()]);
    app.engine = await api.engine_status();
    if (!app.info.setup_done) app.view = 'setup';
    ready = true;
    startUpdateChecks();
    const pending = await api.take_pending_pack();
    if (pending) offerPack(pending);
    await listen('open-pack', (e) => offerPack(e.payload));
  });
</script>

{#if ready}
  {#if app.view === 'setup'}
    <Setup />
  {:else}
    <div class="shell" class:navopen={app.navOpen}>
      <Sidebar />
      {#if app.navOpen}<button class="navscrim" aria-label="Close the menu" onclick={() => (app.navOpen = false)}></button>{/if}
      <main>
        <UpdateCard />
        {#if app.view === 'assistants' || app.view === 'knowledge'}
          <div class="mbar"><button class="icon-btn" aria-label="Menu" onclick={() => (app.navOpen = true)}><Icon name="menu" size={20} /></button><b class="ellipsis">{app.brand?.effective_name || 'TUNAS'}</b></div>
        {/if}
        {#if app.view === 'chat'}<Chat />
        {:else if app.view === 'assistants'}<Assistants />
        {:else if app.view === 'knowledge'}<Knowledge />
        {:else if app.view === 'settings'}<SettingsShell />
        {/if}
      </main>
    </div>
  {/if}
{/if}

{#if kn.consent}
  <div class="overlay">
    <div class="dialog card" style="max-width:520px">
      <div class="row"><Icon name="lock" size={20} /><h2>Allow TUNAS to read {kn.consent.paths.length === 1 ? 'this place' : 'these places'}?</h2></div>
      <div class="small" style="margin:12px 0;display:flex;flex-direction:column;gap:4px">
        {#each kn.consent.paths as p}<div class="ellipsis" title={p}><Icon name="folder" size={14} /> <b>{p}</b></div>{/each}
      </div>
      <ul class="small mute" style="margin:0 0 6px 18px;display:flex;flex-direction:column;gap:4px">
        <li>TUNAS reads the documents here, on this machine, so your assistants can answer from them.</li>
        <li>Nothing is copied out of this place and nothing is sent anywhere. The search index stays on this machine.</li>
        <li>Answers from a <b>cloud</b> model would include the parts of your documents it uses. Local models keep everything here.</li>
        <li>You can remove this permission any time under Knowledge → Folder permissions. Your files are never changed or deleted.</li>
      </ul>
      <div class="row" style="justify-content:flex-end;margin-top:16px">
        <button class="btn" onclick={() => kn.consent.resolve(false)}>Not now</button>
        <button class="btn primary" onclick={() => kn.consent.resolve(true)}>Allow</button>
      </div>
    </div>
  </div>
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
  main { flex: 1; min-width: 0; height: 100%; background: var(--bg); display: flex; flex-direction: column; }
  main > :global(*:not(.mbar):not(.upd)) { flex: 1; min-height: 0; }
  .mbar { display: none; }
  .navscrim { display: none; }
  /* Phones and narrow windows: the side bar becomes a drawer. */
  @media (max-width: 760px) {
    .shell :global(aside) { position: fixed; left: 0; top: 0; bottom: 0; z-index: 40; transform: translateX(-105%); transition: transform .2s ease; box-shadow: var(--shadow); max-width: 86vw; }
    .shell.navopen :global(aside) { transform: none; }
    .navscrim { display: block; position: fixed; inset: 0; z-index: 39; background: rgba(11, 16, 34, .4); }
    .mbar { display: flex; flex: none; align-items: center; gap: 8px; padding: 8px 10px; border-bottom: 1px solid var(--line); background: var(--panel); }
    .toasts { left: 12px; right: 12px; bottom: 12px; }
    .toast { max-width: none; }
  }
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
