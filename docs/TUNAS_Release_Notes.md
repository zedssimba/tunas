# TUNAS — Release Notes

S.A.L.T CI · Owned by S.A.L.T. CI TECHNOLOGY · Distributed by Pemangkin Teknologi
Newest first. Any change after a release gets a new version number.

---

## 0.2.2 — 19 November 2026 (built-in S.A.L.T reference fixes)

- **Fixed: "salt tu apa?" got an invented answer** ("System for Advanced Learning and Testing"). The built-in reference was only used for "S.A.L.T", "salt ci", "teras" and similar, not for the plain word "salt".
  - It now also triggers on "salt" as a word, except in cooking talk ("garam", "sugar", "pepper", "resipi" and similar).
  - It looks at the last three questions, so follow-ups like "siapa pemiliknya?" keep the reference.
- **One fixed fact is always given to the model:** what S.A.L.T stands for, its owner and its distributor, and "never invent other meanings for S.A.L.T". Even questions that don't trigger the full reference can't produce a made-up meaning.
- **Wrong answers earlier in a chat are corrected.** The reference tells the model that anything different said earlier in the chat was wrong.
- Tested on the build machine with Qwen 3.5 2B:
  - "salt tu apa ?" with a co-brand set;
  - the follow-up "siapa pemiliknya dan siapa pengedarnya?";
  - "Berapa harga TERAS?" (answered as not in the reference);
  - a chat that already held the invented answer.

  All four answered from the reference.
- Install over 0.2.1 directly. Everything is kept. Old chats keep their earlier answers; start a new chat to test.

---

## 0.2.1 — 19 November 2026 (simpler main screen, Settings panel, model per chat)

- **Cleaner main screen.**
  - The sidebar now holds only what people use every day: New chat, search, the chats, and My assistants (Knowledge joins it in 0.3).
  - At the bottom: "Powered by S.A.L.T CI" and a **⚙ Settings** button.
- **Settings panel with its own menu:**
  - General: default model, run AI on, appearance, privacy, data folder.
  - Models: check my machine, catalogue, downloads, USB.
  - Branding.
  - Connections: TERAS, cloud providers, apps that use TUNAS.
  - About.

  "Back" or Esc returns to the chat. Links such as "Open Models" open the right page of Settings.
- **Model chip in the chat header.** It shows the model answering this chat. Click it to pick another model for this chat only: a model on this machine, a TERAS model, or (if allowed) a cloud model. "Assistant's model" goes back to the assistant's own choice. The choice is remembered per chat, and starting a new chat or choosing another assistant resets it.
- The engine label in the header now shows only Graphics or Processor, since the chip shows the model.
- The Privacy text in General now mentions TERAS and cloud providers.
- The Settings gear is a proper cog icon.
- Install over 0.2.0 directly. Everything is kept.

---

## 0.2.0 — 19 November 2026 (Connections: TERAS, cloud providers, apps that use TUNAS; built-in S.A.L.T reference)

The order of the plan changed on 19 Nov: Connections and the local API come in 0.2; Knowledge and Documents move to 0.3.

### New: Connections page (sidebar)
- **Your organisation's TERAS:**
  - Enter the TERAS address and your key, then Connect. TERAS's models are listed and become base models for your assistants.
  - Messages go only to your organisation's server, and the chat shows "Your organisation's server".
  - Refresh models, switch off or remove.
- **Cloud AI providers:**
  - Off by default, behind the "Allow cloud providers" switch, with a clear note that chats with a cloud model leave the machine.
  - Providers: OpenAI, Anthropic (Claude), Google Gemini, OpenRouter, or any OpenAI-compatible service.
  - Add your own key, load the provider's model list, tick the models to use (search or type a name).
  - Keys are stored protected by Windows (DPAPI) for the signed-in user.
  - A chat using a cloud model shows an amber "Online · leaves this machine" label.
  - The organisation's policy can block cloud providers.
- **Apps that use TUNAS (local API):**
  - Off by default. When switched on, other apps on this machine (such as S.A.L.T NEXUS desktop) can use TUNAS as their AI engine through an OpenAI-compatible API, at `http://127.0.0.1:11435/v1` (port changeable). This machine only.
  - `GET /v1/models` lists your assistants (for example `surat-rasmi`), local models and connected provider models (`teras/<model>`). `POST /v1/chat/completions` answers, with streaming.
  - Asking for an assistant applies its persona and base model.
  - Each app gets its own key, shown once when made. The list shows how often and when each key was used. Remove a key to cut that app off.
  - The organisation's policy can turn the API off.
- **My assistants:** the base model can now be a model on this machine, a TERAS model, or (with cloud allowed) a cloud model.

