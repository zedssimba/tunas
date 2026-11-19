<script>
  // The TUNAS mark (shown when no co-brand is set). Placeholder: option A until CAPT ZEDs chooses.
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
    <path d="M32 53 C32 44 32 36 32 28" />
    <path d="M32 37 C24 37 18 32 17 24 C25 24 31 29 32 37 Z" />
    <path d="M32 28 C34 19 41 13 50 13 C49 22 42 28 32 28 Z" />
  </g>
</svg>
