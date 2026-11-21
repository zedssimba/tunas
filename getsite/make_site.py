"""Build the get.saltci.app release package for one TUNAS version.

python3 make_site.py 0.6.0 --windows TUNAS_Setup_0.6.0.exe --mac-intel ..._intel.dmg --mac-arm ..._apple-silicon.dmg \
    --android TUNAS_0.6.0_android.apk --notes TUNAS_Release_Notes.md --out /tmp/salt-get

Writes <out>/salt-get/ (docker-compose.yml, nginx.conf, scripts/, site/) with site/tunas/<version>/ holding the
files, and site/tunas/latest.json naming them with their size and SHA-256. Only the newest version is kept on
the site, so the package is also what goes on the server.
"""
import argparse, datetime, hashlib, html, json, os, shutil

HERE = os.path.dirname(os.path.abspath(__file__))


def sha256(p):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


def mb(n):
    return f"{n / 1024 / 1024:.0f} MB"


def main():
    a = argparse.ArgumentParser()
    a.add_argument("version")
    a.add_argument("--windows")
    a.add_argument("--mac-intel")
    a.add_argument("--mac-arm")
    a.add_argument("--android")
    a.add_argument("--notes", required=True)
    a.add_argument("--extra", action="append", default=[], help="more files to put beside the installers (READMEs)")
    a.add_argument("--out", required=True)
    o = a.parse_args()

    root = os.path.join(o.out, "salt-get")
    shutil.rmtree(root, ignore_errors=True)
    os.makedirs(os.path.join(root, "scripts"))
    for f in ("docker-compose.yml", "nginx.conf", "README.txt"):
        shutil.copy(os.path.join(HERE, f), root)
    shutil.copy(os.path.join(HERE, "scripts", "upgrade.ps1"), os.path.join(root, "scripts"))
    site = os.path.join(root, "site")
    vdir = os.path.join(site, "tunas", o.version)
    os.makedirs(vdir)
    for f in ("index.html",):
        shutil.copy(os.path.join(HERE, "site", f), site)
    for f in ("salt.png", "tunas-mark.svg"):
        p = os.path.join(HERE, "site", f)
        if os.path.exists(p):
            shutil.copy(p, site)

    files = {}
    for key, path in (("windows", o.windows), ("mac_intel", o.mac_intel), ("mac_apple_silicon", o.mac_arm), ("android", o.android)):
        if not path:
            continue
        name = os.path.basename(path)
        shutil.copy(path, os.path.join(vdir, name))
        files[key] = {"name": name, "url": f"{o.version}/{name}", "sha256": sha256(path), "size": os.path.getsize(path)}
    for p in o.extra:
        shutil.copy(p, vdir)
    shutil.copy(o.notes, os.path.join(site, "tunas", "TUNAS_Release_Notes.md"))

    latest = {
        "product": "TUNAS",
        "version": o.version,
        "date": datetime.date.today().isoformat(),
        "notes": "TUNAS_Release_Notes.md",
        "files": files,
    }
    with open(os.path.join(site, "tunas", "latest.json"), "w") as f:
        json.dump(latest, f, indent=2)
    print(json.dumps(latest, indent=2))


if __name__ == "__main__":
    main()