### New: built-in S.A.L.T reference
- A short reference on S.A.L.T CI, TERAS, TUNAS, S.A.L.T. CI TECHNOLOGY, Pemangkin Teknologi and the S.A.L.T CI apps is built into TUNAS. Users cannot edit or delete it.
- When a question mentions these names, the reference is given to the model with the question, so answers stay correct offline and on small models. Anything not in it (prices, staff, clients, dates) is answered with "not in this reference" and the contact addresses.
- It works through the local API too.
- Note from testing: the very small 0.8B model can still mix up details; the 2B and 4B models are recommended.

### Changed
- All chats now go through one TUNAS gateway inside the app. It adds the persona, the language rule and the reference, and routes to the local engine, TERAS or a cloud provider.
- Password fields are styled like other fields.

### Tested before release
- Built and run on the build machine, on a real engine with a small model:
  - app key required (a wrong key is refused);
  - model list;
  - assistant persona answer in Malay;
  - streaming;
  - the reference answer on owner and distributor;
  - routing an assistant to an OpenAI-compatible provider;
  - cloud models refused while cloud is switched off.
- Windows run test on CAPT ZEDs' machines.

Install over 0.1.4 directly. Chats, assistants, branding and models are kept.

---

## 0.1.4 — 19 November 2026 (cosmetic fixes from the .76 test)

- **Fixed: blue buttons turned white when the mouse was over them**, so they seemed to disappear. They now stay blue and brighten slightly.
- **Logo background for co-brands.** On the Branding page, a logo can sit on a **light** or **dark** background, as a **rounded square** or a **circle**.
  - The default is still none (transparent).
  - Use it when a white logo vanishes on light screens, or a dark logo on dark ones.
  - The background is used everywhere the logo appears: the sidebar, the welcome screen, the About page, the window icon, and the desktop and Start menu icons.
- **"Powered by S.A.L.T CI" now shows the gold S.A.L.T logo** beside the text: in the sidebar, the Branding page, the setup screens and the About page.
- **"Notebook" is now "machine"** everywhere in the app and installer ("Check my machine", "runs privately on this machine"), since TUNAS runs on any Windows PC, not only notebooks.
- Field hints on the Branding and assistant pages now use the smaller hint style.
- Install over 0.1.3 directly. Chats, assistants, branding and models are kept.

---

## 0.1.3 — 19 November 2026 (installer fix)

- **Fixed: "Error opening file for writing … engine\vulkan\ggml-base.dll" during an update.** A leftover engine process from an earlier version (the graphics test that hung in 0.1.1) still held the file open.
  - The installer now stops TUNAS and engine processes again just before copying, and waits for Windows to release the files.
  - Each version installs its engine into its own folder (`engine\0.1.3\…`), so a stuck old process can no longer block an update. Old engine folders are removed when they are free.
- **Installer header:** the gold S.A.L.T logo on the inner pages is smaller, with space around it, instead of touching the frame.

---

## 0.1.2 — 19 November 2026 (fixes from the .76 test, and S.A.L.T branding)

- **Fixed: setup stuck on "Checking memory, processor and graphics…".** On some graphics drivers (seen on .76 with a Quadro K6000), the engine's device test prints the device list and then never closes, so TUNAS waited forever.
  - TUNAS now reads the list as soon as it appears and closes the test itself.
  - Every outside check has a time limit: 12 s for the Windows graphics query and 15 s for the engine test.
  - The checks run side by side, and the result is remembered for the rest of the session. The engine start uses the same remembered result.
- **S.A.L.T branding in the installer:**
  - The Welcome and Finish pages have a navy side panel with the TUNAS mark at the top, the gold 3D S.A.L.T logo in the middle, the S.A.L.T. CI TECHNOLOGY and Pemangkin logos below, and both web addresses.
  - The other pages show a small gold S.A.L.T logo at the top right.
  - Every page carries "TUNAS 0.1.2 - Powered by S.A.L.T CI".
- **The same brand strip** (gold S.A.L.T logo, "Powered by S.A.L.T CI", owner and distributor logos, links) appears at the bottom of the setup screens and on the About page.
- Install over 0.1.1 directly. Chats, assistants, branding and models are kept.

---

## 0.1.1 — 19 November 2026 (fixes after the first install on .76)

- **Fixed: the app window showed "localhost refused to connect".** 0.1.0 was built without its screens packed inside, so it looked for a development server. The screens are now embedded in TUNAS.exe.
- **Fixed: the installer and the uninstaller could run twice.** Each now allows only one copy of itself at a time.
- **New TUNAS mark: option B** (a shoot growing from a gold seed). It is used for the app icon, the installer, the shortcuts and inside the app when no co-brand is set.
- The release notes now come inside the zip.
- Install over 0.1.0 directly. Chats, assistants, branding and models are kept.

