<script>
  import Icon from '../lib/Icon.svelte';
  import { api } from '../lib/api.js';
  import { ASSISTANT_ICONS, ASSISTANT_COLORS } from '../lib/icons.js';
  import { app, toast, refreshAssistants, refreshProviders, assistantName, modelName } from '../lib/state.svelte.js';
  import { kn, newSetFromFolder, setLabel, refreshKnowledge } from '../lib/knowledge.svelte.js';

  let editSets = $state([]);
  let addOpen = $state(false);
  const isCloud = $derived(!!edit?.model_id?.startsWith('prov:') && !app.providers.some((p) => p.kind === 'teras' && edit.model_id.startsWith(`prov:${p.id}:`)));
  async function newSet() {
    const id = await newSetFromFolder();
    if (id) editSets = [...editSets, id];
  }

  let edit = $state(null); // working copy
  let confirmDel = $state(false);
  const canCreate = $derived(app.brand?.policy?.allow_create_assistants !== false);
  const chatModels = $derived(app.installed.filter((m) => !m.id.includes('embed')));

  function blank() {
    return { id: '', name: '', icon: 'chat', color: ASSISTANT_COLORS[Math.floor(Math.random() * 6)], persona: '', model_id: '', starter: false, sort: 100, created: '', updated: '' };
  }

  async function open(a) {
    edit = a ? { ...a } : blank();
    confirmDel = false;
    addOpen = false;
    editSets = a?.id ? await api.knowledge_for_assistant({ assistantId: a.id }) : [];
    refreshKnowledge();
  }
  async function duplicate(a) { const sets = await api.knowledge_for_assistant({ assistantId: a.id }); edit = { ...a, id: '', name: `${assistantName(a)} (copy)`, starter: false }; editSets = sets; }

  async function save() {
    try {
      const saved = await api.assistant_save({ assistant: $state.snapshot(edit) });
      await api.knowledge_set_for_assistant({ assistantId: saved.id, setIds: $state.snapshot(editSets) });
      refreshKnowledge();
      await refreshAssistants();
      toast(`${saved.name} saved.`, 'ok');
      edit = null;
    } catch (e) { toast(String(e), 'error'); }
  }

  async function remove() {
    await api.assistant_delete({ id: edit.id });
    if (app.assistantId === edit.id) app.assistantId = 'starter-general';
    await refreshAssistants();
    toast('Assistant deleted.');
    edit = null;
  }

  function chatWith(a) {
    app.assistantId = a.id;
    api.setting_set({ key: 'last_assistant', value: a.id });
    app.chatId = null;
    app.view = 'chat';
  }

  const personaExamples = [
    ['Formal Malay letters', 'Anda membantu menulis surat rasmi dalam Bahasa Melayu mengikut format surat rasmi kerajaan. Tanya butiran yang tiada sebelum menulis versi akhir.'],
    ['Lecture helper', 'You help a university lecturer prepare lecture notes, quiz questions and marking rubrics. Keep explanations clear for undergraduate students.'],
    ['Report writer', 'You turn rough notes into a clear report with a title, summary, findings and recommendations. Ask what the report is for if it is unclear.'],
    ['Plain-language explainer', 'You explain complicated documents (circulars, policies, contracts) in plain, simple language, with a short list of what the reader needs to do.'],
  ];
</script>

