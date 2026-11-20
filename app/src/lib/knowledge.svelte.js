// Knowledge and folder access, shared by the Knowledge page, the assistant panel, the chat and Settings.
import { api, listen, pickFolder, pickFiles } from './api.js';
import { app, toast, refreshModels } from './state.svelte.js';

export const EMBED_MODEL = 'qwen3-embed-0.6b';

export const kn = $state({
  model_ready: false,
  sets: [],
  // The consent step: { paths, resolve } while the dialog is open.
  consent: null,
});

let timer = 0;
export async function refreshKnowledge() {
  try {
    const r = await api.knowledge_list();
    kn.model_ready = r.model_ready;
    kn.sets = r.sets;
  } catch (e) {
    console.error(e);
  }
}

export async function initKnowledgeEvents() {
  await refreshKnowledge();
  // Indexing sends many updates; refresh at most twice a second.
  await listen('knowledge-changed', () => {
    if (timer) return;
    timer = setTimeout(() => { timer = 0; refreshKnowledge(); }, 500);
  });
  await listen('model-progress', (e) => {
    if (e.payload?.id === EMBED_MODEL && e.payload.phase === 'done') {
      refreshKnowledge();
      api.knowledge_kick();
    }
  });
}

export const baseName = (p) => (p || '').split(/[\\/]/).filter(Boolean).pop() || p;

/** Ask the user before TUNAS reads a place on this machine. Resolves true when they allow it. */
export async function askConsent(paths) {
  const todo = [];
  for (const p of paths) {
    if (!(await api.folder_is_allowed({ path: p }))) todo.push(p);
  }
  if (!todo.length) return true;
  const ok = await new Promise((resolve) => { kn.consent = { paths: todo, resolve }; });
  kn.consent = null;
  if (!ok) return false;
  for (const p of todo) await api.folder_allow({ path: p });
  return true;
}

/** The search model is needed once; start its download if it is missing. */
export async function ensureSearchModel() {
  if (kn.model_ready) return;
  if (app.progress[EMBED_MODEL]) return;
  try {
    await api.model_download({ id: EMBED_MODEL });
    toast('Downloading the search model (about 640 MB). Your files will be read when it is ready.', 'info', 7000);
  } catch (e) {
    toast(String(e), 'error', 8000);
  }
}

/** Pick a folder, ask for consent, and make a knowledge set from it. Returns the new set id or null. */
export async function newSetFromFolder(name) {
  const folder = await pickFolder('Choose a folder for this knowledge set');
  if (!folder) return null;
  if (!(await askConsent([folder]))) return null;
  try {
    const id = await api.knowledge_create({ name: name || baseName(folder), paths: [folder] });
    await refreshKnowledge();
    await ensureSearchModel();
    toast(`"${name || baseName(folder)}" is being read. You can use it straight away; answers improve as reading finishes.`, 'ok', 6000);
    return id;
  } catch (e) {
    toast(String(e), 'error', 8000);
    return null;
  }
}

/** Add a folder or files to an existing set (with consent). */
export async function addToSet(id, kind = 'folder') {
  const paths = kind === 'folder' ? [await pickFolder('Choose a folder to add')].filter(Boolean)
    : await pickFiles([{ name: 'Documents', extensions: ['pdf', 'docx', 'xlsx', 'xls', 'pptx', 'txt', 'md', 'csv', 'html'] }], 'Choose files to add');
  if (!paths.length) return;
  if (!(await askConsent(paths))) return;
  try {
    for (const p of paths) await api.knowledge_add_source({ id, path: p });
    await refreshKnowledge();
    await ensureSearchModel();
  } catch (e) {
    toast(String(e), 'error', 8000);
  }
}

export function setLabel(s) {
  if (s.state === 'ready') return `${s.done} file${s.done === 1 ? '' : 's'} ready`;
  if (s.state === 'indexing') return `Reading ${s.done} of ${s.files - s.failed}…`;
  if (s.state === 'waiting_model') return 'Waiting for the search model';
  if (s.state === 'paused') return `Paused · ${s.done} of ${s.files} read`;
  return 'No readable files yet';
}
