// "A new TUNAS is ready": checked quietly at start and once a day (see update.rs).
import { api, listen, openUrl } from './api.js';
import { app, toast, setPref } from './state.svelte.js';

export const upd = $state({
  info: null, // { current, latest, available, date, notes_url, file, error }
  checking: false,
  phase: '', // '' | downloading | installing
  done: 0,
  total: 0,
});

const DAY = 24 * 60 * 60 * 1000;
let timer = null;

export function startUpdateChecks() {
  listen('update-progress', (e) => { upd.done = e.payload.done; upd.total = e.payload.total; });
  setTimeout(() => checkUpdate(), 15000);
  timer = setInterval(() => checkUpdate(), DAY);
}

export async function checkUpdate(manual = false) {
  if (upd.checking || upd.phase) return;
  upd.checking = true;
  try {
    upd.info = await api.update_check();
    if (manual && upd.info.error && upd.info.error !== 'off') toast(upd.info.error, 'error', 6000);
  } catch (e) {
    if (manual) toast(String(e), 'error', 6000);
  } finally {
    upd.checking = false;
  }
}

/** Show the card unless the person chose Later for this version. */
export const updateReady = () => !!upd.info?.available && app.prefs.update_later !== upd.info.latest;
export const updateAvailable = () => !!upd.info?.available;

export function later() {
  if (upd.info?.latest) setPref('update_later', upd.info.latest);
}

export function whatsNew() {
  if (upd.info?.notes_url) openUrl(upd.info.notes_url);
}

export async function updateNow() {
  const f = upd.info?.file;
  if (!f || upd.phase) return;
  upd.phase = 'downloading';
  upd.done = 0;
  upd.total = f.size || 0;
  try {
    const path = await api.update_download({ file: f });
    upd.phase = 'installing';
    if (app.info?.os === 'android') {
      const err = window.TunasAndroid?.installUpdate(path);
      if (err) throw new Error(err);
      toast('Android will ask to install the update. If it asks first, allow TUNAS to install apps.', 'ok', 9000);
    } else {
      await api.update_install({ path });
      if (app.info?.os === 'macos') toast(`Drag TUNAS into Applications and choose Replace, then open TUNAS ${upd.info.latest}.`, 'ok', 12000);
    }
  } catch (e) {
    toast(String(e?.message || e), 'error', 9000);
  } finally {
    upd.phase = '';
  }
}
