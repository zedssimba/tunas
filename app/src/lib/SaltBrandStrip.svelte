<script>
  // Standard S.A.L.T brand strip: gold S.A.L.T logo (largest), then the owner and distributor logos.
  import saltGold from '../assets/salt-gold-3d.png';
  import saltTech from '../assets/salt-ci-tech-on-dark.png';
  import pemangkin from '../assets/pemangkin-silver.png';
  import { openUrl } from './api.js';
  let { compact = false } = $props();
</script>

<div class="strip" class:compact>
  <img class="gold" src={saltGold} alt="S.A.L.T CI" />
  <div class="right">
    <div class="pw">Powered by <b>S.A.L.T CI</b></div>
    <div class="cos">
      <img src={saltTech} alt="S.A.L.T. CI TECHNOLOGY" />
      <img src={pemangkin} alt="Pemangkin Teknologi" />
    </div>
    <div class="links">
      <button onclick={() => openUrl('https://www.salt.ci')}>www.salt.ci</button>
      <span>·</span>
      <button onclick={() => openUrl('https://www.pemangkin.tech')}>www.pemangkin.tech</button>
    </div>
  </div>
</div>

<style>
  .strip {
    display: flex; align-items: center; gap: 22px; padding: 18px 24px;
    background: radial-gradient(120% 140% at 0% 0%, #1D2A5E, #0B1022 70%); color: #B9C3E0;
  }
  .gold { height: 92px; width: auto; flex: none; }
  .right { display: flex; flex-direction: column; gap: 10px; min-width: 0; }
  .pw { font-size: 12px; letter-spacing: .08em; text-transform: uppercase; color: #E3C254; font-weight: 700; }
  .pw b { font-weight: 800; }
  .cos { display: flex; align-items: center; gap: 22px; flex-wrap: wrap; }
  .cos img { height: 26px; width: auto; }
  .links { display: flex; gap: 8px; font-size: 12px; color: #8E98B8; }
  .links button { color: #8E98B8; padding: 0; font-size: 12px; }
  .links button:hover { color: #fff; text-decoration: underline; }
  .compact { padding: 12px 22px; gap: 16px; }
  .compact .gold { height: 58px; }
  .compact .cos img { height: 20px; }
  .compact .right { gap: 6px; }
</style>