<div class="page">
  <div class="page-head">
    <div class="grow">
      <h1>My assistants</h1>
      <p>Each assistant has its own name, persona and base model. Use the one that fits the job.</p>
    </div>
    <button class="btn primary" disabled={!canCreate} title={canCreate ? '' : 'Your organisation does not allow new assistants'} onclick={() => open(null)}><Icon name="plus" size={16} /> New assistant</button>
  </div>

  <div class="grid">
    {#each app.assistants as a (a.id)}
      <div class="card tight acard">
        <div class="row">
          <span class="av" style="background:{a.color || 'var(--brand1)'}"><Icon name={a.icon} size={18} stroke={2} /></span>
          <div class="grow">
            <div class="ellipsis"><b>{assistantName(a)}</b></div>
            <div class="small faint ellipsis">{a.model_id ? modelName(a.model_id) || 'Model not installed' : 'Default model'}{a.starter ? ' · Starter' : ''}{a.id.startsWith('pack-') ? ' · From branding pack' : ''}</div>
          </div>
        </div>
        <p class="persona mute small">{a.persona || 'No persona set.'}</p>
        <div class="row">
          <button class="btn sm primary" onclick={() => chatWith(a)}><Icon name="chat" size={14} /> Chat</button>
          <button class="btn sm" onclick={() => open(a)}><Icon name="edit" size={14} /> Edit</button>
          {#if canCreate}<button class="btn sm ghost" onclick={() => duplicate(a)}><Icon name="copy" size={14} /> Copy</button>{/if}
        </div>
      </div>
    {/each}
  </div>
</div>

{#if edit}
  <div class="overlay" role="presentation" onclick={(e) => e.target === e.currentTarget && (edit = null)}>
    <div class="drawer">
      <div class="row head">
        <h2 class="grow">{edit.id ? 'Edit assistant' : 'New assistant'}</h2>
        <button class="icon-btn" aria-label="Close" onclick={() => (edit = null)}><Icon name="x" /></button>
      </div>

      <div class="body col" style="gap:18px">
        <div class="preview">
          <span class="av lg" style="background:{edit.color}"><Icon name={edit.icon} size={24} stroke={2} /></span>
          <div><b>{edit.name || 'Assistant name'}</b><div class="small faint">{edit.model_id ? modelName(edit.model_id) : 'Default model'}</div></div>
        </div>

        <label class="field">Name
          <input type="text" bind:value={edit.name} maxlength="60" placeholder="e.g. Pembantu AI Kak Mela, Surat Rasmi, Lecture Helper" />
        </label>

        <div class="field">Icon and colour
          <div class="icons">
            {#each ASSISTANT_ICONS as ic}
              <button class="ic" class:sel={edit.icon === ic} title={ic} onclick={() => (edit.icon = ic)}><Icon name={ic} size={18} /></button>
            {/each}
          </div>
          <div class="colors">
            {#each ASSISTANT_COLORS as c}
              <button class="sw" class:sel={edit.color === c} style="background:{c}" aria-label={c} onclick={() => (edit.color = c)}></button>
            {/each}
          </div>
        </div>

        <label class="field">Persona
          <span class="hint">How it should behave: its role, tone, language and format. It always replies in the language you write in, unless you tell it otherwise here.</span>
          <textarea rows="8" bind:value={edit.persona} placeholder="You help me write formal letters in Malay. Ask for missing details first…"></textarea>
        </label>
        <div class="row" style="flex-wrap:wrap;gap:6px;margin-top:-10px">
          <span class="small faint">Start from:</span>
          {#each personaExamples as [label, text]}
            <button class="pill" onclick={() => (edit.persona = text)}>{label}</button>
          {/each}
        </div>

        <div class="field">Knowledge
          <span class="hint">Folders of your documents this assistant answers from. Read where they are; nothing is copied.</span>
          <div class="kchips">
            {#each editSets as id}
              {@const ks = kn.sets.find((x) => x.id === id)}
              {#if ks}
                <span class="kchip" title={setLabel(ks)}><Icon name="book" size={13} /> {ks.name}
                  <button aria-label="Remove" onclick={() => (editSets = editSets.filter((x) => x !== id))}><Icon name="x" size={12} /></button></span>
              {/if}
            {/each}
            <button class="pill" onclick={newSet}><Icon name="folder" size={13} /> {app.info?.os === 'android' ? 'New from documents' : 'New from folder'}</button>
            {#if kn.sets.some((x) => !editSets.includes(x.id))}
              <span class="addwrap">
                <button class="pill" onclick={() => (addOpen = !addOpen)}><Icon name="plus" size={13} /> Add existing</button>
                {#if addOpen}
                  <div class="menu card kmenu">
                    {#each kn.sets.filter((x) => !editSets.includes(x.id)) as ks}
                      <button class="mi" onclick={() => { editSets = [...editSets, ks.id]; addOpen = false; }}><Icon name="book" size={14} /><span class="grow">{ks.name}<span class="d">{setLabel(ks)}</span></span></button>
                    {/each}
                  </div>
                {/if}
              </span>
            {/if}
          </div>
          {#if editSets.length && isCloud}
            <span class="hint" style="color:var(--warn)">This assistant uses a cloud model: the parts of your documents used in an answer are sent to that provider.</span>
          {/if}
        </div>

        <label class="field">Base model
          <span class="hint">A model on this machine, your organisation's TERAS, or a cloud model. Connect TERAS and cloud providers in Connections.</span>
          <select bind:value={edit.model_id}>
            <option value="">Default model{app.prefs.default_model ? ` (${modelName(app.prefs.default_model)})` : ''}</option>
            <optgroup label="On this machine">
              {#each chatModels as m}<option value={m.id}>{m.name}</option>{/each}
            </optgroup>
            {#each app.providers.filter((p) => p.enabled && p.models.length && (p.kind === 'teras' || (app.prefs.cloud_enabled === '1' && app.brand?.policy?.allow_cloud !== false))) as p}
              <optgroup label={p.kind === 'teras' ? `${p.name} (your organisation)` : `${p.name} (cloud, online)`}>
                {#each p.models as m}<option value={`prov:${p.id}:${m}`}>{p.name} · {m}</option>{/each}
              </optgroup>
            {/each}
          </select>
          {#if edit.model_id.startsWith('prov:') && !app.providers.some((p) => p.kind === 'teras' && edit.model_id.startsWith(`prov:${p.id}:`))}
            <span class="hint" style="color:var(--warn)">This is a cloud model: what you type to this assistant is sent to the provider over the internet.</span>
          {/if}
        </label>
      </div>

      <div class="row foot">
        {#if edit.id}
          {#if confirmDel}
            <span class="small">Delete this assistant? Its chats stay.</span>
            <button class="btn sm danger" onclick={remove}>Delete</button>
            <button class="btn sm ghost" onclick={() => (confirmDel = false)}>Keep</button>
          {:else}
            <button class="btn sm ghost danger" onclick={() => (confirmDel = true)}><Icon name="trash" size={14} /> Delete</button>
          {/if}
        {/if}
        <div class="spacer"></div>
        <button class="btn" onclick={() => (edit = null)}>Cancel</button>
        <button class="btn primary" onclick={save}>Save</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 14px; }
  .acard { display: flex; flex-direction: column; gap: 12px; }
  .av { width: 36px; height: 36px; border-radius: 10px; display: grid; place-items: center; color: #fff; flex: none; }
  .av.lg { width: 48px; height: 48px; border-radius: 13px; }
  .kchips { display: flex; flex-wrap: wrap; gap: 6px; align-items: center; margin-top: 6px; }
  .kchip { display: inline-flex; align-items: center; gap: 5px; padding: 4px 6px 4px 9px; border-radius: 99px; background: var(--soft); font-size: 12.5px; font-weight: 600; }
  .kchip button { opacity: .6; display: inline-flex; }
  .kchip button:hover { opacity: 1; }
  .addwrap { position: relative; }
  .kmenu { position: absolute; top: 110%; left: 0; z-index: 20; width: 280px; padding: 6px; }
  .kmenu .mi { display: flex; gap: 8px; align-items: center; width: 100%; padding: 7px 8px; border-radius: 8px; text-align: left; font-size: 13px; }
  .kmenu .mi:hover { background: var(--soft); }
  .kmenu .d { display: block; font-size: 11.5px; color: var(--faint); }
  .persona { display: -webkit-box; -webkit-line-clamp: 3; line-clamp: 3; -webkit-box-orient: vertical; overflow: hidden; min-height: 54px; }
  .overlay { position: fixed; inset: 0; background: rgba(11, 16, 34, .4); z-index: 40; display: flex; justify-content: flex-end; }
  .drawer { width: 560px; max-width: 100%; height: 100%; background: var(--panel); display: flex; flex-direction: column; box-shadow: var(--shadow); }
  .head { padding: 18px 22px; border-bottom: 1px solid var(--line); }
  .body { flex: 1; overflow-y: auto; padding: 20px 22px; }
  .foot { padding: 14px 22px; border-top: 1px solid var(--line); }
  .preview { display: flex; align-items: center; gap: 12px; padding: 14px; border-radius: 12px; background: var(--soft); }
  .icons { display: grid; grid-template-columns: repeat(8, 1fr); gap: 6px; font-weight: 400; }
  .ic { height: 36px; border-radius: 9px; border: 1px solid var(--line); display: grid; place-items: center; color: var(--mute); }
  .ic.sel { border-color: var(--brand1); color: var(--brand1); background: color-mix(in srgb, var(--brand1) 9%, transparent); }
  .colors { display: flex; gap: 8px; margin-top: 4px; }
  .sw { width: 26px; height: 26px; border-radius: 50%; border: 2px solid transparent; outline: 2px solid transparent; }
  .sw.sel { outline-color: var(--ink); border-color: var(--panel); }
</style>
