<script>
  import { onMount } from 'svelte';
  import Icon from '../lib/Icon.svelte';
  import Models from './Models.svelte';
  import Branding from './Branding.svelte';
  import Settings from './Settings.svelte';
  import About from './About.svelte';
  import Connections from './Connections.svelte';
  import Folders from './Folders.svelte';
  import { app, closeSettings } from '../lib/state.svelte.js';

  const tabs = [
    ['general', 'settings', 'General', 'Default model, engine, appearance'],
    ['models', 'models', 'Models', 'Check my machine, download, USB'],
    ['folders', 'folder', 'Folders', 'Places TUNAS may read for Knowledge'],
    ['branding', 'brush', 'Branding', 'Name, logo, branding packs'],
    ['connections', 'globe', 'Connections', 'TERAS, cloud, apps that use TUNAS'],
    ['about', 'info', 'About', 'TUNAS, S.A.L.T CI, licences'],
  ];

  onMount(() => {
    const onKey = (e) => { if (e.key === 'Escape' && !document.querySelector('.overlay')) closeSettings(); };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  });
</script>

<div class="shell">
  <nav>
    <div class="head">
      <button class="back" onclick={closeSettings} title="Back to chat (Esc)"><Icon name="chevron" size={16} class="flip" /> Back</button>
      <h2>Settings</h2>
    </div>
    {#each tabs as [id, icon, label, sub]}
      <button class="tab" class:sel={app.settingsTab === id} onclick={() => (app.settingsTab = id)}>
        <Icon name={icon} size={18} />
        <span class="grow"><b>{label}</b><span class="sub">{sub}</span></span>
      </button>
    {/each}
  </nav>
  <section>
    {#if app.settingsTab === 'general'}<Settings />
    {:else if app.settingsTab === 'folders'}<Folders />
    {:else if app.settingsTab === 'models'}<Models />
    {:else if app.settingsTab === 'branding'}<Branding />
    {:else if app.settingsTab === 'connections'}<Connections />
    {:else}<About />{/if}
  </section>
</div>

<style>
  .shell { display: flex; height: 100%; }
  nav { width: 250px; flex: none; border-right: 1px solid var(--line); background: var(--panel); padding: 14px 10px; display: flex; flex-direction: column; gap: 2px; }
  .head { padding: 2px 6px 12px; display: flex; flex-direction: column; gap: 8px; }
  .back { display: inline-flex; align-items: center; gap: 4px; color: var(--mute); font-weight: 600; font-size: 13px; align-self: flex-start; padding: 4px 8px 4px 2px; border-radius: 8px; }
  .back:hover { background: var(--hover); color: var(--ink); }
  .back :global(.flip) { transform: rotate(180deg); }
  .tab { display: flex; align-items: flex-start; gap: 10px; padding: 9px 10px; border-radius: 10px; text-align: left; color: var(--mute); }
  .tab :global(.icon) { margin-top: 2px; }
  .tab:hover { background: var(--hover); color: var(--ink); }
  .tab.sel { background: color-mix(in srgb, var(--brand1) 10%, transparent); color: var(--ink); }
  .tab b { display: block; font-weight: 600; }
  .sub { display: block; font-size: 11.5px; color: var(--faint); }
  section { flex: 1; min-width: 0; height: 100%; }
</style>
