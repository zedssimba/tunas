// Thin wrapper over the TUNAS backend commands and events.
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open, save } from '@tauri-apps/plugin-dialog';
import { openUrl } from '@tauri-apps/plugin-opener';

export const api = new Proxy({}, {
  get: (_, name) => (args = {}) => invoke(String(name), args),
});

export { listen, openUrl };

export async function pickFile(filters, title) {
  const r = await open({ multiple: false, directory: false, filters, title });
  return typeof r === 'string' ? r : r?.path ?? null;
}

export async function pickFiles(filters, title) {
  const r = await open({ multiple: true, directory: false, filters, title });
  if (!r) return [];
  return (Array.isArray(r) ? r : [r]).map((x) => (typeof x === 'string' ? x : x.path));
}

export async function pickFolder(title) {
  const r = await open({ multiple: false, directory: true, title });
  return typeof r === 'string' ? r : null;
}

export async function saveFile(defaultPath, filters, title) {
  return await save({ defaultPath, filters, title });
}

export function fmtBytes(n) {
  if (!n && n !== 0) return '';
  const u = ['B', 'KB', 'MB', 'GB', 'TB'];
  let i = 0;
  while (n >= 1024 && i < u.length - 1) { n /= 1024; i++; }
  return `${n.toFixed(n >= 10 || i === 0 ? 0 : 1)} ${u[i]}`;
}

export function fmtEta(remaining, rate) {
  if (!rate || rate <= 0) return '';
  const s = remaining / rate;
  if (s < 60) return `${Math.ceil(s)} s left`;
  if (s < 3600) return `${Math.ceil(s / 60)} min left`;
  return `${(s / 3600).toFixed(1)} h left`;
}

export const SALT_LINKS = {
  salt: 'https://www.salt.ci',
  teras: 'https://www.salt.ci',
  pemangkin: 'https://www.pemangkin.tech',
  email: 'mailto:info@salt.ci',
};
