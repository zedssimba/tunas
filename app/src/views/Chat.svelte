<script>
  import { onMount, tick } from 'svelte';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import Icon from '../lib/Icon.svelte';
  import BrandLogo from '../lib/BrandLogo.svelte';
  import { api, pickFiles } from '../lib/api.js';
  import { md } from '../lib/markdown.js';
  import { buildMessages, streamChat, makeTitle } from '../lib/chat.js';
  import { app, toast, refreshChats, assistantName, currentAssistant, modelName } from '../lib/state.svelte.js';

  let messages = $state([]);
  let input = $state('');
  let files = $state([]); // {name, path, kind, text, chars, truncated, warning, loading}
  let busy = $state(false);
  let streaming = $state('');
  let abort = null;
  let scroller;
  let ta;
  let dragging = $state(false);
  let pickerOpen = $state(false);
  let loadedChat = null;
  let supported = [];

  const assistant = $derived(currentAssistant());
  const noModels = $derived(app.installed.filter((m) => !m.id.includes('embed')).length === 0);

  $effect(() => {
    const id = app.chatId;
    if (id !== loadedChat) {
      loadedChat = id;
      load(id);
    }
  });

  async function load(id) {
    if (!id) { messages = []; return; }
    messages = await api.messages_list({ chatId: id });
    await scrollDown(true);
  }

  async function scrollDown(force = false) {
    await tick();
    if (!scroller) return;
    const near = scroller.scrollHeight - scroller.scrollTop - scroller.clientHeight < 160;
    if (force || near) scroller.scrollTop = scroller.scrollHeight;
  }

  function autosize() {
    if (!ta) return;
    ta.style.height = 'auto';
    ta.style.height = Math.min(ta.scrollHeight, 220) + 'px';
  }

  function maxCharsPerFile() {
    const ctx = app.engine?.ctx || 8192;
    return Math.floor((ctx * 3.3 * 0.55) / Math.max(1, files.length + 1));
  }

  async function addPaths(paths) {
    const fresh = paths.filter((p) => !files.some((f) => f.path === p));
    if (!fresh.length) return;
    const bad = fresh.filter((p) => !supported.includes(p.split('.').pop().toLowerCase()));
    if (bad.length) toast(`Not supported yet: ${bad.map((p) => p.split(/[\\/]/).pop()).join(', ')}`, 'error');
    const ok = fresh.filter((p) => !bad.includes(p));
    if (!ok.length) return;
    files = [...files, ...ok.map((p) => ({ path: p, name: p.split(/[\\/]/).pop(), loading: true }))];
    const res = await api.files_extract({ paths: ok, maxChars: maxCharsPerFile() });
    files = files.map((f) => {
      const i = ok.indexOf(f.path);
      if (i < 0) return f;
      const r = res[i];
      if (r.Ok) return { ...r.Ok, loading: false };
      if (r.Err) return { ...f, loading: false, error: r.Err };
      return { ...r, loading: false };
    });
    const failed = files.filter((f) => f.error);
    for (const f of failed) toast(f.error, 'error');
    files = files.filter((f) => !f.error);
  }

  async function attach() {
    const p = await pickFiles([{ name: 'Documents', extensions: supported }], 'Add files to this chat');
    if (p.length) addPaths(p);
  }

  async function ensureEngine() {
    const want = assistant?.model_id || '';
    const st = app.engine;
    if (st?.state === 'ready' && (!want || st.model_id === want)) {
      // also check default model setting matches when assistant has none
      if (want || !app.prefs.default_model || st.model_id === app.prefs.default_model) return st;
    }
    return await api.engine_ensure({ modelId: want });
  }

  async function send(textOverride) {
    const text = (textOverride ?? input).trim();
    if ((!text && !files.length) || busy) return;
    if (noModels) { app.view = 'models'; return; }
    if (files.some((f) => f.loading)) { toast('Still reading your files…'); return; }
    busy = true;
    const sentFiles = files.map(({ name, kind, text, chars, truncated, warning }) => ({ name, kind, text, chars, truncated, warning }));
    input = '';
    files = [];
    autosize();
    try {
      let chatId = app.chatId;
      if (!chatId) {
        const c = await api.chat_create({ assistantId: assistant.id, title: (text || sentFiles[0]?.name || 'New chat').slice(0, 48) });
        chatId = c.id;
        loadedChat = chatId;
        app.chatId = chatId;
        messages = [];
        refreshChats();
      }
      const um = await api.message_add({
        message: { id: '', chat_id: chatId, role: 'user', content: text || 'Please look at the attached file.', files: sentFiles, stats: '', created: '' },
      });
      messages = [...messages, um];
      await scrollDown(true);
      await runAssistant(chatId, messages.length === 1);
    } catch (e) {
      toast(String(e.message || e), 'error', 8000);
    } finally {
      busy = false;
      streaming = '';
      abort = null;
      await tick();
      ta?.focus();
    }
  }

  async function runAssistant(chatId, first) {
    const engine = await ensureEngine();
    streaming = '…';
    abort = new AbortController();
    const msgs = buildMessages(assistant, messages, engine.ctx || 8192);
    let text = '';
    let stats = null;
    try {
      const r = await streamChat({
        engine,
        messages: msgs,
        signal: abort.signal,
        onToken: (t) => { streaming = t; scrollDown(); },
      });
      text = r.text;
      stats = r.stats;
    } catch (e) {
      if (e.name === 'AbortError') {
        text = streaming === '…' ? '' : streaming;
        stats = { stopped: true };
      } else {
        throw e;
      }
    }
    if (!text.trim()) {
      if (stats?.stopped) return;
      text = '_No answer was produced. Try again, or try another model._';
    }
    const am = await api.message_add({
      message: {
        id: '', chat_id: chatId, role: 'assistant', content: text, files: [],
        stats: JSON.stringify({ ...(stats || {}), model: engine.model_id, engine: engine.engine }), created: '',
      },
    });
    messages = [...messages, am];
    streaming = '';
    await scrollDown();
    if (first && !stats?.stopped) {
      const userText = messages[0]?.content || '';
      const title = await makeTitle({ engine, userText, answer: text });
      if (title) { await api.chat_rename({ id: chatId, title }); refreshChats(); }
    } else {
      refreshChats();
    }
  }

  function stop() { abort?.abort(); }

  async function regenerate() {
    if (busy || !app.chatId) return;
    const last = messages[messages.length - 1];
    if (!last || last.role !== 'assistant') return;
    await api.messages_delete_from({ chatId: app.chatId, messageId: last.id });
    messages = messages.slice(0, -1);
    busy = true;
    try { await runAssistant(app.chatId, false); }
    catch (e) { toast(String(e.message || e), 'error', 8000); }
    finally { busy = false; streaming = ''; abort = null; }
  }

  async function editAndResend(m) {
    if (busy) return;
    await api.messages_delete_from({ chatId: app.chatId, messageId: m.id });
    const idx = messages.findIndex((x) => x.id === m.id);
    messages = messages.slice(0, idx);
    input = m.content;
    files = (m.files || []).map((f) => ({ ...f, loading: false }));
    await tick();
    autosize();
    ta?.focus();
  }

  async function copy(text) {
    await navigator.clipboard.writeText(text);
    toast('Copied.', 'ok', 1500);
  }

  function onKey(e) {
    if (e.key === 'Enter' && !e.shiftKey && !e.isComposing) {
      e.preventDefault();
      send();
    }
  }

  function chooseAssistant(a) {
    pickerOpen = false;
    if (a.id === app.assistantId) return;
    app.assistantId = a.id;
    api.setting_set({ key: 'last_assistant', value: a.id });
    if (app.chatId) { app.chatId = null; }
  }

  function onMdClick(e) {
    const b = e.target.closest('[data-copy]');
    if (b) {
      const code = b.parentElement.querySelector('code');
      copy(code?.innerText || '');
    }
  }

  function parseStats(s) {
    try { return s ? JSON.parse(s) : null; } catch { return null; }
  }

  const suggestions = $derived.by(() => {
    const id = assistant?.id || '';
    if (id.includes('surat')) return ['Tulis surat rasmi memohon cuti tanpa gaji selama 3 bulan', 'Surat jemputan kepada Ketua Jabatan untuk merasmikan program'];
    if (id.includes('official')) return ['Write an official letter requesting a site visit next week', 'Draft a letter confirming attendance at a workshop'];
    if (id.includes('minit')) return ['Sediakan minit daripada nota mesyuarat yang saya tampal', 'Format minit ringkas dengan jadual tindakan'];
    if (id.includes('meeting')) return ['Turn my notes into meeting minutes with an action table', 'Summarise the decisions from this transcript'];
    if (id.includes('email')) return ['Write a polite follow-up email about a late payment', 'Tulis emel ringkas memaklumkan mesyuarat ditangguhkan'];
    if (id.includes('summar')) return ['Summarise the attached document in five points', 'Ringkaskan dokumen ini dan senaraikan tarikh penting'];
    return ['Explain the main points of the attached PDF', 'Bantu saya rancang jadual kerja minggu ini', 'Write a short speech for a staff gathering', 'Terangkan beza antara AI di peranti dan AI awan'];
  });

  onMount(() => {
    api.files_supported().then((s) => (supported = s));
    let unlisten;
    getCurrentWebview().onDragDropEvent((ev) => {
      const t = ev.payload.type;
      if (t === 'enter' || t === 'over') dragging = true;
      else if (t === 'leave') dragging = false;
      else if (t === 'drop') {
        dragging = false;
        if (app.view === 'chat' && ev.payload.paths?.length) addPaths(ev.payload.paths);
      }
    }).then((u) => (unlisten = u));
    ta?.focus();
    return () => unlisten?.();
  });
