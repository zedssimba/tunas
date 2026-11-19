<script>
  // The TUNAS mark (shown when no co-brand is set): option B, a shoot growing from a gold seed.
  let { size = 32 } = $props();
  const id = 'tg' + Math.random().toString(36).slice(2, 8);
</script>

<svg width={size} height={size} viewBox="0 0 64 64" aria-label="TUNAS">
  <defs>
    <linearGradient id={id} x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="#2F74D8" /><stop offset=".55" stop-color="#6D4FE0" /><stop offset="1" stop-color="#E3C254" />
    </linearGradient>
  </defs>
  <rect width="64" height="64" rx="15" fill="url(#{id})" />
  <g fill="none" stroke="#fff" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
    <path d="M32 44 C32 35 33 29 37 23" />
    <path d="M37 23 C39 16 45 12 52 12 C51 19 45 24 37 23 Z" />
    <path d="M33 33 C27 32 22 28 21 22 C27 22 32 26 33 33 Z" />
  </g>
  <circle cx="32" cy="47.4" r="5.1" fill="#F5D76E" stroke="#fff" stroke-width="2.2" />
</svg>
