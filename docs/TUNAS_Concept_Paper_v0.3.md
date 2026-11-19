# TUNAS — Concept Paper v0.3

S.A.L.T CI · Owned by S.A.L.T. CI TECHNOLOGY · Distributed by Pemangkin Teknologi · 19 November 2026
Prepared by: CAPT ZEDs, Founder and Creator of S.A.L.T CI
Status: concept for review. Version 0.1 is being built. Based on the brief `claude/TERAS_to_NEW_Desktop_Lite_Brief_2026-11-19.md` and the decisions of 19 Nov 2026.

**Changes from v0.2:**
- **TUNAS is free** for everyone, with no licence key or activation (decision 14, section 13).
- **Local AI API:** other apps on the notebook, such as NEXUS desktop, can use TUNAS as their AI engine (decision 15, section 4.6).
- **NEXUS desktop:** the earlier desktop version is given out free alongside TUNAS (decision 16).

**Changes from v0.1:**
- Knowledge is designed properly (section 5).
- The open questions of v0.1 are settled as recommended (decisions 9–13).
- The phase order is updated (section 14).

---

## 1. What TUNAS is

TUNAS ("shoot", the first growth) is a small, light AI app that runs on a person's own notebook. It downloads open-source models and runs them on the notebook, fully offline. It lets the person build their own assistants with a name, a persona and their own knowledge. It makes Word, Excel and PDF files, and when they choose, connects to cloud providers or to their organisation's TERAS server. It can also be the AI engine for other apps on the same notebook.

It is one user on one device, with no enterprise features. TUNAS is the shoot; TERAS is the heartwood it grows into.

## 2. Why it exists