</script>

<div class="chat-view">
  <header>
    <div class="picker">
      <button class="who" onclick={() => (pickerOpen = !pickerOpen)}>
        <span class="av" style="background:{assistant?.color || 'var(--brand1)'}"><Icon name={assistant?.icon} size={15} stroke={2} /></span>
        <span class="ellipsis"><b>{assistantName(assistant)}</b></span>
        <Icon name="chevron" size={14} class="rot" />
      </button>
      {#if pickerOpen}
        <div class="menu card" role="menu">
          {#each app.assistants as a (a.id)}
            <button class="mi" class:sel={a.id === assistant?.id} onclick={() => chooseAssistant(a)}>
              <span class="av" style="background:{a.color || 'var(--brand1)'}"><Icon name={a.icon} size={14} stroke={2} /></span>
              <span class="grow ellipsis">{assistantName(a)}</span>
              {#if a.model_id}<span class="faint small">{modelName(a.model_id)}</span>{/if}
            </button>
          {/each}
          <button class="mi manage" onclick={() => { pickerOpen = false; app.view = 'assistants'; }}><Icon name="plus" size={14} /> Create or edit assistants</button>
        </div>
        <button class="scrim" aria-label="Close" onclick={() => (pickerOpen = false)}></button>
      {/if}
    </div>
    <div class="spacer"></div>
    {#if app.engine?.state === 'starting'}
      <span class="pill brand"><span class="dot pulse"></span>{app.engine.message}</span>
    {:else if app.engine?.state === 'ready'}
      <span class="pill" title="Running on this machine">
        <Icon name={app.engine.engine === 'vulkan' ? 'chip' : 'cpu'} size={13} />
        {modelName(app.engine.model_id) || app.engine.model_id} · {app.engine.engine === 'vulkan' ? 'Graphics' : 'Processor'}
      </span>
    {:else if app.engine?.state === 'error'}
      <span class="pill err" title={app.engine.message}><Icon name="warn" size={13} /> Engine problem</span>
    {/if}
    <span class="pill ok" title="Everything stays on this machine"><Icon name="lock" size={12} /> Offline &amp; private</span>
  </header>

  <div class="scroll" bind:this={scroller} onclick={onMdClick} role="presentation">
    <div class="thread">
      {#if !messages.length && !streaming}
        <div class="welcome">
          <BrandLogo src={app.brand?.logo_data_url} bg={app.brand?.logo_bg} shape={app.brand?.logo_shape} size={72} />
          <h1>{app.brand?.effective_welcome || `Hello, I'm ${assistantName(assistant)}.`}</h1>
          <p class="mute">{app.brand?.effective_welcome ? `I'm ${assistantName(assistant)}. ` : ''}Ask me anything in English or Bahasa Melayu, or drop a PDF, Word, Excel or PowerPoint file here.</p>
          {#if noModels}
            <div class="card tight notice">
              <Icon name="models" size={20} />
              <div class="grow"><b>No model installed yet.</b><div class="mute small">Download a model, or import one from a USB stick, to start chatting.</div></div>
              <button class="btn primary sm" onclick={() => (app.view = 'models')}>Open Models</button>
            </div>
          {:else}
            <div class="sugs">
              {#each suggestions as s}
                <button class="sug" onclick={() => { input = s; tick().then(autosize); ta?.focus(); }}>{s}</button>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      {#each messages as m, i (m.id)}
        {#if m.role === 'user'}
          <div class="msg user">
            {#if m.files?.length}
              <div class="fchips">
                {#each m.files as f}<span class="fchip"><Icon name="file" size={13} />{f.name}</span>{/each}
              </div>
            {/if}
            <div class="bubble">{m.content}</div>
            <div class="tools">
              <button class="icon-btn" title="Copy" onclick={() => copy(m.content)}><Icon name="copy" size={14} /></button>
              <button class="icon-btn" title="Edit and send again" disabled={busy} onclick={() => editAndResend(m)}><Icon name="edit" size={14} /></button>
            </div>
          </div>
        {:else}
          {@const st = parseStats(m.stats)}
          <div class="msg bot">
            <span class="av big" style="background:{assistant?.color || 'var(--brand1)'}"><Icon name={assistant?.icon} size={16} stroke={2} /></span>
            <div class="grow">
              <div class="md">{@html md(m.content)}</div>
              <div class="tools">
                <button class="icon-btn" title="Copy" onclick={() => copy(m.content)}><Icon name="copy" size={14} /></button>
                {#if i === messages.length - 1}
                  <button class="icon-btn" title="Answer again" disabled={busy} onclick={regenerate}><Icon name="refresh" size={14} /></button>
                {/if}
                {#if st?.tps}<span class="faint small">{modelName(st.model) || st.model} · {st.tps} tokens/s</span>{/if}
                {#if st?.stopped}<span class="faint small">Stopped</span>{/if}
                {#if st?.finish === 'length'}<span class="faint small">Answer reached the length limit</span>{/if}
              </div>
            </div>
          </div>
        {/if}
      {/each}

      {#if streaming}
        <div class="msg bot">
          <span class="av big" style="background:{assistant?.color || 'var(--brand1)'}"><Icon name={assistant?.icon} size={16} stroke={2} /></span>
          <div class="grow">
            {#if streaming === '…'}
              <div class="typing"><i></i><i></i><i></i>{#if app.engine?.state === 'starting'}<span class="faint small">{app.engine.message}</span>{/if}</div>
            {:else}
              <div class="md">{@html md(streaming)}</div>
            {/if}
          </div>
        </div>
      {/if}
    </div>
  </div>

  <div class="composer-wrap">
    <div class="composer" class:drag={dragging}>
      {#if files.length}
        <div class="fchips">
          {#each files as f, i (f.path)}
            <span class="fchip" title={f.warning || (f.truncated ? 'Only the first part of this file fits' : `${f.chars?.toLocaleString()} characters`)} class:warn={f.warning || f.truncated}>
              <Icon name={f.loading ? 'refresh' : f.warning ? 'warn' : 'file'} size={13} />
              {f.name}{#if f.truncated}<span class="faint"> · part</span>{/if}
              <button aria-label="Remove" onclick={() => (files = files.filter((_, j) => j !== i))}><Icon name="x" size={12} /></button>
            </span>
          {/each}
        </div>
      {/if}
      <div class="row inner">
        <button class="icon-btn" title="Add files (PDF, Word, Excel, PowerPoint, text)" onclick={attach}><Icon name="clip" size={18} /></button>
        <textarea bind:this={ta} bind:value={input} rows="1" placeholder={`Message ${assistantName(assistant)}…`}
          oninput={autosize} onkeydown={onKey}></textarea>
        {#if busy && abort}
          <button class="send stop" title="Stop" onclick={stop}><Icon name="stop" size={16} /></button>
        {:else}
          <button class="send" title="Send" disabled={busy || (!input.trim() && !files.length)} onclick={() => send()}><Icon name="send" size={18} stroke={2.2} /></button>
        {/if}
      </div>
      {#if dragging}<div class="dropzone"><Icon name="upload" size={22} /> Drop files to add them to this chat</div>{/if}
    </div>
    <div class="foot faint small">Answers are made on this machine and can be wrong. Check important facts.</div>
  </div>
</div>

<style>
  .chat-view { display: flex; flex-direction: column; height: 100%; }
  header { display: flex; align-items: center; gap: 8px; padding: 12px 20px; border-bottom: 1px solid var(--line); background: var(--panel); }
  .picker { position: relative; }
  .who { display: flex; align-items: center; gap: 9px; padding: 5px 10px 5px 6px; border-radius: 10px; max-width: 360px; }
  .who:hover { background: var(--hover); }
  .who :global(.rot) { transform: rotate(90deg); color: var(--faint); }
  .av { width: 26px; height: 26px; border-radius: 8px; display: inline-grid; place-items: center; color: #fff; flex: none; }
  .av.big { width: 30px; height: 30px; border-radius: 9px; margin-top: 2px; }
  .menu { position: absolute; top: 44px; left: 0; z-index: 20; width: 320px; padding: 6px; box-shadow: var(--shadow); max-height: 60vh; overflow-y: auto; }
  .scrim { position: fixed; inset: 0; z-index: 10; cursor: default; }
  .mi { display: flex; align-items: center; gap: 9px; width: 100%; padding: 7px 8px; border-radius: 8px; text-align: left; }
  .mi:hover { background: var(--hover); }
  .mi.sel { background: color-mix(in srgb, var(--brand1) 10%, transparent); }
  .mi.manage { color: var(--brand1); font-weight: 600; border-top: 1px solid var(--line); border-radius: 0 0 8px 8px; margin-top: 4px; padding-top: 10px; }
  .dot { width: 7px; height: 7px; border-radius: 50%; background: currentColor; }
  .pulse { animation: pulse 1.2s infinite; }
  @keyframes pulse { 50% { opacity: .3; } }

  .scroll { flex: 1; overflow-y: auto; }
  .thread { max-width: 820px; margin: 0 auto; padding: 26px 24px 20px; display: flex; flex-direction: column; gap: 22px; }
  .welcome { display: flex; flex-direction: column; align-items: center; text-align: center; gap: 12px; padding: 7vh 0 10px; }
  .welcome h1 { font-size: 26px; max-width: 640px; }
  .welcome p { max-width: 560px; }
  .sugs { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; width: 100%; max-width: 640px; margin-top: 14px; }
  .sug { text-align: left; padding: 12px 14px; border: 1px solid var(--line); border-radius: 12px; background: var(--panel); color: var(--mute); }
  .sug:hover { border-color: var(--brand1); color: var(--ink); }
  .notice { display: flex; align-items: center; gap: 12px; text-align: left; margin-top: 14px; width: 100%; max-width: 560px; }

  .msg { display: flex; gap: 12px; }
  .msg.user { flex-direction: column; align-items: flex-end; }
  .bubble { background: color-mix(in srgb, var(--brand1) 10%, var(--panel)); border: 1px solid color-mix(in srgb, var(--brand1) 18%, var(--line)); padding: 10px 14px; border-radius: 14px 14px 4px 14px; max-width: 80%; white-space: pre-wrap; overflow-wrap: anywhere; }
  .tools { display: flex; align-items: center; gap: 2px; min-height: 28px; opacity: 0; transition: opacity .15s; }
  .msg:hover .tools, .msg:last-child .tools { opacity: 1; }
  .tools .icon-btn { width: 28px; height: 28px; }
  .tools span { margin-left: 6px; }
  .typing { display: flex; align-items: center; gap: 4px; height: 26px; }
  .typing i { width: 7px; height: 7px; border-radius: 50%; background: var(--faint); animation: b 1.2s infinite; }
  .typing i:nth-child(2) { animation-delay: .15s; }
  .typing i:nth-child(3) { animation-delay: .3s; }
  .typing span { margin-left: 8px; }
  @keyframes b { 30% { transform: translateY(-4px); opacity: .5; } }

  .fchips { display: flex; flex-wrap: wrap; gap: 6px; }
  .fchip { display: inline-flex; align-items: center; gap: 5px; padding: 3px 8px; font-size: 12px; border-radius: 7px; background: var(--soft); border: 1px solid var(--line); max-width: 280px; }
  .fchip.warn { border-color: color-mix(in srgb, var(--warn) 45%, var(--line)); }
  .fchip button { display: inline-grid; place-items: center; color: var(--faint); }

  .composer-wrap { padding: 8px 24px 12px; }
  .composer { position: relative; max-width: 820px; margin: 0 auto; background: var(--panel); border: 1px solid var(--line); border-radius: 16px; padding: 8px; box-shadow: var(--shadow); display: flex; flex-direction: column; gap: 8px; }
  .composer:focus-within { border-color: color-mix(in srgb, var(--brand1) 50%, var(--line)); }
  .composer .fchips { padding: 2px 4px 0; }
  .inner { align-items: flex-end; }
  .composer textarea { border: 0; box-shadow: none; resize: none; padding: 7px 4px; max-height: 220px; background: transparent; }
  .send { width: 38px; height: 38px; border-radius: 11px; display: grid; place-items: center; color: #fff; background: var(--grad); flex: none; }
  .send.stop { background: var(--ink); color: var(--panel); }
  .composer.drag { border-color: var(--brand1); border-style: dashed; }
  .dropzone { position: absolute; inset: 0; border-radius: 16px; display: flex; align-items: center; justify-content: center; gap: 8px; background: color-mix(in srgb, var(--brand1) 10%, var(--panel)); color: var(--brand1); font-weight: 600; }
  .foot { text-align: center; margin-top: 6px; }
</style>
