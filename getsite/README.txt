S.A.L.T CI downloads (get.saltci.app) - TUNAS installers and update file
S.A.L.T CI · Owned by S.A.L.T. CI TECHNOLOGY · Distributed by Pemangkin Teknologi

What this is
  A plain web server (nginx in Docker, port 2065 on .76) holding the TUNAS installers for Windows,
  Mac and Android, and tunas/latest.json, which TUNAS reads once a day to offer "Update".

Public address
  get.saltci.app -> Vultr VPS -> Nginx Proxy Manager on disti (Let's Encrypt) -> 10.10.10.76:2065

Each new TUNAS release
  The release comes as a new salt-get zip. Expand it over E:\salt-get and run scripts\upgrade.ps1.
  Only the newest version is kept. latest.json carries each file's SHA-256: TUNAS refuses a
  download that does not match.

Check
  https://get.saltci.app/              the download page
  https://get.saltci.app/tunas/latest.json
