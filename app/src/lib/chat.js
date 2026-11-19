// Talking to the local engine (llama.cpp server, OpenAI-compatible) with streaming.
import { app, assistantName } from './state.svelte.js';

export function systemPrompt(assistant) {
  const name = assistantName(assistant) || 'TUNAS';
  const brand = app.brand?.effective_name && app.brand.effective_name !== 'TUNAS' ? app.brand.effective_name : '';
  const today = new Date().toLocaleDateString('en-GB', { weekday: 'long', day: 'numeric', month: 'long', year: 'numeric' });
  const lines = [
    `You are ${name}${brand && brand !== name ? `, the AI assistant of ${brand}` : ''}.`,
    'You run privately on this notebook in TUNAS, powered by S.A.L.T CI. Nothing the user writes leaves the device.',
    "Always reply in the language of the user's latest message: English if they write in English, Malay (Bahasa Melayu) if they write in Malay, and natural mixed Malay-English if they mix both. If they ask for a specific language, use it.",
    'Be clear, accurate and well organised. Use Markdown for headings, lists and tables when it helps. If you are not sure, say so instead of guessing.',
    `Today is ${today}.`,
  ];
  const persona = (assistant?.persona || '').trim();
  return persona ? `${lines.join('\n')}\n\nYour role:\n${persona}` : lines.join('\n');
}

/** Rough token estimate (good enough for budgeting). */
export function estTokens(s) {
  return Math.ceil((s || '').length / 3.3);
}

/** Turn stored messages into the API format, attaching file text, within the context budget. */
export function buildMessages(assistant, history, ctx) {
  const sys = systemPrompt(assistant);
  const budget = Math.max(1024, ctx - 1400 - estTokens(sys)); // leave room for the answer
  const out = [];
  let used = 0;
  for (let i = history.length - 1; i >= 0; i--) {
    const m = history[i];
    let content = m.content;
    const files = Array.isArray(m.files) ? m.files : [];
    if (m.role === 'user' && files.length) {
      const parts = files
        .filter((f) => f.text)
        .map((f) => `<file name="${f.name}"${f.truncated ? ' note="only the first part of this file is included"' : ''}>\n${f.text}\n</file>`);
      if (parts.length) content = `${parts.join('\n\n')}\n\n${content}`;
    }
    const t = estTokens(content);
    if (out.length && used + t > budget) break;
    out.unshift({ role: m.role, content: used + t > budget ? content.slice(0, Math.floor((budget - used) * 3.3)) : content });
    used += t;
  }
  return [{ role: 'system', content: sys }, ...out];
}

/**
 * Stream a chat completion. Calls onToken(textSoFar). Returns { text, stats }.
 */
export async function streamChat({ engine, messages, signal, onToken, maxTokens = 2048 }) {
  const res = await fetch(`http://127.0.0.1:${engine.port}/v1/chat/completions`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${engine.key}` },
    body: JSON.stringify({
      messages,
      stream: true,
      max_tokens: maxTokens,
      temperature: 0.6,
      top_p: 0.9,
      chat_template_kwargs: { enable_thinking: false },
      timings_per_token: false,
    }),
    signal,
  });
  if (!res.ok) {
    let msg = `The engine answered ${res.status}`;
    try { msg = (await res.json()).error?.message || msg; } catch {}
    throw new Error(msg);
  }
  const reader = res.body.getReader();
  const dec = new TextDecoder();
  let buf = '';
  let text = '';
  let stats = null;
  const t0 = performance.now();
  let tFirst = 0;
  let finish = '';
  for (;;) {
    const { value, done } = await reader.read();
    if (done) break;
    buf += dec.decode(value, { stream: true });
    let idx;
    while ((idx = buf.indexOf('\n')) >= 0) {
      const line = buf.slice(0, idx).trim();
      buf = buf.slice(idx + 1);
      if (!line.startsWith('data:')) continue;
      const data = line.slice(5).trim();
      if (data === '[DONE]') continue;
      let j;
      try { j = JSON.parse(data); } catch { continue; }
      const ch = j.choices?.[0];
      const piece = ch?.delta?.content || '';
      if (piece) {
        if (!tFirst) tFirst = performance.now();
        text += piece;
        onToken?.(text);
      }
      if (ch?.finish_reason) finish = ch.finish_reason;
      if (j.timings) {
        stats = {
          first_ms: Math.round((tFirst || performance.now()) - t0),
          tokens: j.timings.predicted_n,
          tps: Math.round((j.timings.predicted_per_second || 0) * 10) / 10,
          prompt_tokens: j.timings.prompt_n,
          finish,
        };
      }
    }
  }
  if (stats) stats.finish = finish;
  return { text, stats };
}

/** Ask the model for a short chat title from the first exchange. */
export async function makeTitle({ engine, userText, answer }) {
  try {
    const res = await fetch(`http://127.0.0.1:${engine.port}/v1/chat/completions`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${engine.key}` },
      body: JSON.stringify({
        messages: [
          { role: 'system', content: 'You write very short chat titles. Reply with the title only: 2 to 6 words, no quotes, no full stop, in the same language as the conversation.' },
          { role: 'user', content: `User: ${userText.slice(0, 1200)}\n\nAssistant: ${answer.slice(0, 800)}\n\nTitle:` },
        ],
        max_tokens: 20,
        temperature: 0.3,
        chat_template_kwargs: { enable_thinking: false },
      }),
    });
    const j = await res.json();
    let t = (j.choices?.[0]?.message?.content || '').split('\n')[0].replace(/^["'#*\s]+|["'.*\s]+$/g, '');
    if (t.toLowerCase().startsWith('title:')) t = t.slice(6).trim();
    return t.slice(0, 60);
  } catch {
    return '';
  }
}

/** Quick speed test: time to first word and words per second on this notebook. */
export async function speedTest(engine) {
  const messages = [
    { role: 'system', content: 'You are a helpful assistant.' },
    { role: 'user', content: 'Write three short sentences about why reading is good for you.' },
  ];
  const { stats } = await streamChat({ engine, messages, maxTokens: 96 });
  return stats;
}
