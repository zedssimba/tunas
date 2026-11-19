<script>
  // "Powered by S.A.L.T CI" with the gold S.A.L.T logo. Always shown; not part of any co-brand layer.
  import saltGold from '../assets/salt-gold-3d.png';
  let { size = 11, logo = 18 } = $props();
</script>

<span class="pw" style="font-size:{size}px">
  <img src={saltGold} alt="S.A.L.T CI" style="height:{logo}px" />
  <span>Powered by <b>S.A.L.T CI</b></span>
</span>

<style>
  .pw { display: inline-flex; align-items: center; gap: 6px; color: var(--faint); letter-spacing: .02em; }
  img { width: auto; flex: none; }
  b { font-weight: 700; background: var(--salt-grad); -webkit-background-clip: text; background-clip: text; color: transparent; }
</style>
