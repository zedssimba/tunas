// Talking to the local engine (llama.cpp server, OpenAI-compatible) with streaming.
import { app } from './state.svelte.js';

/** Rough token estimate (good enough for budgeting). */
export function estTokens(s) {
  return Math.ceil((s || '').length / 3.3);
}

/** Turn stored messages into the API format, attaching file text, within the context budget. */
export function buildMessages(assistant, history, ctx) {
  // The TUNAS gateway adds the system prompt (persona, language rule, built-in S.A.L.T reference).
  const budget = Math.max(1024, ctx - 2400); // leave room for the system prompt and the answer
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
  return out;
}

/**
 * Stream a chat completion. Calls onToken(textSoFar). Returns { text, stats }.
 */
export async function streamChat({ messages, signal, onToken, onThinking, maxTokens = 2048, assistant = null, model = '', raw = false, think = false }) {
  const gw = app.gateway;
  const res = await fetch(`http://127.0.0.1:${gw.port}/v1/chat/completions`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${gw.key}` },
    body: JSON.stringify({
      model,
      tunas_assistant: assistant?.id,
      tunas_raw: raw,
      messages,
      stream: true,
      max_tokens: think ? undefined : maxTokens,
      temperature: 0.6,
      top_p: 0.9,
      chat_template_kwargs: { enable_thinking: !!think },
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
  let reasoning = '';
  let tThinkStart = 0;
  let tThinkEnd = 0;
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
      const rpiece = ch?.delta?.reasoning_content || '';
      if (rpiece) {
        if (!tThinkStart) tThinkStart = performance.now();
        reasoning += rpiece;
        onThinking?.(reasoning);
      }
      const piece = ch?.delta?.content || '';
      if (piece) {
        if (tThinkStart && !tThinkEnd) tThinkEnd = performance.now();
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
  const think_ms = tThinkStart ? Math.round((tThinkEnd || performance.now()) - tThinkStart) : 0;
  if (stats) { stats.finish = finish; if (think_ms) stats.think_ms = think_ms; }
  else if (text) {
    // Cloud providers send no timings: estimate from the text.
    const secs = Math.max(0.001, (performance.now() - (tFirst || t0)) / 1000);
    stats = { first_ms: Math.round((tFirst || performance.now()) - t0), tps: Math.round((text.length / 4 / secs) * 10) / 10, est: true, finish };
    if (think_ms) stats.think_ms = think_ms;
  }
  return { text, reasoning, stats };
}

/** Ask the model for a short chat title from the first exchange. */
export async function makeTitle({ model, userText, answer }) {
  try {
    const gw = app.gateway;
    const res = await fetch(`http://127.0.0.1:${gw.port}/v1/chat/completions`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${gw.key}` },
      body: JSON.stringify({
        model,
        tunas_raw: true,
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

/** Quick speed test on a local model: time to first word and tokens per second on this machine. */
export async function speedTest(model) {
  const messages = [
    { role: 'system', content: 'You are a helpful assistant.' },
    { role: 'user', content: 'Write three short sentences about why reading is good for you.' },
  ];
  const { stats } = await streamChat({ messages, maxTokens: 96, model, raw: true });
  return stats;
}
