<script>
  import { onMount } from 'svelte';
  import { api } from '../lib/api.js';
  import { app, toast } from '../lib/state.svelte.js';

  // A short brief about the person, sent with every message to local and cloud models so answers fit them.
  // TERAS models use the person's own TERAS personalisation instead.
  const FIELDS = [
    ['call', 'What should TUNAS call you?', 'e.g. Puan Nurul, Badrul, Dr. Huda', 60],
    ['position', 'Your position', 'e.g. Penolong Pegawai Tadbir, Senior Lecturer', 80],
    ['organisation', 'Organisation or department', 'e.g. Fakulti Kejuruteraan, Jabatan Kewangan', 100],
    ['work', 'What you work on', 'e.g. staff circulars, procurement, teaching mechanical design', 240],
    ['languages', 'Languages', 'e.g. Bahasa Melayu for letters, English for reports', 100],
    ['style', 'How you like answers', 'e.g. short and to the point; formal; with examples', 160],
  ];
  let form = $state({ on: true, call: '', position: '', organisation: '', work: '', languages: '', style: '' });
  let loaded = $state(false);

  onMount(async () => {
    try {
      const raw = await api.setting_get({ key: 'about_me' });
      if (raw) form = { ...form, ...JSON.parse(raw) };
    } catch {}
    loaded = true;
  });

  const orgHint = $derived(app.brand?.letterhead_org || app.brand?.effective_name || '');

  async function save() {
    const clean = { ...form };
    for (const [k, , , max] of FIELDS) clean[k] = (clean[k] || '').trim().slice(0, max);
    await api.setting_set({ key: 'about_me', value: JSON.stringify(clean) });
    form = clean;
    toast('Saved. New answers will use it.', 'ok');
  }
  async function clearAll() {
    form = { on: true, call: '', position: '', organisation: '', work: '', languages: '', style: '' };
    await api.setting_set({ key: 'about_me', value: JSON.stringify(form) });
    toast('Cleared.', 'ok');
  }
</script>

<div class="page">
  <div class="page-head"><div class="grow"><h1>About me</h1><p>A few lines about you, so answers fit you and your work. Fill in only what you like.</p></div></div>

  {#if loaded}
    <div class="stack">
      <div class="card set">
        <div class="grow"><b>Use this in my chats</b><div class="small mute">Sent with each message to the models on this machine and to cloud models. TERAS models use your own TERAS personalisation instead.</div></div>
        <label class="switch"><input type="checkbox" bind:checked={form.on} onchange={save} /><span></span></label>
      </div>

      <div class="card fields" class:off={!form.on}>
        {#each FIELDS as [k, label, ph, max]}
          <label class="field">
            <span class="lab">{label}</span>
            {#if k === 'work' || k === 'style'}
              <textarea rows="2" maxlength={max} placeholder={ph} bind:value={form[k]}></textarea>
            {:else}
              <input type="text" maxlength={max} placeholder={k === 'organisation' && orgHint ? `e.g. ${orgHint}` : ph} bind:value={form[k]} />
            {/if}
          </label>
        {/each}
        <div class="row">
          <span class="small faint grow">Kept on this machine only. It is never shared with other apps.</span>
          <button class="btn" onclick={clearAll}>Clear</button>
          <button class="btn primary" onclick={save}>Save</button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .stack { display: flex; flex-direction: column; gap: 10px; max-width: 900px; }
  .set { display: flex; align-items: center; gap: 18px; padding: 16px 18px; }
  .fields { padding: 16px 18px; display: flex; flex-direction: column; gap: 12px; }
  .fields.off { opacity: .55; }
  .field { display: flex; flex-direction: column; gap: 5px; }
  .lab { font-weight: 600; font-size: 13px; }
  .field input, .field textarea { width: 100%; }
  .row { display: flex; align-items: center; gap: 8px; margin-top: 4px; }
  .switch { position: relative; width: 42px; height: 24px; flex: none; }
  .switch input { opacity: 0; width: 0; height: 0; }
  .switch span { position: absolute; inset: 0; border-radius: 99px; background: var(--line); transition: .15s; cursor: pointer; }
  .switch span::after { content: ''; position: absolute; left: 3px; top: 3px; width: 18px; height: 18px; border-radius: 50%; background: #fff; transition: .15s; }
  .switch input:checked + span { background: var(--brand1); }
  .switch input:checked + span::after { transform: translateX(18px); }
</style>