- **Bundles:** shipped with OEM products, and with PC and notebook leasing companies that rent units to government agencies.
- **Public use:** free for anyone, as a way to get to know S.A.L.T.
- **Marketing:** tasteful paths to the S.A.L.T CI pages for "more features", that is, TERAS.
- **The moment that sells it** (CAPT ZEDs' experience coaching university faculty and department heads): when they went home with an AI on their notebook carrying *their own* name and logo, it made a lasting impression. Co-branding is therefore a headline feature, not a setting.

## 3. Decisions so far (19 Nov 2026)

| # | Decision |
|---|---|
| 1 | Name: **TUNAS** |
| 2 | A separate, light app, not a cut-down TERAS. Windows first, Mac later. |
| 3 | **English first, Malay well supported.** The whole app (menus, settings, messages, installer) is in English. The chat adapts to the user's language. Models are chosen for a good balance of both. |
| 4 | Anyone can co-brand: OEM, reseller, organisation or department, individual. Layered, most specific shown first. |
| 5 | The person can create their own assistants: a name, a persona prompt, a base model (downloaded, or cloud if connected) and knowledge. This covers personal naming ("Pembantu AI Kak Mela"). |
| 6 | Whether users can change or add branding is an option set by the organisation. |
| 7 | Reference machine: 8 GB RAM, no GPU. CAPT ZEDs has one and can bring up a VM with that spec. |
| 8 | "Powered by S.A.L.T CI" stays visible and cannot be removed. |
| 9 | **Knowledge:** named knowledge sets built from the user's folders, attached to assistants inside the assistant panel. The TERAS idea, kept simple. (section 5) |
| 10 | **Own TUNAS mark** (a shoot), shown when no co-brand is set. Four options shown on 19 Nov; CAPT ZEDs to choose. |
| 11 | **Starter assistants:** General Assistant, Surat Rasmi, Official Letter, Minit Ringkas, Meeting Minutes, Email Writer and Summariser. |
| 12 | **Model downloads** come from a S.A.L.T download server with fixed, tested versions and checksums. The public model hub is the fallback. |
| 13 | **Branding packs can be signed.** Signed packs from S.A.L.T or partners show as "Verified". Home-made packs still import, but are labelled as such. |
| 14 | **TUNAS is free, with no licence.** No key, no activation, no expiry. People keep the installed app and the installer. It is a marketing tool: the revenue target is TERAS in the enterprise. |
| 15 | **Local AI API.** TUNAS can serve other apps on the same notebook as their AI engine, through an OpenAI-compatible chat-completions API. Off by default. |
| 16 | **NEXUS desktop:** the earlier desktop version of NEXUS is also given out free, and can use TUNAS as its AI engine. |

## 4. What a user gets (version 1)

### 4.1 Chat
- A clean chat with the chosen assistant; answers stream in as they are written.
- Chats are saved on the device, searchable, and titled by the model from their content.
- **Files dropped into a chat** (PDF, Word, Excel, PowerPoint, text) are read for that chat only.
- **Per-chat knowledge:** the user can also add a knowledge set to the current chat (section 5.4).

### 4.2 Models
- **Ready after install:** 2–3 small base models are installed during setup (section 10).
- **Check my notebook:** a screen that reads the RAM, processor and graphics, then recommends a model size and engine (CPU or integrated graphics). A speed test shows the real speed on this notebook.
- **Model catalogue:** a curated list of tested open models, each with its size and licence. Advanced users can also add a model file (GGUF) of their own.

### 4.3 My assistants
One panel holds everything that makes an assistant:

| Field | What it does |
|---|---|
| **Name** and **icon** | "Surat Rasmi", "Pembantu AI Kak Mela" |
| **Persona prompt** | How it behaves ("You write formal Malay government letters") |
| **Base model** | Any downloaded model, or a cloud or TERAS model when connected |
| **Knowledge** | One or more knowledge sets it answers from (section 5) |
| **Documents** | Which document types it may produce, and the letterhead it uses |

Users can copy, edit or delete the starter assistants (decision 11).

### 4.4 Documents
- The assistant can produce Word, Excel and PDF files from the chat: a letter, a memo, minutes, a table, a simple report.
- Files can carry the user's letterhead from their brand (section 6).
- The style follows TERAS's document maker, so a document made in TUNAS looks like one made in TERAS.

### 4.5 Providers
- A panel to connect cloud AI providers with the user's own key: OpenAI-compatible, plus the major providers by name.
- Cloud is **off until the user turns it on**, and the organisation can block it (section 6.6).
- **Connect to TERAS** is a built-in provider: the TERAS address and a key. Once connected, TUNAS lists the organisation's TERAS models and knowledge collections, and the user can use them in their assistants (sections 5.5 and 12).

### 4.6 Local AI API (for other apps)
- **What:** an OpenAI-compatible API on the notebook (`/v1/models`, `/v1/chat/completions` with streaming), so other apps can use TUNAS as their AI engine.
- **First user:** NEXUS desktop. A customer who buys S.A.L.T apps can run them on their own AI, on their own notebook.
- **Names:** apps see the user's assistants and models by name (for example `surat-rasmi` or `qwen3.5-4b`).
  - Asking for an assistant applies its persona and knowledge.
  - Asking for a model uses that model with no persona.
- **Safe by default:**
  - off until the user switches it on in Settings;
  - listens on this notebook only (127.0.0.1), on a fixed port shown in Settings;
  - needs a key that TUNAS generates; the user copies it into the other app and can renew it at any time;
  - a small list shows which apps have used it recently.
- **One engine, shared:** requests from apps and from the TUNAS chat take turns on the same loaded model, so the notebook never loads two models at once.
- **Organisation control:** the organisation policy can turn the API off (section 6.6).
- **Not a server for other computers:** reaching TUNAS from other machines on the network is out of scope. Sharing AI across people is what TERAS does.

## 5. Knowledge

### 5.1 The idea
It works the way TERAS does: knowledge is a thing of its own that you attach to an assistant. It is simplified for one person: no groups, no sharing, no permissions, no admin screens.

A **knowledge set** is a name plus one or more folders or files on the notebook, for example:
- "Pekeliling & SOP Jabatan": a folder of circulars;
- "My Lecture Notes": two folders;
- "Tender 2026": one folder.

### 5.2 Where the user meets it: the assistant panel
Most people never need a separate Knowledge screen. In the assistant panel, the Knowledge field offers:
- **+ New from folder:** pick a folder and give it a name. TUNAS starts indexing in the background. The assistant can be used straight away, and answers improve as indexing completes.
- **Add existing:** tick any knowledge set already made.

Because a knowledge set is separate from the assistant, the same folder is indexed **once** and shared by several assistants.

### 5.3 The Knowledge page (for tidying up)
A simple list of knowledge sets, showing for each:
- its folders, file count and size on disk;
- its status (ready, indexing with progress, paused, files it could not read);
- which assistants use it.

Actions: rename, add or remove a folder, re-index, pause, delete. Deleting a knowledge set removes only TUNAS's index, never the user's files.

### 5.4 In the chat
- **Add knowledge to this chat:** a small button (or typing `#`) picks a knowledge set for this chat only, without changing the assistant.
- **Dropped files:** read for that chat only, not indexed into a set. The chat offers "Save to a knowledge set" if the user wants to keep them.
- **Sources:** answers drawn from knowledge show their sources (file name and page), and a click opens the file.

### 5.5 TERAS knowledge (when connected)
- Once Connect to TERAS is on, the organisation's TERAS knowledge collections that the user is allowed to see appear in the same Knowledge field, marked "TERAS".
- They are searched on the TERAS server, not copied to the notebook. An assistant can mix local and TERAS knowledge.
- This is the clearest view of what TERAS adds: "my documents" become "my organisation's documents."

### 5.6 How it works underneath
- **File types (version 1):** PDF with text, Word, Excel, PowerPoint, plain text, Markdown and CSV. Scanned PDFs (OCR) are out of version 1; such files are listed under "could not read".
- **Index:** one local SQLite file with a vector extension, plus keyword search. Results are found by meaning and by exact words, which matters for reference numbers and names.
- **Embedding model:** a small multilingual model that runs on CPU, part of the starting set (section 10). It must work for English, Malay and mixed text.
- **Light on the notebook:**
  - indexing runs at low priority and pauses while the user is chatting;
  - it can wait for the charger;
  - big folders show an estimated time before starting.
- **Watching for changes:** new, changed or deleted files in a knowledge folder update the index automatically.
- **Privacy:** the index lives on the notebook only. Local knowledge is never sent to a cloud model unless the user's assistant uses a cloud base model, and the assistant panel says so clearly.
- **Organisation policy:** the organisation can block local knowledge from being used with cloud models at all.

## 6. Co-branding

### 6.1 Who can brand, in layers
1. **S.A.L.T CI**: always present as "Powered by S.A.L.T CI", following the brand rules.
2. **OEM or reseller**: from the installer settings file or the factory image.
3. **Organisation or department**: a faculty, unit or agency, usually from a branding pack.
4. **The person**: their own name or label.

The most specific layer is shown first. All layers stay credited on the About page. With no co-brand set, the TUNAS mark is shown.

### 6.2 What a brand contains
- A name ("AI Fakulti Kejuruteraan").
- A logo (PNG or SVG).
- Optionally an accent colour (or gradient) and a short welcome line.
- Optionally letterhead details for documents (organisation name, address line).

### 6.3 Where the brand shows
- The app name in the title bar.
- The welcome screen.
- The desktop and Start menu icon, and their names.
- The default assistant's name.
- The letterhead on documents the app makes.

The "Powered by S.A.L.T CI" credit sits small in the sidebar footer, the About page and the setup screens.

### 6.4 Easy by design
- A few clicks, no technical step: Name → Logo → Colour → Welcome line.
- A **live preview** of the window, the welcome screen, and the desktop and taskbar icons while choosing.
- Available during setup and later on the Branding page.
- The logo is checked and resized for the icon automatically. A clear warning appears if it is too small or too wide to read as an icon.

### 6.5 Branding pack
- **The pack:** one file (`.tunasbrand`) holds a brand and, optionally, assistants and an organisation policy.
- **Making one:** it is made in TUNAS itself (Branding → Make a pack).
- **At a workshop:** the trainer hands the file out on a USB stick or by link. Each person double-clicks it, and the whole faculty is branded in one step.
- **Signing:** packs signed by S.A.L.T or a partner show "Verified" (decision 13). The signing keys are held by S.A.L.T and never leave the signing machine.

### 6.6 Organisation control
The organisation chooses, in its pack or policy file, whether users may:
- change the organisation brand;
- add their own personal brand layer on top;
- create their own assistants (default: yes);
- turn on cloud providers;
- use local knowledge with cloud models;
- switch on the local AI API for other apps.

On agency-managed notebooks, the policy is installed machine-wide where normal users cannot edit it. It can be deployed through the agency's usual tools, such as Group Policy or Intune.

## 7. Language

- **Interface:** English throughout, including the installer, menus, settings, errors and help.
- **Chat:** the base instruction behind every assistant tells the model to reply in the user's language and to handle mixed Malay–English naturally, as people actually write.
- **Starter assistants and templates:** offered in both languages.
- **Model choice:** each bundled model, including the embedding model, must pass a balanced English and Malay test (section 10.3).
- **Malay interface:** not planned for version 1. The interface is built so it can be translated later if a channel partner asks.

## 8. Installer and deployment

### 8.1 Interactive install (public and workshops)
1. The installer copies the app for the current user, with no administrator rights needed.
2. TUNAS opens into setup: welcome, then **Check my notebook**.
3. **Models:** 2–3 pre-ticked by notebook size, each showing its size.
4. **Source:** the S.A.L.T download server (public hub as fallback), or **offline**: models from a folder on a USB stick or on the image.
5. **Branding (optional):** a name and logo with a live preview, or a branding pack.
6. Straight into a working chat. Downloads continue in the background.

### 8.2 Silent install (OEM, leasing and agency imaging)
One command with a settings file that presets:
- the brand layers;
- the models, and their offline source;
- the TERAS address;
- the organisation policy (section 6.6) and the update channel.

There are no screens and no questions. The result is identical on every unit.

### 8.3 Updates
- A quiet in-app check, with no pop-ups: a small dot on Settings.
- An **offline update file** for agencies that block the internet.
- Models update separately from the app.

## 9. Privacy and security

- Everything (chats, assistants, knowledge indexes) stays on the device by default.
- Cloud keys and the TERAS key are kept by Windows' own protection (DPAPI), tied to the user's account.
- Cloud is off until the user turns it on. The organisation can block it entirely.
- The local AI API is off by default, local to the notebook, and protected by a key.
- No telemetry unless the user opts in. Even then, only anonymous app health, never content.
- Works fully offline once installed.
- Downloaded and imported models are checked against their published checksums before use.

## 10. Hardware and models

### 10.1 Reality
Leased government notebooks are often 8–16 GB with integrated graphics only. A slow first answer loses the trial, so the defaults are small, quantised models.

| Notebook | Default |
|---|---|
| 8 GB, no GPU | A fast model of about 2B, plus a model of about 4B marked "better but slower" |
| 16 GB, integrated graphics | A model of about 4B as the default, on integrated graphics (Vulkan) where it helps |
| 16 GB+ with a dedicated GPU | Larger models offered in the catalogue |

### 10.2 The starting set (first catalogue, to be confirmed by testing)
| Model | Size (Q4) | Licence | In the bundled set |
|---|---|---|---|
| Qwen 3.5 2B | 1.3 GB | Apache-2.0 | Yes: the fast default for 8 GB |
| Qwen 3.5 4B | 2.7 GB | Apache-2.0 | Yes: better quality |
| IBM Granite 4.2 3B | 2.2 GB | Apache-2.0 | Yes: a second opinion |
| Qwen 3.5 0.8B | 0.5 GB | Apache-2.0 | Catalogue: for 4–6 GB notebooks |
| SEA-LION v4 4B (AI Singapore) | 2.5 GB | Listed MIT; built on a Gemma base, so Gemma terms may also apply | Catalogue only: the user downloads it from the public hub |
| Qwen 3 Embedding 0.6B | 0.6 GB | Apache-2.0 | For Knowledge (0.2) |

**Licence rule:** preloading or auto-downloading a model is redistribution, and so is hosting it on the S.A.L.T download server. Only models whose licence clearly allows it go in the bundled set. Others appear in the catalogue for the user to download themselves from the public hub, with their licence shown.

### 10.3 Test plan
- **Reference machine:** CAPT ZEDs' 8 GB, no-GPU notebook, plus a VM with the same spec.
- **The VM tests the CPU path only.** Integrated-graphics (Vulkan) speed must be measured on the real notebook.
- **Measures:**
  - time to the first word;
  - words per second;
  - RAM use while a browser and Office are open;
  - indexing time for a 500-file folder;
  - quality on a fixed test set: English and Malay questions, mixed-language chat, a formal Malay letter, an English email, summarising a PDF, and answering from a knowledge set in both languages.
- **Pass mark for the default model:** first word within a few seconds, and readable speed on the 8 GB machine.

## 11. Technology

| Part | Choice | Why |
|---|---|---|
| App shell | **Tauri** (Rust core, web front end) | Installer and memory far smaller than Electron, which leaves more RAM for the model on an 8 GB notebook |
| AI engine | **llama.cpp**, bundled as a separate process, in CPU and Vulkan builds | Proven, fast on CPU, uses integrated graphics through Vulkan; the app picks the build and falls back to the CPU |
| Model format | GGUF, quantised | The standard for llama.cpp; small and fast |
| Storage and knowledge | SQLite with a vector extension and full-text search | One local file for chats, assistants and knowledge indexes; no database server |
| Documents | A small local document maker in the app, following TERAS's styles and templates | TUNAS cannot depend on the TERAS server or carry Python; if TERAS's maker can be shared without Python, reuse it |
| Local AI API | A small OpenAI-compatible gateway in the TUNAS core, in front of the engine | One engine shared by the chat and other apps; keys and assistant names handled in one place |
| Installer | A standard Windows installer, per user, with silent mode and a settings file | What agency IT expects; no administrator rights for public users |
| Downloads | S.A.L.T download server behind a CDN, with checksums; public hub fallback | Fixed, tested versions; not tied to any one S.A.L.T machine |
| Not included | Docker, Postgres, Python, background services | Light, and nothing for agency IT to object to |

The installer itself stays small. The models are the real size (about 1–3 GB each), downloaded or read from the offline source.

## 12. The line against TERAS, and the path to it

| TUNAS | TERAS (what "more features" sells) |
|---|---|
| One user, one device | The whole organisation |
| Local models plus the user's own cloud keys | Shared models, local and cloud chosen per task and secrecy level |
| Personal knowledge sets from local folders | Knowledge collections for teams and the whole organisation, with permissions |
| Personal assistants | Model Studio, fine-tuning, personalisation and memory |
| Simple documents | Templates at every level, the document sandbox |
| A local AI API for apps on the same notebook | AI for every app and every user across the organisation |
| — | Sign-in and SSO, Musyawarah meetings, MIMPI, admin, audit, federation, S.A.L.T apps |

**Respectful marketing:**
- An About page with "What TERAS adds".
- A "Connect to TERAS" card in Providers.
- A gentle note on the Knowledge page: "Share knowledge with your whole team with TERAS."
- No banners or pop-ups, or agency IT will strip the app from the image.

**Connect to TERAS** turns a TUNAS notebook into a window on the organisation's TERAS. The user sees the organisation's models and knowledge by name, alongside their own local assistants and knowledge sets. This matches the USM rollout strategy, where people start with S.A.L.T on their own notebooks and grow into department and central servers.

## 13. Licensing of the app

- **Free for everyone, with no licence key, activation or expiry.** People keep the installed app and the installer, and may pass the installer on.
- **Why:** TUNAS is marketing. It costs S.A.L.T nothing per copy, and every copy carries "Powered by S.A.L.T CI" and the path to TERAS, where the enterprise revenue is.
- **NEXUS desktop** (the earlier desktop version) is given out free on the same terms, and works with TUNAS through the local AI API.
- **OEM, leasing and reseller arrangements** (pre-installation, co-branded images, workshops, support) go through Pemangkin Teknologi. Prices are not part of design documents.

## 14. Phases

| Release | Contents |
|---|---|
| **0.1** | Windows installer (per user); llama.cpp in CPU and Vulkan builds; Check my notebook with speed test; model catalogue and downloads with checksums; offline import from a folder or USB stick, and copying models to one; chat with dropped files; My assistants (name, icon, persona, base model); co-branding with live preview, icon and shortcut renaming; branding pack import and export (with optional rules); TUNAS mark; About page with brand layers |
| **0.2** | Knowledge: knowledge sets in the assistant panel, the Knowledge page, per-chat knowledge and sources. Documents: Word, Excel and PDF with letterhead. |
| **0.3** | Providers panel and Connect to TERAS (TERAS models and knowledge); **local AI API for other apps (NEXUS desktop first)**; silent install with settings file; machine-wide organisation policy and locks; signed branding packs; in-app update check and offline update file |
| **0.4** | Polish from trials; Mac version; knowledge pack and product brief |

Knowledge moves ahead of Connect to TERAS. The public trial and workshops get their biggest "wow" from their own documents and their own brand, and neither needs a server. Model and embedding testing (section 10.3) runs alongside 0.1, so the bundled set is settled before the first public build.

## 15. Open questions

1. **TUNAS mark:** CAPT ZEDs to choose from options A–D (shown 19 Nov).
2. **Download server address:** for example `models.salt.ci`, and where it is hosted. Until it exists, downloads come from the public hub automatically.
3. **A first channel partner:** if an OEM or leasing partner is already lined up, silent install and policy may need to move into 0.2.
4. **NEXUS desktop:** which version is given out, and whether it needs a small update to choose TUNAS as its AI engine (the API address and key).

---
S.A.L.T CI · Owned by S.A.L.T. CI TECHNOLOGY · Distributed by Pemangkin Teknologi · www.salt.ci · info@salt.ci · www.pemangkin.tech · info@pemangkin.tech
S.A.L.T CI and its components are proprietary technology of S.A.L.T. CI TECHNOLOGY, distributed and commercialised by Pemangkin Teknologi.
