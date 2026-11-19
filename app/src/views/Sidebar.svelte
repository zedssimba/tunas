<script>
  import PoweredBy from '../lib/PoweredBy.svelte';
  import Icon from '../lib/Icon.svelte';
  import BrandLogo from '../lib/BrandLogo.svelte';
  import { api } from '../lib/api.js';
  import { app, refreshChats, assistantName, openSettings } from '../lib/state.svelte.js';

  let searchTimer;
  let renaming = $state(null);
  let renameText = $state('');

  function newChat() {
    app.chatId = null;
    app.view = 'chat';
  }

  function openChat(c) {
    app.chatId = c.id;
    app.assistantId = c.assistant_id;
    app.view = 'chat';
  }

  function onSearch(e) {
    app.chatQuery = e.target.value;
    clearTimeout(searchTimer);
    searchTimer = setTimeout(refreshChats, 200);
  }

  async function del(c, ev) {
    ev.stopPropagation();
    await api.chat_delete({ id: c.id });
    if (app.chatId === c.id) app.chatId = null;
    refreshChats();
  }

  function startRename(c, ev) {
    ev.stopPropagation();
    renaming = c.id;
    renameText = c.title;
  }

  async function finishRename(c) {
    const t = renameText.trim();
    renaming = null;
    if (t && t !== c.title) {
      await api.chat_rename({ id: c.id, title: t });
      refreshChats();
    }
  }

  function group(chats) {
    const today = new Date(); today.setHours(0, 0, 0, 0);
    const week = new Date(today); week.setDate(week.getDate() - 7);
    const g = { Today: [], 'Previous 7 days': [], Earlier: [] };
    for (const c of chats) {
      const d = new Date(c.updated);
      if (d >= today) g.Today.push(c);
      else if (d >= week) g['Previous 7 days'].push(c);
      else g.Earlier.push(c);
    }
    return Object.entries(g).filter(([, v]) => v.length);
  }

  const aName = (id) => assistantName(app.assistants.find((a) => a.id === id));
</script>

<aside>
  <div class="brand">
    <BrandLogo src={app.brand?.logo_data_url} bg={app.brand?.logo_bg} shape={app.brand?.logo_shape} size={36} />
    <div class="grow">
      <div class="name ellipsis" title={app.brand?.effective_name}>{app.brand?.effective_name || 'TUNAS'}</div>
      <div class="sub ellipsis">{app.brand?.effective_name && app.brand.effective_name !== 'TUNAS' ? 'TUNAS · Personal AI' : 'Personal AI'}</div>
    </div>
  </div>

  <button class="btn primary new" onclick={newChat}><Icon name="plus" size={16} /> New chat</button>

  <div class="search">
    <Icon name="search" size={15} />
    <input type="search" placeholder="Search chats" value={app.chatQuery} oninput={onSearch} />
  </div>

  <div class="chats">
    {#each group(app.chats) as [label, list]}
      <div class="glabel">{label}</div>
      {#each list as c (c.id)}
        <div class="chat" class:active={app.chatId === c.id && app.view === 'chat'} role="button" tabindex="0"
          onclick={() => openChat(c)} onkeydown={(e) => e.key === 'Enter' && openChat(c)}>
          {#if renaming === c.id}
            <!-- svelte-ignore a11y_autofocus -->
            <input type="text" class="rename" bind:value={renameText} autofocus
              onclick={(e) => e.stopPropagation()}
              onblur={() => finishRename(c)}
              onkeydown={(e) => { if (e.key === 'Enter') e.target.blur(); if (e.key === 'Escape') { renaming = null; } }} />
          {:else}
            <div class="grow">
              <div class="ellipsis title">{c.title}</div>
              <div class="ellipsis faint small">{aName(c.assistant_id) || 'Assistant'}</div>
            </div>
            <div class="acts">
              <button class="icon-btn" title="Rename" onclick={(e) => startRename(c, e)}><Icon name="edit" size={14} /></button>
              <button class="icon-btn" title="Delete" onclick={(e) => del(c, e)}><Icon name="trash" size={14} /></button>
            </div>
          {/if}
        </div>
      {/each}
    {:else}
      <div class="empty faint small">{app.chatQuery ? 'No chats match your search.' : 'Your chats will appear here. They stay on this machine.'}</div>
    {/each}
  </div>

  <nav>
    <button class:active={app.view === 'assistants'} onclick={() => (app.view = 'assistants')}><Icon name="user" size={17} /> My assistants</button>
  </nav>

  <div class="foot">
    <button class="credit" onclick={() => openSettings('about')} title="About TUNAS and S.A.L.T CI"><PoweredBy /></button>
    <button class="gear" class:active={app.view === 'settings'} title="Settings" aria-label="Settings" onclick={() => openSettings('general')}><Icon name="settings" size={19} /></button>
  </div>
</aside>

<style>
  aside {
    width: 276px; flex: none; height: 100%; display: flex; flex-direction: column; gap: 10px;
    background: var(--side); border-right: 1px solid var(--line); padding: 16px 12px 10px;
  }
  .brand { display: flex; align-items: center; gap: 11px; padding: 2px 6px 6px; }
  .name { font-weight: 700; font-size: 15px; }
  .sub { font-size: 11.5px; color: var(--faint); }
  .new { width: 100%; height: 40px; }
  .search { display: flex; align-items: center; gap: 6px; padding: 0 10px; border: 1px solid var(--line); border-radius: 9px; color: var(--faint); background: var(--bg); }
  .search input { border: 0; background: transparent; padding: 7px 0; box-shadow: none; }
  .chats { flex: 1; overflow-y: auto; margin: 0 -4px; padding: 0 4px; }
  .glabel { font-size: 11px; font-weight: 700; color: var(--faint); text-transform: uppercase; letter-spacing: .07em; margin: 12px 8px 4px; }
  .chat { display: flex; align-items: center; gap: 6px; padding: 7px 8px; border-radius: 9px; cursor: pointer; }
  .chat:hover { background: var(--hover); }
  .chat.active { background: color-mix(in srgb, var(--brand1) 11%, transparent); }
  .chat .title { font-weight: 500; }
  .chat .acts { display: none; }
  .chat:hover .acts { display: flex; }
  .acts .icon-btn { width: 26px; height: 26px; }
  .rename { padding: 4px 8px; }
  .empty { padding: 14px 10px; }
  nav { display: flex; flex-direction: column; gap: 1px; border-top: 1px solid var(--line); padding-top: 8px; }
  nav button { display: flex; align-items: center; gap: 10px; padding: 7px 10px; border-radius: 8px; color: var(--mute); font-weight: 500; text-align: left; }
  nav button:hover { background: var(--hover); color: var(--ink); }
  nav button.active { color: var(--ink); background: var(--hover); }
  .foot { display: flex; align-items: center; gap: 6px; padding: 4px 2px 0 6px; }
  .credit { flex: 1; text-align: left; min-width: 0; }
  .gear { width: 36px; height: 36px; border-radius: 10px; display: grid; place-items: center; color: var(--mute); }
  .gear:hover, .gear.active { background: var(--hover); color: var(--ink); }
</style>
