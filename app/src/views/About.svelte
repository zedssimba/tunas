<script>
  import PoweredBy from '../lib/PoweredBy.svelte';
  import Icon from '../lib/Icon.svelte';
  import Mark from '../lib/Mark.svelte';
  import BrandLogo from '../lib/BrandLogo.svelte';
  import SaltBrandStrip from '../lib/SaltBrandStrip.svelte';
  import { openUrl, SALT_LINKS } from '../lib/api.js';
  import { app } from '../lib/state.svelte.js';
  import { upd, checkUpdate, updateNow, whatsNew } from '../lib/update.svelte.js';

  const LABEL = { personal: 'Personal brand', org: 'Organisation', oem: 'Provided by' };
  const credited = $derived((app.brand?.layers || []).filter((l) => l.brand));

  const teras = [
    ['server', 'One AI for the whole organisation', 'Shared models on your own servers, local or cloud, chosen per task and secrecy level.'],
    ['book', 'Knowledge for teams', 'Collections of the organisation’s documents, with permissions, for everyone who needs them.'],
    ['user', 'Sign-in and administration', 'Single sign-on, roles, an audit trail and central settings for IT.'],
    ['chat', 'Meetings with AI', 'People and several AI models in one discussion, with minutes made for you.'],
    ['models', 'Model Studio', 'Customise and fine-tune models for your organisation’s work.'],
    ['globe', 'The S.A.L.T CI apps', 'Email, presentations, meetings, navigation and more, all running on the same core.'],
  ];
</script>

<div class="page">
  <div class="hero card">
    <div class="row" style="gap:16px">
      <Mark size={56} />
      <div class="grow">
        <h1>TUNAS</h1>
        <div class="mute">Personal AI for your machine · Version {app.info?.version}</div>
      </div>
    </div>
    <p style="margin-top:14px">TUNAS runs open AI models on this machine, fully offline and private. <i>Tunas</i> means a young shoot, the first growth: it is the personal start of S.A.L.T CI.</p>
    <div style="margin-top:14px"><PoweredBy size={14} logo={30} /></div>
  </div>

  <div class="section-title">Updates</div>
  <div class="card tight row updrow">
    <Icon name={upd.info?.available ? 'download' : 'check'} size={18} />
    <div class="grow">
      {#if upd.checking}<b>Checking for a new TUNAS…</b>
      {:else if upd.info?.error === 'off'}<b>TUNAS {app.info?.version}</b><div class="small mute">Update checks are off on this machine (organisation policy).</div>
      {:else if upd.info?.available}<b>TUNAS {upd.info.latest} is ready</b><div class="small mute">You have {upd.info.current}.{#if upd.phase === 'downloading'} Downloading…{/if}</div>
      {:else if upd.info?.error}<b>TUNAS {app.info?.version}</b><div class="small mute">Could not check: {upd.info.error}</div>
      {:else if upd.info}<b>TUNAS {app.info?.version} · Up to date</b><div class="small mute">TUNAS checks once a day. Only the version is asked for; nothing about you is sent.</div>
      {:else}<b>TUNAS {app.info?.version}</b><div class="small mute">TUNAS checks for a new version once a day.</div>{/if}
    </div>
    {#if upd.info?.available}
      {#if upd.info.notes_url}<button class="btn sm ghost" onclick={whatsNew}>What's new</button>{/if}
      <button class="btn sm primary" disabled={!!upd.phase} onclick={updateNow}>Update</button>
    {:else if upd.info?.error !== 'off'}
      <button class="btn sm" disabled={upd.checking} onclick={() => checkUpdate(true)}>Check now</button>
    {/if}
  </div>

  {#if credited.length}
    <div class="section-title">Brands on this copy</div>
    <div class="credits">
      {#each credited as l}
        <div class="card tight row">
          <BrandLogo src={l.logo_data_url} size={36} bg={l.brand.logo_bg} shape={l.brand.logo_shape} />
          <div class="grow"><div class="small faint">{LABEL[l.layer]}</div><b>{l.brand.name || 'Logo only'}</b>{#if l.brand.issuer}<div class="small faint">Pack made by {l.brand.issuer}</div>{/if}</div>
        </div>
      {/each}
    </div>
  {/if}

  <div class="section-title">What TERAS adds</div>
  <div class="card teras">
    <p class="mute" style="margin-bottom:14px">TUNAS is for one person on one machine. <b style="color:var(--ink)">TERAS</b>, the intelligent core of S.A.L.T CI, brings the same AI to a whole organisation, on its own servers.</p>
    <div class="tgrid">
      {#each teras as [icon, title, text]}
        <div class="titem"><Icon name={icon} size={18} /><div><b>{title}</b><div class="small mute">{text}</div></div></div>
      {/each}
    </div>
    <div class="row" style="margin-top:16px">
      <button class="btn primary" onclick={() => openUrl(SALT_LINKS.teras)}>Learn about TERAS <Icon name="external" size={14} /></button>
      <span class="small faint">www.salt.ci</span>
    </div>
  </div>

  <div class="section-title">Open-source components</div>
  <div class="card tight small mute">
    TUNAS includes llama.cpp (MIT licence) as its AI engine and is built with Tauri (MIT / Apache-2.0) and Svelte (MIT).
    Each AI model keeps its own licence, shown on the Models page.
  </div>

  <div class="stripwrap"><SaltBrandStrip /></div>

  <div class="legal small faint">
    <div>S.A.L.T CI · Owned by S.A.L.T. CI TECHNOLOGY · Distributed by Pemangkin Teknologi</div>
    <div>S.A.L.T CI and its components are proprietary technology of S.A.L.T. CI TECHNOLOGY, distributed and commercialised by Pemangkin Teknologi.</div>
    <div class="row" style="gap:14px;margin-top:6px;flex-wrap:wrap">
      <button class="lnk" onclick={() => openUrl(SALT_LINKS.salt)}>www.salt.ci</button>
      <button class="lnk" onclick={() => openUrl('mailto:info@salt.ci')}>info@salt.ci</button>
      <button class="lnk" onclick={() => openUrl(SALT_LINKS.pemangkin)}>www.pemangkin.tech</button>
      <button class="lnk" onclick={() => openUrl('mailto:info@pemangkin.tech')}>info@pemangkin.tech</button>
    </div>
  </div>
</div>

<style>
  .page { max-width: 980px; }
  .hero { background: linear-gradient(135deg, color-mix(in srgb, #2F74D8 8%, var(--panel)), color-mix(in srgb, #E3C254 10%, var(--panel))); }
  .credits { display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px; }
  .tgrid { display: grid; grid-template-columns: 1fr 1fr; gap: 14px 22px; }
  .titem { display: flex; gap: 10px; align-items: flex-start; }
  .titem :global(.icon) { color: var(--brand1); margin-top: 2px; }
  .stripwrap { margin-top: 26px; border-radius: 14px; overflow: hidden; }
  .legal { margin-top: 14px; display: flex; flex-direction: column; gap: 3px; }
  .lnk { color: var(--brand1); padding: 0; font-size: 12px; }
  .lnk:hover { text-decoration: underline; }
  .updrow { align-items: center; gap: 12px; }
  @media (max-width: 760px) { .credits, .tgrid { grid-template-columns: 1fr; } .updrow { flex-wrap: wrap; } }
</style>