---

## 0.1.0 — 19 November 2026 (first build, for testing on CAPT ZEDs' notebooks)

**Status: built, not yet tested on Windows.** It was built and checked on the build machine. The Windows run test is on CAPT ZEDs' 8 GB reference notebook and VM.

### Package
- `TUNAS_0.1.0.zip` (25.7 MB), containing `TUNAS_Setup_0.1.0.exe`.
- The installer is per user, needs no administrator rights, and installs to `%LOCALAPPDATA%\Programs\TUNAS`.
- Data (chats, assistants, branding, models) lives in `%LOCALAPPDATA%\TUNAS`. Uninstall asks whether to keep it.
- The installer is not code-signed yet, so Windows SmartScreen may warn on first run.

### What is in 0.1.0
- **Setup on first start:**
  - Welcome, then Check my notebook.
  - Models: pre-ticked for this notebook. Download them, or take them from a folder or USB stick.
  - Branding (optional).
  - Ready. Downloads continue in the background.
- **AI engine:**
  - llama.cpp (build 08c252e, 18 Nov 2026), in CPU and Vulkan builds.
  - Automatic mode uses the graphics through Vulkan when a usable device is found, and falls back to the processor.
  - Settings can force Graphics or Processor.
  - The engine listens on 127.0.0.1 only, with a random port and a random key, and closes when TUNAS closes.
- **Check my notebook:**
  - Shows memory, processor, graphics and free disk, with a recommendation.
  - **Test speed** measures the time to the first word and tokens per second on this notebook.
- **Model catalogue:**

  | Model | Licence | In the default set |
  |---|---|---|
  | Qwen 3.5 2B | Apache-2.0 | Yes |
  | Qwen 3.5 4B | Apache-2.0 | Yes |
  | Granite 4.2 3B | Apache-2.0 | Yes |
  | Qwen 3.5 0.8B | Apache-2.0 | Catalogue only |
  | SEA-LION v4 4B | Licence note shown | Catalogue only, user downloads it |

  - Downloads pin the exact file version, resume after an interruption, and are checked against a SHA-256 checksum.
  - They try `models.salt.ci` first (not set up yet) and then fall back to Hugging Face.
- **Offline models:**
  - Import from a folder or USB stick. Each file is checked against the catalogue checksum before it is copied in.
  - Copy installed models to a folder, to prepare a USB stick.
  - Add your own GGUF model file.
- **Chat:**
  - Answers stream in; you can stop, answer again, edit and resend, and copy.
  - Chats are saved on the notebook, searchable, and titled by the model.
  - The chat follows the user's language (English, Malay, or mixed).
  - Drop or attach PDF, Word, Excel, PowerPoint, text, Markdown, CSV and HTML files. Their text is read for that chat only.
  - Scanned PDFs are reported as not readable.
- **My assistants:**
  - Name, icon, colour, persona and base model.
  - Seven starter assistants: General Assistant, Surat Rasmi, Official Letter, Minit Ringkas, Meeting Minutes, Email Writer and Summariser.
  - Create, edit, copy and delete.
- **Co-branding:**
  - Three layers: personal, organisation, and OEM (OEM is read-only, set at installation). "Powered by S.A.L.T CI" is always shown.
  - Each brand has a name, a logo (PNG, SVG or JPG, checked for icon use), two colours, a welcome line and an assistant name.
  - Live preview of the window, the welcome screen, and the desktop and taskbar icons.
  - Saving updates the window title and icon, and renames and re-icons the TUNAS desktop and Start menu shortcuts.
- **Branding packs (`.tunasbrand`):**
  - Make one from a brand, optionally with assistants and organisation rules.
  - Import from the Branding page or during setup, or double-click the file (the installer registers `.tunasbrand`).
- **Organisation rules:**
  - Change organisation brand, add personal brand, create assistants.
  - Rules come from a pack, or machine-wide from `%ProgramData%\TUNAS\policy.json`.
- **About:** the brands on this copy, "What TERAS adds", open-source notices, and the brand and IP lines.
- **Settings:** default model, run AI on (Automatic, Graphics or Processor), light or dark, engine status and stop, data folder.

### Not in 0.1.0 (planned)
- Knowledge and documents: 0.2.
- Providers, Connect to TERAS, the local AI API, silent install with a settings file, signed packs, and updates: 0.3.
- The letterhead fields are kept in the brand file but not used until documents arrive (0.2).
- The TUNAS mark is option A as a placeholder until CAPT ZEDs chooses.

---
S.A.L.T CI and its components are proprietary technology of S.A.L.T. CI TECHNOLOGY, distributed and commercialised by Pemangkin Teknologi.
