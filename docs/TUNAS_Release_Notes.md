# TUNAS — Release Notes

S.A.L.T CI · Owned by S.A.L.T. CI TECHNOLOGY · Distributed by Pemangkin Teknologi
Newest first. Any change after a release gets a new version number.

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
