<script>
  import { onMount, onDestroy, tick } from 'svelte';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import Icon from '../lib/Icon.svelte';
  import BrandLogo from '../lib/BrandLogo.svelte';
  import { api, pickFiles, saveFile } from '../lib/api.js';
  import { kn, newSetFromFolder, setLabel, refreshKnowledge } from '../lib/knowledge.svelte.js';
  import { md } from '../lib/markdown.js';
  import { buildMessages, streamChat, makeTitle, terasFiles, withoutTerasFileLinks } from '../lib/chat.js';
  import { app, toast, refreshChats, assistantName, currentAssistant, modelName, openSettings } from '../lib/state.svelte.js';

  let messages = $state([]);
  let input = $state('');
  let files = $state([]); // {name, path, kind, text, chars, truncated, warning, loading}
  let busy = $state(false);
  let streaming = $state('');
  // Smooth typing: tokens arrive in bursts; the screen reveals them at an even pace and re-renders
  // the formatted text at most every 60 ms.
  let target = '';
  let shownLen = 0;
  let streamHtml = $state('');
  let liveText = $state('');
  let lastRender = 0;
  let raf = 0;
  let thinkText = $state('');
  let thinkStart = $state(0);
  let thinkNow = $state(0);
  let thinkOpen = $state(true);
  let thinkModeOpen = $state(false);
  let openThoughts = $state({});

  function pump(ts) {
    if (shownLen < target.length) {
      const gap = target.length - shownLen;
      shownLen += Math.max(1, Math.ceil(gap / 10));
      if (ts - lastRender > 60 || shownLen >= target.length) {
        liveText = target.slice(0, shownLen);
        streamHtml = md(liveText);
        lastRender = ts;
        scrollDown();
      }
    }
    if (thinkStart) thinkNow = performance.now();
    raf = requestAnimationFrame(pump);
  }
  function startPump() { target = ''; shownLen = 0; streamHtml = ''; liveText = ''; lastRender = 0; cancelAnimationFrame(raf); raf = requestAnimationFrame(pump); }
  function stopPump() { cancelAnimationFrame(raf); raf = 0; }

  let abort = null;
  let titleCtl = null;
  // Knowledge: the assistant's own sets, plus sets picked for this chat with the # button.
  let chatSets = $state([]);
  let assistantSets = $state([]);
  let kOpen = $state(false);
  let liveSources = $state([]);
  let liveActivity = $state([]);
  // Friendly names for what TERAS is doing.
  const TOOL_NAMES = { create_document: 'Making the document', render_document: 'Checking the document', web_search: 'Searching the web', fetch_url: 'Reading a web page', knowledge_search: 'Searching knowledge', search_knowledge: 'Searching knowledge', run_code: 'Running code', python: 'Running code' };
  const toolName = (t) => TOOL_NAMES[t] || (t || 'Tool').replace(/[_-]+/g, ' ').replace(/^./, (c) => c.toUpperCase());
  const provOf = (model) => (typeof model === 'string' && model.startsWith('prov:') ? model.split(':')[1] : '');
  async function terasDownload(st, f) {
    const pid = provOf(st?.model);
    const path = await saveFile(f.name, [{ name: 'File', extensions: [f.name.split('.').pop() || '*'] }], 'Save the file from TERAS');
    if (!path) return;
    try {
      await api.teras_download({ providerId: pid, filePath: f.path, saveTo: path });
      toast(`Saved ${f.name}`, 'ok');
      api.open_written({ path }).catch(() => {});
    } catch (e) {
      const msg = String(e);
      if (msg.startsWith('auth')) {
        toast('TERAS asked for your sign-in for this file, so it opens in your browser.', 'ok', 6000);
        api.teras_open({ providerId: pid, filePath: f.path }).catch((x) => toast(String(x), 'error'));
      } else toast(msg, 'error', 8000);
    }
  }
  let saveOpen = $state(null);
  $effect(() => {
    const id = assistant?.id;
    if (id) api.knowledge_for_assistant({ assistantId: id }).then((v) => (assistantSets = v));
    else assistantSets = [];
  });
  async function saveChatSets(ids) {
    chatSets = ids;
    if (app.chatId) await api.setting_set({ key: `chat_knowledge:${app.chatId}`, value: JSON.stringify(ids) });
  }
  function toggleSet(id) {
    saveChatSets(chatSets.includes(id) ? chatSets.filter((x) => x !== id) : [...chatSets, id]);
  }
  async function newChatSet() {
    kOpen = false;
    const id = await newSetFromFolder();
    if (id) saveChatSets([...chatSets, id]);
  }
  const setName = (id) => kn.sets.find((x) => x.id === id)?.name;
  function setNote(id) {
    const ks = kn.sets.find((x) => x.id === id);
    if (!ks || ks.state === 'ready') return '';
    if (ks.state === 'waiting_model') return ' · waiting for the search model';
    if (ks.state === 'indexing') return ` · reading ${ks.done}/${ks.files - ks.failed}`;
    if (ks.state === 'paused') return ' · paused';
    if (ks.state === 'empty') return ' · no readable files';
    return '';
  }
  // The citation numbers an answer actually uses: [2], [1][3], [1, 4], [2-3].
  function citedNumbers(text) {
    const set = new Set();
    for (const m of (text || '').matchAll(/\[(\d+(?:\s*[,\u2013-]\s*\d+)*)\]/g)) {
      for (const part of m[1].split(',')) {
        const [a, b] = part.split(/[\u2013-]/).map((x) => parseInt(x.trim(), 10));
        if (Number.isFinite(b) && b >= a && b - a < 20) { for (let i = a; i <= b; i++) set.add(i); }
        else if (Number.isFinite(a)) set.add(a);
      }
    }
    return set;
  }
  // One entry per file, with all its citation numbers: "Release_Notes.md [1][2][5]".
  // Only the pieces the answer cites are listed; an answer that cites nothing lists no sources.
  function groupSources(list, text) {
    const cited = citedNumbers(text);
    const by = new Map();
    for (const src of (list || []).filter((x) => cited.has(x.n))) {
      const g = by.get(src.path) || { path: src.path, name: src.name, set: src.set, items: [] };
      g.items.push(src);
      by.set(src.path, g);
    }
    return [...by.values()];
  }
  const srcTitle = (g) => [g.path || 'From TERAS', ...g.items.map((x) => `[${x.n}] ${x.label || 'whole file'}`)].join('\n');
  async function openSource(src) {
    if (!src.path) return; // TERAS's own sources live on TERAS
    try { await api.open_file({ path: src.path }); } catch (e) { toast(String(e), 'error'); }
  }

  // Documents: requests that ask for a document get a ready file card under the answer.
  const DOC_WORDS = /\b(surat|letter|memo|memorandum|minit|minutes|laporan|report|kertas|proposal|cadangan|notis|notice|pekeliling|circular|jadual|table|senarai|invois|invoice|sebut harga|quotation|agenda|sijil|certificate|resume|cv|kenyataan|borang)\b/i;
  function wantsDoc(i) {
    const prev = messages[i - 1];
    return prev?.role === 'user' && DOC_WORDS.test(prev.content || '');
  }
  function docTitle(text) {
    const lines = (text || '').split('\n').map((l) => l.trim());
    const clean = (l) => l.replace(/^#+\s*/, '').replace(/[*_`]/g, '').replace(/\s*\[\d+\]/g, '').trim().slice(0, 60);
    const h = lines.find((l) => /^#{1,3}\s/.test(l));
    if (h) return clean(h);
    // Letters and memos: the subject line.
    const subj = lines.find((l) => /^\**\s*(perkara|tajuk|subject|re|ref\.? ?:? ?perkara)\s*[:：]/i.test(l));
    if (subj) { const t = clean(subj.replace(/^\**\s*[^:：]+[:：]\s*/, '')); if (t) return t; }
    // A short line in bold on its own, such as **MINIT MESYUARAT ...**.
    const bold = lines.find((l) => /^\*\*[^*]{6,80}\*\*$/.test(l));
    if (bold) return clean(bold);
    return (app.chats.find((c) => c.id === app.chatId)?.title || 'TUNAS document').slice(0, 60);
  }
  const hasTable = (text) => /^\s*\|.*\|\s*$/m.test(text || '') && /^\s*\|?\s*:?-{3,}/m.test(text || '');
  async function saveAs(m, format) {
    saveOpen = null;
    const title = docTitle(m.content);
    const safe = title.replace(/[\\/:*?"<>|]/g, '').trim() || 'TUNAS document';
    const names = { docx: 'Word document', xlsx: 'Excel workbook', pdf: 'PDF' };
    const path = await saveFile(`${safe}.${format}`, [{ name: names[format], extensions: [format] }], 'Save the document');
    if (!path) return;
    try {
      await api.doc_export({ markdown: m.content, format, path, title, letterhead: true });
      toast(`Saved ${safe}.${format}`, 'ok');
      api.open_written({ path }).catch(() => {});
    } catch (e) {
      toast(String(e), 'error', 8000);
    }
  }
  // Leaving the chat screen stops the answer (what was written so far is kept), so the engine is free
  // for the next question instead of finishing an answer nobody is watching.
  onDestroy(() => { abort?.abort(); titleCtl?.abort(); stopPump(); });
  let scroller;
  let ta;
  let dragging = $state(false);
  let pickerOpen = $state(false);
  let loadedChat = null;
  let supported = [];

  const assistant = $derived(currentAssistant());
  const noModels = $derived(app.installed.filter((m) => !m.id.includes('embed')).length === 0 && !effModel.startsWith('prov:'));

  $effect(() => {
    const id = app.chatId;
    if (id !== loadedChat) {
      // Switching to another chat stops the answer in the one being left.
      abort?.abort();
      loadedChat = id;
      load(id);
    }
  });

  async function load(id) {
    if (!id) { messages = []; modelOverride = ''; chatSets = []; return; }
    modelOverride = (await api.setting_get({ key: `chat_model:${id}` })) || '';
    try { chatSets = JSON.parse((await api.setting_get({ key: `chat_knowledge:${id}` })) || '[]'); } catch { chatSets = []; }
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
    const ctx = route?.ctx || app.engine?.ctx || 8192;
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

  let route = $state({ kind: 'local', label: '', provider: '', ctx: 8192, error: '' });
  let modelOverride = $state('');
  let modelOpen = $state(false);
  const effModel = $derived(modelOverride || assistant?.model_id || '');
  $effect(() => {
    const m = effModel;
    const _d = app.prefs.default_model + app.installed.length + app.providers.length + app.prefs.cloud_enabled;
    api.route_info({ modelId: m }).then((r) => (route = r));
  });

  async function send(textOverride) {
    const text = (textOverride ?? input).trim();
    if ((!text && !files.length) || busy) return;
    if (noModels) { openSettings('models'); return; }
    if (files.some((f) => f.loading)) { toast('Still reading your files…'); return; }
    busy = true;
    titleCtl?.abort(); // a chat title still being written must not make this question wait
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
        if (modelOverride) api.setting_set({ key: `chat_model:${chatId}`, value: modelOverride });
        if (chatSets.length) api.setting_set({ key: `chat_knowledge:${chatId}`, value: JSON.stringify(chatSets) });
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
    const r = await api.route_info({ modelId: effModel });
    route = r;
    if (r.error) throw new Error(r.error);
    const mode = r.kind === 'local' ? (app.thinkModes[r.model_id] || 'off') : 'off';
    streaming = '…';
    liveSources = [];
    liveActivity = [];
    thinkText = '';
    thinkStart = 0;
    thinkOpen = mode === 'shown';
    startPump();
    abort = new AbortController();
    const msgs = buildMessages(assistant, messages, r.ctx || 8192);
    let text = '';
    let reasoning = '';
    let sources = [];
    let stats = null;
    try {
      const res = await streamChat({
        assistant,
        model: effModel,
        messages: msgs,
        signal: abort.signal,
        think: mode !== 'off',
        knowledge: $state.snapshot(chatSets),
        onSources: (src) => { liveSources = src; },
        onActivity: (a) => { liveActivity = a; scrollDown(); },
        onThinking: (t) => { if (!thinkStart) thinkStart = performance.now(); if (mode === 'shown') { thinkText = t; scrollDown(); } },
        onToken: (t) => {
          if (thinkStart && streaming === '…' && mode === 'shown') thinkOpen = false; // fold the thoughts when the answer starts
          streaming = t;
          target = t;
        },
      });
      text = res.text;
      reasoning = res.reasoning;
      sources = res.sources || [];
      stats = res.stats;
    } catch (e) {
      if (e.name === 'AbortError') {
        text = streaming === '…' ? '' : streaming;
        stats = { stopped: true };
      } else {
        stopPump();
        if (e instanceof TypeError) throw new Error('The answer stopped because the engine was stopped or restarted. Send your message again to continue.');
        throw e;
      }
    }
    // Let the last words appear before saving (at most 1.5 s; not when stopped).
    const tEnd = Date.now() + 1500;
    while (raf && !stats?.stopped && shownLen < target.length && Date.now() < tEnd) await new Promise((r) => setTimeout(r, 16));
    stopPump();
    if (!text.trim()) {
      if (stats?.stopped) { streaming = ''; return; }
      text = '_No answer was produced. Try again, or try another model._';
    }
    const extra = { model: route.kind === 'local' ? app.engine?.model_id : effModel, engine: route.kind === 'local' ? app.engine?.engine : route.kind };
    liveActivity = [];
    if (mode === 'shown' && reasoning) extra.reasoning = reasoning;
    if (sources.length) extra.sources = sources;
    const am = await api.message_add({
      message: { id: '', chat_id: chatId, role: 'assistant', content: text, files: [], stats: JSON.stringify({ ...(stats || {}), ...extra }), created: '' },
    });
    if (app.chatId === chatId) messages = [...messages, am];
    streaming = '';
    thinkText = '';
    thinkStart = 0;
    await scrollDown();
    if (first && !stats?.stopped) {
      // The title is written in the background: the user can ask the next question straight away.
      const userText = messages[0]?.content || '';
      const ctl = new AbortController();
      titleCtl = ctl;
      setTimeout(() => ctl.abort(), 60000);
      makeTitle({ model: effModel, userText, answer: text, signal: ctl.signal }).then(async (title) => {
        if (title) { await api.chat_rename({ id: chatId, title }); refreshChats(); }
      });
    } else {
      refreshChats();
    }
  }

  // Thinking mode per local model: off | hidden | shown.
  const thinkMode = $derived(route.kind === 'local' ? (app.thinkModes[route.model_id] || 'off') : 'off');
  $effect(() => {
    const id = route.kind === 'local' ? route.model_id : '';
    if (id && app.thinkModes[id] === undefined) {
      api.setting_get({ key: `think:${id}` }).then((v) => { app.thinkModes = { ...app.thinkModes, [id]: v || 'off' }; });
      api.setting_get({ key: `think_len:${id}` }).then((v) => { app.thinkLens = { ...app.thinkLens, [id]: v || 'normal' }; });
    }
  });
  async function setThink(m) {
    thinkModeOpen = false;
    app.thinkModes = { ...app.thinkModes, [route.model_id]: m };
    await api.setting_set({ key: `think:${route.model_id}`, value: m });
  }
  async function setLen(v) {
    app.thinkLens = { ...app.thinkLens, [route.model_id]: v };
    await api.setting_set({ key: `think_len:${route.model_id}`, value: v });
  }
  const THINK_LABEL = { off: 'Thinking off', hidden: 'Thinking (hidden)', shown: 'Thinking (shown)' };
  const secs = (ms) => (ms / 1000).toFixed(ms < 10000 ? 1 : 0);

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

  async function chooseModel(id) {
    modelOpen = false;
    modelOverride = id;
    if (app.chatId) await api.setting_set({ key: `chat_model:${app.chatId}`, value: id });
  }

  const modelGroups = $derived.by(() => {
    const g = [{ label: 'On this machine', items: app.installed.filter((m) => !m.id.includes('embed')).map((m) => ({ id: m.id, name: m.name })) }];
    const cloudOk = app.prefs.cloud_enabled === '1' && app.brand?.policy?.allow_cloud !== false;
    for (const p of app.providers) {
      if (!p.enabled || !p.models.length || (p.kind !== 'teras' && !cloudOk)) continue;
      g.push({ label: p.kind === 'teras' ? `${p.name} (your organisation)` : `${p.name} (cloud, online)`, items: p.models.map((m) => ({ id: `prov:${p.id}:${m}`, name: m })) });
    }
    return g;
  });

  function chooseAssistant(a) {
    pickerOpen = false;
    if (a.id === app.assistantId) return;
    app.assistantId = a.id;
    modelOverride = '';
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
    <div class="picker">
      <button class="mchip" class:over={!!modelOverride} onclick={() => (modelOpen = !modelOpen)} title="Model for this chat">
        <Icon name={route.kind === 'cloud' ? 'globe' : route.kind === 'teras' ? 'server' : 'models'} size={14} />
        <span class="ellipsis">{route.label || 'No model'}</span>
        <Icon name="chevron" size={12} class="rot" />
      </button>
      {#if modelOpen}
        <div class="menu card" role="menu">
          <button class="mi" class:sel={!modelOverride} onclick={() => chooseModel('')}>
            <span class="grow">Assistant's model</span><span class="faint small">{assistant?.model_id ? modelName(assistant.model_id) : 'Default'}</span>
          </button>
          {#each modelGroups as g}
            {#if g.items.length}
              <div class="glabel">{g.label}</div>
              {#each g.items as m}
                <button class="mi" class:sel={modelOverride === m.id} onclick={() => chooseModel(m.id)}><span class="grow ellipsis">{m.name}</span></button>
              {/each}
            {/if}
          {/each}
          <button class="mi manage" onclick={() => { modelOpen = false; openSettings('models'); }}><Icon name="plus" size={14} /> Get more models</button>
        </div>
        <button class="scrim" aria-label="Close" onclick={() => (modelOpen = false)}></button>
      {/if}
    </div>
    {#if route.kind === 'local'}
      <div class="picker">
        <button class="mchip" class:over={thinkMode !== 'off'} onclick={() => (thinkModeOpen = !thinkModeOpen)} title="Let this model think before answering">
          <Icon name="bulb" size={14} /><span>{THINK_LABEL[thinkMode]}</span><Icon name="chevron" size={12} class="rot" />
        </button>
        {#if thinkModeOpen}
          <div class="menu card think" role="menu">
            <button class="mi" class:sel={thinkMode === 'off'} onclick={() => setThink('off')}><span class="grow"><b>Off</b><span class="d">Fastest. Answers straight away.</span></span></button>
            <button class="mi" class:sel={thinkMode === 'hidden'} onclick={() => setThink('hidden')}><span class="grow"><b>On, hidden</b><span class="d">Thinks first for better answers; shows only "Thinking…" and the time.</span></span></button>
            <button class="mi" class:sel={thinkMode === 'shown'} onclick={() => setThink('shown')}><span class="grow"><b>On, shown</b><span class="d">Shows the thinking live as it happens, then folds it away. Good for classes.</span></span></button>
            {#if thinkMode !== 'off'}
              <div class="lenrow">
                <span class="small mute">How long to think</span>
                <div class="seg sm">
                  {#each [['short', 'Short'], ['normal', 'Normal'], ['long', 'Long']] as [v, l]}
                    <button class:sel={(app.thinkLens[route.model_id] || 'normal') === v} onclick={() => setLen(v)}>{l}</button>
                  {/each}
                </div>
              </div>
            {/if}
            <div class="small faint" style="padding:6px 8px 2px">Saved for {route.label}. Not every model can think.</div>
          </div>
          <button class="scrim" aria-label="Close" onclick={() => (thinkModeOpen = false)}></button>
        {/if}
      </div>
    {/if}
    <div class="spacer"></div>
    {#if route.kind === 'cloud'}
      <span class="pill warn" title="This assistant uses a cloud model"><Icon name="external" size={12} /> Online · leaves this machine</span>
    {:else if route.kind === 'teras'}
      <span class="pill ok" title="Sent only to your organisation's TERAS server"><Icon name="lock" size={12} /> Your organisation's server</span>
    {:else}
      {#if app.engine?.state === 'starting'}
        <span class="pill brand"><span class="dot pulse"></span>{app.engine.message}</span>
      {:else if app.engine?.state === 'ready'}
        <span class="pill" title="Running on this machine">
          <Icon name={app.engine.engine === 'vulkan' ? 'chip' : 'cpu'} size={13} />
          {app.engine.engine === 'vulkan' ? 'Graphics' : 'Processor'}
        </span>
      {:else if app.engine?.state === 'error'}
        <span class="pill err" title={app.engine.message}><Icon name="warn" size={13} /> Engine problem</span>
      {/if}
      <span class="pill ok" title="Everything stays on this machine"><Icon name="lock" size={12} /> Offline &amp; private</span>
    {/if}
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
              <button class="btn primary sm" onclick={() => openSettings('models')}>Open Models</button>
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
              {#if st?.think_ms}
                {#if st.reasoning}
                  <button class="thought" onclick={() => (openThoughts = { ...openThoughts, [m.id]: !openThoughts[m.id] })}>
                    <Icon name="bulb" size={13} /> Thought for {secs(st.think_ms)} s <Icon name="chevron" size={11} class={openThoughts[m.id] ? 'down' : ''} />
                  </button>
                  {#if openThoughts[m.id]}<div class="thinkbox md">{@html md(st.reasoning)}</div>{/if}
                {:else}
                  <div class="thought plain"><Icon name="bulb" size={13} /> Thought for {secs(st.think_ms)} s</div>
                {/if}
              {/if}
              <div class="md">{@html md(withoutTerasFileLinks(m.content))}</div>
              {#if groupSources(st?.sources, m.content).length}
                <div class="sources">
                  <span class="small faint">Sources</span>
                  {#each groupSources(st.sources, m.content) as g}
                    <button class="srcchip" title={srcTitle(g)} onclick={() => openSource(g)}>
                      <span class="nums">{#each g.items as x}<b>{x.n}</b>{/each}</span> {g.name}
                    </button>
                  {/each}
                </div>
              {/if}
              {#if terasFiles(m.content).length && provOf(st?.model)}
                {#each terasFiles(m.content) as f (f.path)}
                  <div class="doccard">
                    <Icon name="file" size={18} />
                    <div class="grow"><b>{f.name}</b><div class="small mute">Made by TERAS{f.size ? ` · ${f.size}` : ''}</div></div>
                    <button class="btn sm primary" onclick={() => terasDownload(st, f)}>Download</button>
                  </div>
                {/each}
              {:else if wantsDoc(i) && !st?.stopped}
                <div class="doccard">
                  <Icon name="file" size={18} />
                  <div class="grow"><b>Your document is ready</b><div class="small mute">{docTitle(m.content)} · with your letterhead</div></div>
                  <button class="btn sm primary" onclick={() => saveAs(m, 'docx')}>Word</button>
                  <button class="btn sm" onclick={() => saveAs(m, 'pdf')}>PDF</button>
                  {#if hasTable(m.content)}<button class="btn sm" onclick={() => saveAs(m, 'xlsx')}>Excel</button>{/if}
                </div>
              {/if}
              <div class="tools">
                <button class="icon-btn" title="Copy" onclick={() => copy(m.content)}><Icon name="copy" size={14} /></button>
                <span class="savewrap">
                  <button class="icon-btn" title="Save as Word, PDF or Excel" onclick={() => (saveOpen = saveOpen === m.id ? null : m.id)}><Icon name="download" size={14} /></button>
                  {#if saveOpen === m.id}
                    <div class="menu card savemenu">
                      <button class="mi" onclick={() => saveAs(m, 'docx')}><Icon name="file" size={14} /> Word (.docx)</button>
                      <button class="mi" onclick={() => saveAs(m, 'pdf')}><Icon name="file" size={14} /> PDF</button>
                      <button class="mi" onclick={() => saveAs(m, 'xlsx')}><Icon name="file" size={14} /> Excel (.xlsx){#if !hasTable(m.content)}<span class="faint small"> · text only</span>{/if}</button>
                    </div>
                    <button class="scrim" aria-label="Close" onclick={() => (saveOpen = null)}></button>
                  {/if}
                </span>
                {#if i === messages.length - 1}
                  <button class="icon-btn" title="Answer again" disabled={busy} onclick={regenerate}><Icon name="refresh" size={14} /></button>
                {/if}
                {#if st?.remote}<span class="faint small">{modelName(st.model) || st.model} · {secs(st.total_ms)} s{#if st.tools?.length} · {st.tools.length} tool {st.tools.length === 1 ? 'step' : 'steps'}{/if}</span>
                {:else if st?.tps && !st?.est}<span class="faint small">{modelName(st.model) || st.model} · {st.tps} tokens/s</span>
                {:else if st?.model}<span class="faint small">{modelName(st.model) || st.model}</span>{/if}
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
            {#if thinkStart}
              {#if thinkMode === 'shown' && thinkText}
                <button class="thought live" onclick={() => (thinkOpen = !thinkOpen)}>
                  <Icon name="bulb" size={13} /> {streaming === '…' ? 'Thinking' : 'Thought'} · {secs(thinkNow - thinkStart)} s <Icon name="chevron" size={11} class={thinkOpen ? 'down' : ''} />
                </button>
                {#if thinkOpen}<div class="thinkbox live md">{@html md(thinkText)}</div>{/if}
              {:else if streaming === '…'}
                <div class="thought plain pulse"><Icon name="bulb" size={13} /> Thinking… {secs(thinkNow - thinkStart)} s</div>
              {/if}
            {/if}
            {#if liveActivity.length}
              <div class="activity">
                {#each liveActivity as a (a.id)}
                  <span class="act" class:run={a.state === 'running'} class:bad={a.state === 'failed'}>
                    {#if a.state === 'running'}<i class="spin"></i>{:else if a.state === 'failed'}<Icon name="x" size={12} />{:else}<Icon name="check" size={12} />{/if}
                    {toolName(a.tool)}{#if a.state === 'failed'} · failed, retrying{:else if a.state === 'done' && a.ms} · {secs(a.ms)} s{:else if a.state === 'running'}…{/if}
                  </span>
                {/each}
              </div>
            {/if}
            {#if streaming === '…'}
              {#if !thinkStart}
                <div class="typing"><i></i><i></i><i></i>{#if app.engine?.state === 'starting'}<span class="faint small">{app.engine.message}</span>{:else}<span class="faint small">Reading your message…</span>{/if}</div>
              {/if}
            {:else}
              <div class="md">{@html streamHtml}</div>
            {/if}
            {#if groupSources(liveSources, liveText).length}
              <div class="sources">
                <span class="small faint">Sources</span>
                {#each groupSources(liveSources, liveText) as g}<span class="srcchip" title={srcTitle(g)}><span class="nums">{#each g.items as x}<b>{x.n}</b>{/each}</span> {g.name}</span>{/each}
              </div>
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
      {#if chatSets.length || assistantSets.length}
        <div class="kbar">
          <Icon name="book" size={13} />
          <span class="small faint">Answering from</span>
          {#each assistantSets as id}{#if setName(id)}<span class="kchip fixed" class:wait={setNote(id)} title="Part of this assistant">{setName(id)}{setNote(id)}</span>{/if}{/each}
          {#each chatSets as id}{#if setName(id)}<span class="kchip" class:wait={setNote(id)}>{setName(id)}{setNote(id)}<button aria-label="Remove" onclick={() => toggleSet(id)}><Icon name="x" size={11} /></button></span>{/if}{/each}
        </div>
      {/if}
      <div class="row inner">
        <button class="icon-btn" title="Add files (PDF, Word, Excel, PowerPoint, text)" onclick={attach}><Icon name="clip" size={18} /></button>
        <span class="kwrap">
          <button class="icon-btn" class:on={chatSets.length} title="Use knowledge in this chat (#)" onclick={() => { kOpen = !kOpen; if (kOpen) refreshKnowledge(); }}><Icon name="hash" size={18} /></button>
          {#if kOpen}
            <div class="menu card kmenu">
              <div class="small faint" style="padding:4px 8px 6px">Knowledge for this chat</div>
              {#each kn.sets as ks}
                <button class="mi" class:sel={chatSets.includes(ks.id) || assistantSets.includes(ks.id)} disabled={assistantSets.includes(ks.id)} onclick={() => toggleSet(ks.id)}>
                  <Icon name={chatSets.includes(ks.id) || assistantSets.includes(ks.id) ? 'check' : 'book'} size={14} />
                  <span class="grow">{ks.name}<span class="d">{assistantSets.includes(ks.id) ? 'Part of this assistant' : setLabel(ks)}</span></span>
                </button>
              {/each}
              <button class="mi" onclick={newChatSet}><Icon name="folder" size={14} /><span class="grow">New from folder…<span class="d">Read a folder on this machine, with your permission</span></span></button>
            </div>
            <button class="scrim" aria-label="Close" onclick={() => (kOpen = false)}></button>
          {/if}
        </span>
        <textarea bind:this={ta} bind:value={input} rows="1" placeholder={`Message ${assistantName(assistant)}…`}
          oninput={(e) => { autosize(); if (input === '#') { input = ''; kOpen = true; refreshKnowledge(); } }} onkeydown={onKey}></textarea>
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
  .sources { display: flex; flex-wrap: wrap; gap: 6px; align-items: center; margin: 8px 0 2px; }
  .srcchip { display: inline-flex; gap: 5px; align-items: center; font-size: 12px; padding: 3px 9px; border-radius: 99px; border: 1px solid var(--line); background: var(--panel); max-width: 320px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  button.srcchip:hover { border-color: var(--brand1); color: var(--brand1); }
  .srcchip .nums { display: inline-flex; gap: 3px; }
  .srcchip b { color: var(--brand1); font-size: 11px; min-width: 16px; height: 16px; padding: 0 4px; border-radius: 99px; display: inline-grid; place-items: center; background: color-mix(in srgb, var(--brand1) 12%, transparent); }
.activity { display: flex; flex-wrap: wrap; gap: 6px; margin: 2px 0 8px; }
  .act { display: inline-flex; gap: 6px; align-items: center; font-size: 12px; padding: 3px 10px; border-radius: 99px; background: var(--soft); color: var(--mute); border: 1px solid var(--line); }
  .act.run { color: var(--brand1); border-color: color-mix(in srgb, var(--brand1) 35%, transparent); }
  .act.bad { color: #b45309; }
  .act .spin { width: 10px; height: 10px; border-radius: 50%; border: 2px solid currentColor; border-right-color: transparent; animation: spin .8s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
  .thinkbox.md :global(p) { margin: 0 0 6px; }
  .thinkbox.md { white-space: normal; }
  .doccard { display: flex; gap: 10px; align-items: center; margin: 10px 0 4px; padding: 10px 12px; border: 1px solid var(--line); border-radius: 12px; background: var(--soft); max-width: 560px; }
  .savewrap, .kwrap { position: relative; display: inline-flex; }
  .savemenu { top: auto; bottom: 30px; width: 220px; }
  .kmenu { top: auto; bottom: 42px; width: 320px; }
  .kmenu .d { display: block; font-size: 11.5px; color: var(--faint); font-weight: 400; }
  .icon-btn.on { color: var(--brand1); }
  .kbar { display: flex; flex-wrap: wrap; gap: 6px; align-items: center; padding: 6px 10px 0; }
  .kchip { display: inline-flex; gap: 4px; align-items: center; font-size: 12px; font-weight: 600; padding: 2px 6px 2px 9px; border-radius: 99px; background: color-mix(in srgb, var(--brand1) 10%, transparent); color: var(--brand1); }
  .kchip.fixed { padding-right: 9px; }
  .kchip.wait { color: var(--warn, #b7791f); background: color-mix(in srgb, var(--warn, #b7791f) 12%, transparent); }
  .kchip button { display: inline-flex; opacity: .7; }
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
  .mchip { display: flex; align-items: center; gap: 7px; height: 32px; padding: 0 10px; border-radius: 9px; border: 1px solid var(--line); color: var(--mute); font-weight: 600; font-size: 13px; max-width: 260px; }
  .mchip:hover { background: var(--hover); color: var(--ink); }
  .mchip.over { border-color: color-mix(in srgb, var(--brand1) 45%, var(--line)); color: var(--ink); }
  .mchip :global(.rot) { transform: rotate(90deg); color: var(--faint); }
  .glabel { font-size: 11px; font-weight: 700; color: var(--faint); text-transform: uppercase; letter-spacing: .07em; margin: 8px 8px 2px; }
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
  .thought { display: inline-flex; align-items: center; gap: 6px; font-size: 12.5px; color: var(--mute); padding: 3px 9px; border-radius: 99px; background: var(--soft); margin-bottom: 8px; }
  .thought :global(.down) { transform: rotate(90deg); }
  .thought.plain { cursor: default; }
  .thought.pulse { animation: pulse 1.4s infinite; }
  .thinkbox { font-size: 12.5px; line-height: 1.55; color: var(--mute); font-style: italic; white-space: pre-wrap; border-left: 2px solid var(--line); padding: 2px 0 2px 12px; margin: 0 0 10px; max-height: 260px; overflow-y: auto; }
  .menu.think { width: 330px; }
  .lenrow { display: flex; align-items: center; justify-content: space-between; gap: 8px; padding: 8px 8px 4px; border-top: 1px solid var(--line); margin-top: 4px; }
  .seg { display: inline-flex; border: 1px solid var(--line); border-radius: 9px; padding: 2px; gap: 2px; background: var(--bg); }
  .seg button { padding: 3px 9px; border-radius: 7px; font-size: 12px; font-weight: 600; color: var(--mute); }
  .seg button.sel { background: var(--panel); color: var(--ink); box-shadow: 0 1px 3px rgba(0,0,0,.08); }
  .menu .d { display: block; font-size: 12px; color: var(--faint); font-weight: 400; }
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
