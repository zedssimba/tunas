# S.A.L.T TUNAS: Administrator Guide

TUNAS 0.5.2 · November 2026 · https://www.salt.ci
S.A.L.T CI · Owned by S.A.L.T. CI TECHNOLOGY · Distributed by Pemangkin Teknologi

This is the knowledge copy of the TUNAS Administrator Guide for AI knowledge bases (TUNAS 0.5.2). What TUNAS is, and what is not available yet, is in the TUNAS Product Brief (knowledge version). Day-to-day use is in the TUNAS User Guide.

This guide is for IT departments, trainers, resellers and OEM partners who install TUNAS on many computers, brand it, set the organisation's rules, and look after it.

## 1. What gets installed where

TUNAS installs per user, with no administrator rights.

| What | Where |
|---|---|
| The app, its AI engines and fonts | `%LOCALAPPDATA%\Programs\TUNAS` |
| Chats, assistants and settings | `%LOCALAPPDATA%\TUNAS\tunas.db` |
| Knowledge indexes | `%LOCALAPPDATA%\TUNAS\knowledge.db` |
| Models | `%LOCALAPPDATA%\TUNAS\models` |
| Logs | `%LOCALAPPDATA%\TUNAS\logs` |
| Personal and per-user organisation brand | `%LOCALAPPDATA%\TUNAS\brand\personal` and `...\brand\org` |
| Machine-wide organisation and OEM brand | `%ProgramData%\TUNAS\brand\org` and `...\brand\oem` |
| Machine-wide policy | `%ProgramData%\TUNAS\policy.json` |

- The installer registers TUNAS under the current user, creates Desktop and Start menu shortcuts, and makes `.tunasbrand` files open with TUNAS.
- The AI engines are kept in versioned folders inside the app (`engine\<version>\cpu` and `engine\<version>\vulkan`), so an update never fights a running older engine.
- **Updating:** run the new installer for each user. All data in `%LOCALAPPDATA%\TUNAS` is kept.
- **Uninstalling** asks whether to delete chats, assistants, branding and models too.

### On macOS (from 0.5.0)

TUNAS for Mac comes as a `.dmg` per kind of Mac: `mac_apple-silicon` (engine on Metal graphics) and `mac_intel` (engine on the processor). Users drag TUNAS into Applications.

| What | Where |
|---|---|
| The app, its AI engine and model list | `/Applications/TUNAS.app` |
| Chats, assistants, settings, knowledge, models, logs, personal brand | `~/Library/Application Support/TUNAS` (same layout as on Windows) |
| Machine-wide organisation and OEM brand | `/Library/Application Support/TUNAS/brand/org` and `.../brand/oem` |
| Machine-wide policy | `/Library/Application Support/TUNAS/policy.json` |
| TERAS and cloud keys | The user's macOS Keychain, service name "TUNAS" |

- **Not yet signed by Apple.** Until the packages are signed and notarised, remove the download quarantine once per Mac: `xattr -dr com.apple.quarantine /Applications/TUNAS.app` (or as part of your deployment script).
- Writing to `/Library/Application Support` needs an administrator, so users can't change the machine-wide brand or policy.
- `.tunasbrand` files open with TUNAS by double-click.
- On a Mac the Dock and Finder icon stays the TUNAS mark (`logo_as_icon` is a Windows option); the brand's logo shows inside TUNAS and on documents.
- When TUNAS starts it stops any engine left running from an earlier session.
- The 0.5.x packages are the first Mac builds; tell S.A.L.T CI about anything that behaves differently from Windows.

## 2. Requirements

| Item | Details |
|---|---|
| Windows | Windows 10 or 11, 64-bit |
| macOS | macOS 11 (Big Sur) or later; Apple silicon (M1 or later) or Intel (2013 or later, with AVX2) |
| Memory | 8 GB recommended; 4 GB for Qwen 3.5 0.8B only |
| Graphics | Optional. On Windows, any card with a working Vulkan driver can be used; TUNAS tests it before use and falls back to the processor |
| Disk | About 1 to 3 GB per model, plus about 0.6 GB for the Knowledge reading model |
| Network | Only to download models, or for TERAS and cloud providers. Not needed otherwise |
| Antivirus | The installer is not yet code-signed; some antivirus tools scan it first and may launch it twice. Allow-list it for mass rollouts |

## 3. Models without the internet

For classrooms, agencies and images with no internet:

1. On one computer with TUNAS and the models installed, open **Settings › Models › Prepare a USB stick** (Copy models to…) and choose the USB stick or a shared folder.
2. On each target computer, in setup choose **From a folder or USB stick**, or later **Settings › Models › Import from a folder or USB stick**.
3. TUNAS checks every file against the catalogue's checksum before copying it in. Files not in the catalogue are skipped (they can be added as **Your own models**).

The Knowledge reading model (Qwen 3 Embedding 0.6B) can be included on the stick so knowledge sets work offline from the start.

## 4. Branding for an organisation

TUNAS has three brand layers, shown most specific first: **personal**, **organisation**, **OEM**. "Powered by S.A.L.T CI" is shown by the interface itself and is not part of any layer.

### A brand

Each layer is a folder with `brand.json` and a logo file:

| Field | Meaning |
|---|---|
| `name` | The name shown in the title bar, side bar, shortcuts and About |
| `assistant_name` | Optional name for the General Assistant |
| `welcome` | The welcome line on the welcome screen |
| `color1`, `color2` | The brand colours (hex) |
| `logo_file` | The logo's file name in the same folder (PNG or SVG) |
| `logo_bg`, `logo_shape` | Logo background ('' transparent, `light`, `dark`) and shape (`square` or `circle`) |
| `logo_as_icon` | `true` to also use the logo as the taskbar, desktop and Start menu icon; off by default |
| `letterhead_org`, `letterhead_address` | The organisation name and address on saved Word and PDF documents |
| `issuer` | Who made the brand, shown to users |

The easiest way to make one is in TUNAS itself: **Settings › Branding**, fill in the brand with its live preview, then **Branding packs › Make a pack…**.

### Branding packs

A `.tunasbrand` file is a zip with `brand.json`, the logo, and optionally `assistants.json` and `policy.json`.

- **Make a pack:** Settings › Branding › Branding packs › **Make a pack…**. Choose the layer, the assistants to include (each carries its name, icon, colour and persona), the **Made by** name, and optionally **Include organisation rules**:
  - People may change the organisation brand;
  - People may add their own brand on top;
  - People may create their own assistants.
- **Install a pack:** double-click the file, or **Import branding pack** in Settings › Branding, or **I have a branding pack** in setup. The pack becomes the user's organisation layer; its assistants are added (replacing earlier copies with the same names); its rules apply.
- A pack can't be imported over a machine-wide organisation brand, or when the current rules don't allow the organisation brand to be changed.

### Machine-wide brand (IT)

For a whole computer or a disk image, copy the brand folder (`brand.json` and the logo) to:
- `%ProgramData%\TUNAS\brand\org`: the organisation brand for every user of the computer; users can't change or replace it;
- `%ProgramData%\TUNAS\brand\oem`: the OEM or reseller brand, for notebook makers, leasing companies and resellers; it can't be changed or removed in the app.

Writing to `%ProgramData%` needs administrator rights, so normal users can't alter these. The easiest source of `brand.json` and the logo is a pack made in TUNAS: unzip it.

### Icons and shortcuts

- With a brand set, TUNAS renames its Desktop and Start menu shortcuts to the brand's name.
- With `logo_as_icon` on, TUNAS draws an icon from the logo and uses it for the window, taskbar and shortcuts; otherwise the TUNAS mark stays.
- Windows keeps its own icon cache. If a pinned taskbar icon still shows an older icon, unpin and pin TUNAS again, or clear the icon cache.

## 5. Organisation policy

The policy decides what users may change and use. TUNAS reads, in order:
1. `%ProgramData%\TUNAS\policy.json` (machine policy, set by IT; wins over everything);
2. the policy from an imported branding pack;
3. otherwise, everything is allowed.

```json
{
  "allow_change_org_brand": false,
  "allow_personal_brand": true,
  "allow_create_assistants": true,
  "allow_cloud": false,
  "allow_knowledge_with_cloud": false,
  "allow_local_api": true
}
```

| Setting | When false |
|---|---|
| `allow_change_org_brand` | Users can't edit or replace the organisation brand |
| `allow_personal_brand` | Users can't add their own brand on top |
| `allow_create_assistants` | Users can't make new assistants |
| `allow_cloud` | Cloud AI providers can't be added or used; the Connections page says the organisation does not allow them. TERAS stays available |
| `allow_knowledge_with_cloud` | Reserved for keeping knowledge passages away from cloud models |
| `allow_local_api` | Other apps can't use TUNAS; the local AI API stays off |

