import { marked } from 'marked';
import DOMPurify from 'dompurify';

marked.setOptions({ gfm: true, breaks: false });

export function md(text) {
  const html = marked.parse(text || '');
  const clean = DOMPurify.sanitize(html, { USE_PROFILES: { html: true } });
  // Wrap code blocks so we can add a copy button, and make links open outside.
  return clean
    .replace(/<pre><code/g, '<pre class="code"><button class="copy-code" data-copy>Copy</button><code')
    .replace(/<a /g, '<a target="_blank" rel="noopener" ');
}
