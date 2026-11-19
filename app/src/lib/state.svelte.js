// Shared app state (Svelte 5 runes).
import { api, listen } from './api.js';

export const app = $state({
  info: null,
  brand: null,
  assistants: [],
  installed: [],
  catalog: [],
  chats: [],
  engine: { state: 'stopped' },
  progress: {}, // model id -> progress
  view: 'chat', // chat | assistants | settings | setup
  settingsTab: 'general', // general | models | branding | connections | about
  backView: 'chat',
  chatId: null,
  assistantId: 'starter-general',
  chatQuery: '',
  toasts: [],
  theme: 'light',
  prefs: { engine_pref: 'auto', default_model: '', cloud_enabled: '0' },
  gateway: { port: 0, key: '' },
  providers: [],
});

let toastSeq = 0;
export function toast(text, kind = 'info', ms = 4200) {
  const id = ++toastSeq;
  app.toasts.push({ id, text, kind });
  setTimeout(() => { app.toasts = app.toasts.filter((t) => t.id !== id); }, ms);
}

export async function refreshBrand() {
  app.brand = await api.brand_state();
  applyBrandColors();
}

export function applyBrandColors() {
  const b = app.brand;
  const r = document.documentElement.style;
  r.setProperty('--brand1', b?.color1 || '#2F74D8');
  r.setProperty('--brand2', b?.color2 || '#6D4FE0');
  document.title = b?.effective_name || 'TUNAS';
}

export async function refreshAssistants() { app.assistants = await api.assistants_list(); }
export async function refreshModels() {
  const [installed, catalog] = await Promise.all([api.models_installed(), api.catalog_list()]);
  app.installed = installed;
  app.catalog = catalog;
}
export async function refreshProviders() { app.providers = await api.providers_list(); }
export async function refreshChats() { app.chats = await api.chats_list({ query: app.chatQuery || '' }); }

export async function loadPrefs() {
  for (const k of Object.keys(app.prefs)) {
    const v = await api.setting_get({ key: k });
    if (v != null) app.prefs[k] = v;
  }
  const t = await api.setting_get({ key: 'theme' });
  if (t) app.theme = t;
  document.documentElement.dataset.theme = app.theme;
  const last = await api.setting_get({ key: 'last_assistant' });
  if (last) app.assistantId = last;
}

export async function setPref(key, value) {
  app.prefs[key] = value;
  await api.setting_set({ key, value: String(value) });
}

export async function setTheme(t) {
  app.theme = t;
  document.documentElement.dataset.theme = t;
  await api.setting_set({ key: 'theme', value: t });
}

/** The display name for an assistant: the General Assistant takes the brand's assistant name if set. */
export function assistantName(a) {
  if (!a) return '';
  if (a.id === 'starter-general' && app.brand?.effective_assistant_name) return app.brand.effective_assistant_name;
  return a.name;
}

export function currentAssistant() {
  return app.assistants.find((a) => a.id === app.assistantId) || app.assistants[0] || null;
}

export function modelName(id) {
  if (!id) return '';
  if (id.startsWith('prov:')) {
    const [, pid, ...rest] = id.split(':');
    const p = app.providers.find((x) => x.id === pid);
    return `${p ? p.name : 'Removed provider'} · ${rest.join(':')}`;
  }
  const m = app.installed.find((x) => x.id === id);
  return m ? m.name : '';
}

export async function initEvents() {
  await listen('engine-status', (e) => { app.engine = e.payload; });
  await listen('model-progress', (e) => {
    const p = e.payload;
    app.progress = { ...app.progress, [p.id]: p };
    if (p.phase === 'done') {
      refreshModels();
      toast(`${nameFor(p.id)} is installed and ready.`, 'ok');
      clearProgress(p.id, 1500);
    } else if (p.phase === 'error') {
      toast(`${nameFor(p.id)}: ${p.message}`, 'error', 8000);
      clearProgress(p.id, 0);
    } else if (p.phase === 'cancelled') {
      clearProgress(p.id, 0);
    }
  });
  await listen('brand-changed', (e) => { app.brand = e.payload; applyBrandColors(); });
}

function nameFor(id) {
  return app.catalog.find((m) => m.id === id)?.name || app.installed.find((m) => m.id === id)?.name || 'Model';
}

function clearProgress(id, ms) {
  setTimeout(() => {
    const p = { ...app.progress };
    delete p[id];
    app.progress = p;
  }, ms);
}

export function openSettings(tab = 'general') {
  if (app.view !== 'settings') app.backView = app.view;
  app.settingsTab = tab;
  app.view = 'settings';
}

export function closeSettings() {
  app.view = app.backView && app.backView !== 'settings' ? app.backView : 'chat';
}