Settings left out of the file count as allowed. The policy's source (machine, pack or default) is known to the app, and the Branding and Connections pages show what is locked.

## 6. TERAS and cloud providers

- **TERAS:** give users the organisation's TERAS address and a TERAS key for TUNAS. In **Settings › Connections › Connect to TERAS**, TUNAS lists the TERAS models by name; users pick them as an assistant's base model or per chat. Questions sent to TERAS are answered on the organisation's own server.
- **Cloud providers:** off until the user switches on **Allow cloud providers**. The main providers are listed by name, and any OpenAI-compatible address can be added. Users bring their own keys and choose which models their assistants may use. Block with `allow_cloud: false`.
- **Keys:** TERAS and cloud keys are kept encrypted with Windows' DPAPI, tied to the Windows user (on a Mac, in the user's Keychain). They don't travel in branding packs and can't be read on another account or computer.
- **TERAS tools and files:** TUNAS asks TERAS for its tool and source events, shows the person what TERAS is doing, and turns files TERAS makes into download cards (from TERAS's `file` event, or the `/api/files/…` links in older answers). The download always goes to the connected TERAS address. The download uses the TERAS key; if TERAS refuses the key, the file opens in the browser, where the person signs in.
- **Documents with TERAS models (from 0.5.1):** when the person asks a TERAS model for a document, TUNAS tells the model to make it with TERAS's `create_document` tool, so it gets TERAS's formats (Word, PDF, PowerPoint, Excel, CSV, Markdown), document templates and branding. With local and cloud models TUNAS makes the file itself. **Rich document** (from 0.5.2, needs TERAS 0.25.6): the person's button beside the message box sends `"teras": {"rich_document": true}`, and TERAS uses its document sandbox for that request if the person may use it. TUNAS sends no answer-length limit to TERAS or cloud models, so long documents aren't cut short. TERAS's audit log shows these calls under the key the person made for TUNAS.
- **About me and TERAS:** the person's About me (Settings) is not sent to TERAS models, which use the person's own TERAS personalisation.
- **Knowledge with remote models:** when an assistant on TERAS or a cloud model has knowledge sets, the question and the matching passages from the user's documents are sent with the question. Use local models for documents that must stay on the computer.

## 7. The local AI API

TUNAS can be the AI engine for other apps on the same computer, such as S.A.L.T desktop apps.

- **Turn on:** Settings › Connections › **Let other apps use TUNAS**. The port is 11435 by default and can be changed there.
- **Address:** `http://127.0.0.1:<port>/v1`, reachable from the same computer only.
- **Keys:** **Make key** for each app; the key is shown once and stored only as a hash. Each key shows when it was last used and how often. Remove a key to cut that app off.
- **Endpoints:** `GET /v1/models`, `POST /v1/chat/completions` (with streaming), and `GET /health`.
- **Model names:**
  - an assistant's short name, for example `surat-rasmi`: its persona and knowledge sets apply;
  - a local model, for example `qwen3.5-4b`;
  - a connected provider's model, for example `teras/<model-name>`.
- **One engine:** API requests and the TUNAS chat share the same loaded model and take turns.
- **Block** with `allow_local_api: false`.

## 8. About me

- **Stored** in `tunas.db` (the `about_me` setting), on the computer only, per Windows user.
- **Sent** as a short note with each message to local and cloud models, and with requests from other apps through the local AI API that use local or cloud models. Never sent to TERAS models.
- **Letterhead:** documents use the Branding letterhead organisation and address as the sender. Set them in a branding pack so every computer writes from the organisation.

## 9. Knowledge on managed computers

- **Permissions are per user:** TUNAS reads only folders the user allowed, never writes to them, and stops reading a folder when its permission is removed.
- **Index:** kept in `%LOCALAPPDATA%\TUNAS\knowledge.db`, separate from chats. It holds extracted text and search vectors, so treat it as as sensitive as the documents.
- **Reading model:** Qwen 3 Embedding 0.6B, downloaded the first time a knowledge set is made, or imported from a USB stick.
- **Performance:** the reading engine runs at below-normal priority on all processor cores, or on the graphics card when the main engine runs there, and waits while a chat is answering. On processor-only notebooks, large PDF folders take a while the first time; later only new and changed files are read.
- **Network drives and cloud folders:** work as long as the files can be opened. Files kept only online are downloaded by the drive's own app when read.
- **Refreshing:** each time TUNAS starts, and on the set's refresh button.

## 10. Engines and graphics

- TUNAS ships two builds of the llama.cpp engine: processor and Vulkan graphics. On Apple silicon Macs the graphics build uses Metal; Intel Macs have the processor build only.
- **Run AI on** (Settings › General): Automatic, Graphics or Processor. Automatic tests the graphics card once, remembers the result, and uses the processor if the card fails or has too little memory.
- On some computers with built-in graphics, the processor is faster. Compare with **Test speed** in Settings › Models.
- Changing Run AI on restarts the engine with the new choice.

## 11. Logs and troubleshooting

Logs are in `%LOCALAPPDATA%\TUNAS\logs`:

| Log | What is in it |
|---|---|
| `engine-cpu.log`, `engine-vulkan.log` | The chat engine on the processor or the graphics card: loading, devices found, errors |
| `engine-search.log` | The Knowledge reading engine |
| `knowledge.log` | Knowledge reading and each search: the scores, word matches and whether the documents were used |
| `teras.log` | The tool and source events TERAS sends to TUNAS's chat |

**PowerShell checks** (run in a normal PowerShell window):

```powershell
cd C:\
Get-ChildItem "$env:LOCALAPPDATA\TUNAS\logs"
Get-Content "$env:LOCALAPPDATA\TUNAS\logs\knowledge.log" -Tail 30
Get-Process llama-server -ErrorAction SilentlyContinue | Select-Object Id, StartTime, Path
```

| Problem | What to do |
|---|---|
| The installer runs twice | Antivirus relaunches it after scanning. Close the second window, or allow-list the installer |
| "Network error" when stopping or switching engines | Update to TUNAS 0.2.4 or later |
| Old engine processes still running after an update | `Stop-Process -Name llama-server` (with TUNAS closed), or restart Windows |
| Graphics not used | Check `engine-vulkan.log`; update the graphics driver; set Run AI on to Graphics and test |
| Thinking never ends | Set the model's thinking length to short, or use On, hidden; update to 0.2.5 or later |
| Knowledge reading is slow | Check `knowledge.log`; on a graphics card set Run AI on to Graphics; avoid huge scanned PDFs |
| Assistant ignores the documents | Check the set is Ready and is in the assistant's Knowledge field; update to 0.3.1 or later |
| Taskbar shows an old icon | Unpin and pin again, or clear the Windows icon cache |

## 12. Rolling out to a class or a department

1. Prepare one computer: install TUNAS, the models (and the reading model), and the brand.
2. **Make a pack…** with the organisation's brand, its assistants and its rules.
3. **Prepare a USB stick** with the models.
4. On each computer: run the installer, choose **From a folder or USB stick** in setup, then **I have a branding pack**.
5. For locked-down computers, also place `brand\org` and `policy.json` under `%ProgramData%\TUNAS`.

Silent installation with a settings file is on the roadmap; today each install runs the normal per-user installer.

## 13. Questions and answers

**Can users install TUNAS without administrator rights?**
Yes. It installs per user. Only the machine-wide brand and policy under `%ProgramData%\TUNAS` need administrator rights.

**Bolehkah pengguna memasang TUNAS tanpa hak pentadbir?**
Boleh. Ia dipasang untuk setiap pengguna. Hanya jenama dan polisi seluruh mesin di bawah `%ProgramData%\TUNAS` memerlukan hak pentadbir.

**How do we stop staff using cloud AI?**
Place a `policy.json` with `"allow_cloud": false` in `%ProgramData%\TUNAS`. TERAS remains available.

**Bagaimana menghalang kakitangan menggunakan AI awan?**
Letakkan `policy.json` dengan `"allow_cloud": false` di `%ProgramData%\TUNAS`. TERAS masih boleh digunakan.

**Can other computers on the network use one TUNAS?**
No. The local AI API listens on the computer itself only. For shared AI, use TERAS.

**Does TUNAS report anything back?**
No. TUNAS sends no content and no usage data. It only contacts the model download server when downloading, and TERAS or cloud providers when the user uses them.

**Where do we get OEM or co-branded images?**
Through Pemangkin Teknologi, the authorised distributor of S.A.L.T CI.

---
S.A.L.T CI · Owned by S.A.L.T. CI TECHNOLOGY · Distributed by Pemangkin Teknologi · www.salt.ci · info@salt.ci · www.pemangkin.tech · info@pemangkin.tech
S.A.L.T CI and its components are proprietary technology of S.A.L.T. CI TECHNOLOGY, distributed and commercialised by Pemangkin Teknologi.
