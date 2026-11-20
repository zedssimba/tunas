# TUNAS — Release Notes

S.A.L.T CI · Owned by S.A.L.T. CI TECHNOLOGY · Distributed by Pemangkin Teknologi
Newest first. Any change after a release gets a new version number.

---

## 0.3.6 — 20 November 2026 (sources back for questions about your documents)

- **Fixed: questions about the documents got citation numbers but no sources listed.** 0.3.5 judged relevance mostly by closeness of meaning, with a line measured on the demo documents. On a folder of release notes the scores came out differently: "what is in our new product release" scored 0.497 with 30 matching passages, just under the line, so no passages were given to the model. It answered from earlier in the chat, with old citation numbers and nothing to list.
  - Words from the documents now decide: a message whose content words appear in the documents uses them (from a closeness of 0.40). Closeness alone counts only when it is high (0.58).
  - Greetings stay out: on the same release notes "salam" scored 0.503 but matches no words, so it gets a plain reply.
  - Checked against every search logged on both machines: all 13 messages (greetings, small talk, unrelated questions and questions about the documents) are now decided correctly.
- A few more everyday words ("new", "tell", "know", "need" and similar) no longer count as matches.
- Install over 0.3.5 directly. Everything is kept.

---

## 0.3.5 — 20 November 2026 (sources only when they are used)

- **Fixed: every answer listed the documents, even for "hi", "salam" or "terima kasih".**
  - Before, an assistant with Knowledge always took the 6 closest passages, however unrelated the message, and the chat listed all of them under the answer.
  - Now TUNAS checks that the message is really about the documents before using them. Greetings, thanks and unrelated questions get a normal answer with no sources.
  - Measured with the demo set: greetings and unrelated requests scored 0.17 to 0.47 by meaning; questions about the documents scored 0.55 to 0.76. Documents are used from 0.56, or from 0.50 when words from the documents also match.
- **Short messages are no longer joined to the previous question.** "terima kasih" after a question about the circular no longer brings the circular back. A short follow-up question ("bila tarikhnya?", "and the deadline?") is still searched together with the question before it when it finds nothing on its own.
- **Sources list only what the answer cites.** If the answer uses [1] and [3], only those files are listed. An answer that cites nothing lists no sources. Older chats show the same, without changing what is saved.
- **The Knowledge log** (`logs\knowledge.log`) records each search: how close the best passage was, the word matches, and whether the documents were used.
- Tested on the build machine: "Berapa hari seminggu pegawai boleh bekerja dari rumah?" was answered from the circular with one source chip (before: 6 sources from 4 files); "terima kasih" in the same chat got a plain reply with no sources.
- Install over 0.3.4 directly. Everything is kept.

---

## 0.3.4 — 20 November 2026 (chatting with TERAS models)

- **Fixed: chatting with a TERAS model failed with "`temperature` is deprecated for this model".**
  - TUNAS's chat sends its own sampling settings (temperature and similar), tuned for the small models on this machine. Some newer models behind TERAS refuse them.
  - TUNAS's chat now leaves those settings to TERAS and cloud models, and sends them only to local models.
  - Apps using the TUNAS API still have their settings passed on. If TERAS or a cloud provider refuses one, TUNAS sends the request once more without them, so the answer still comes.
  - Tested on the build machine with a stand-in TERAS that refuses `temperature`: the first request was refused, the second went through without it, and the answer came back.
- Install over 0.3.3 directly. Everything is kept, including the TERAS connection.

---

## 0.3.3 — 20 November 2026 (sources grouped by file)

- **Each file is listed once under an answer, with all its citation numbers**, for example `1 2 3 4 5  SALT_CI_All_Release_Notes.md`, instead of the same file repeated for every citation.
  - Hover over it to see what each number points to (the section, page, slide or sheet).
  - Click it to open the file.
- Install over 0.3.2 directly.

---

## 0.3.2 — 20 November 2026 (faster reading for Knowledge)

- **Reading uses the graphics card when it can.** On the Windows test machine, Knowledge read about one piece of text per second on the processor (low priority, half the cores) while the Quadro sat idle.
  - Once the chat engine has run on graphics on a machine, the search engine uses graphics too: many times faster.
  - If graphics cannot start, it falls back to the processor.
- **On the processor, all cores are used,** still at low priority, so the machine stays responsive. Measured on the build machine: twice as fast as using half the cores.
- **Progress inside big files:** the Knowledge page now shows, for example, "SOP.pdf · 120 of 800 pieces" instead of only the file name.
- **A reading log** (`logs\knowledge.log`) records each file read, how long it took, waits while an answer is being written, which processor the search engine runs on, and anything that could not be read.
- Install over 0.3.1 directly. Reading carries on where it stopped.

---

## 0.3.1 — 20 November 2026 (fixes from the first Knowledge test)

- **Fixed: an assistant with Knowledge answered from the built-in S.A.L.T reference instead of your documents.**
  - The reference told the model to use only its own facts about S.A.L.T CI, and to say that releases and versions are "not in this reference". That beat the release notes you had given it.
  - Now your own documents come first for anything the reference does not cover: releases, versions, dates, figures. The reference still keeps the basic facts right.
- **The chat says when knowledge is not ready yet.**
  - The "Answering from" line above the message box shows "waiting for the search model" or "reading 3/12" until a set is fully read.
  - If you ask before then, the assistant says the documents are not ready, instead of answering as if they did not exist.
- **Better answers from documents with sections** (release notes, SOPs, minutes):
  - Markdown files are read section by section, and every piece carries its heading (for example "0.2.4 — 20 November 2026"), so details from different versions are not mixed.
  - Pieces from the same file reach the model in the order they appear in the file, together with the file's list of sections.
  - Questions with "latest", "newest", "terkini" or "terbaru" also get the opening of the best-matching document, where newest entries usually are.
  - Version and reference numbers (0.2.5, 7/2026) count extra in the search.
  - Markdown files you already added are read again once after the update.
  - Tested with the TUNAS release notes plus a S.A.L.T CI release-note PDF: "what is our latest release", "what is the latest TUNAS release?" and "apa yang baru dalam TUNAS 0.2.5?" are answered correctly by the 2B model, with sources.
- **One place for knowledge.** The separate Settings → Folders page is gone. Folder permissions are now at the bottom of the **Knowledge** page, under the knowledge sets.
- **Clearer download message.** The download on first use is TUNAS's search model (about 640 MB, once), not your files. The Knowledge page now says so.
- **Cloud-drive folders** (Google Drive, OneDrive, Dropbox): the set shows a note that the drive app may fetch each file the first time TUNAS reads it. Mark the folder "Available offline" for faster reading.
- Install over 0.3.0 directly. Everything is kept.

---

## 0.3.0 — 20 November 2026 (Knowledge and Documents)

### Knowledge: assistants that answer from your own documents
- **Your files stay where they are.** TUNAS reads documents in their own folders on this machine. It never copies them into the app, never changes or deletes them, and sends nothing anywhere.
- **TUNAS asks first.** The first time it reads a folder, it explains what will happen and waits for **Allow**.
  - **Settings → Folders** lists every place TUNAS may read. **Remove** stops the reading and deletes that place's search index; the files are untouched.
- **Knowledge sets.** A knowledge set is a name plus one or more folders or files, for example "Pekeliling & SOP Jabatan". Several assistants can share one set, and it is read only once.
- **Where you use it:**
  - **My assistants:** each assistant has a **Knowledge** field ("New from folder", "Add existing").
  - **In a chat:** the new **#** button next to the paper clip, or typing **#**, adds a set to that chat only. The line above the message box shows what the answer draws on.
  - **Knowledge** in the sidebar: every set with its status, size, the assistants that use it and files it could not read. Rename, add or remove a folder, check for changes, pause, or delete. Deleting removes only TUNAS's index.
- **Sources under each answer**, with the file name and page, slide or sheet. Click one to open the file.
- **File types:** PDF with text, Word, Excel, PowerPoint, text, Markdown, CSV and web pages. Scanned PDFs are listed as "could not read" (reading scanned pages comes later).
- **Search finds by meaning and by exact words,** so reference numbers such as JPA(S)PKPA/7/2026 and names are found. Malay and English questions find documents in either language.
- **The search model** (Qwen 3 Embedding, about 640 MB) downloads by itself the first time you make a knowledge set. After that, everything works offline.
- **Light on the machine:**
  - reading runs in the background, one file at a time, at low priority;
  - it waits while an answer is being written;
  - the search model is unloaded after 5 idle minutes.
- **Cloud models:** if an assistant uses a cloud model, the parts of your documents used in an answer go to that provider. The assistant panel says so; local models keep everything on the machine.
- **Changes in folders:** press the refresh button on a set ("Check for new and changed files"), or restart TUNAS (it checks at start). Watching folders automatically comes after macOS.
- Tested on the build machine with a folder of Word, PDF (3 pages), Excel, PowerPoint and Markdown files:
  - all five files were read in about 5 seconds;
  - "Berapa hari seminggu pegawai boleh bekerja dari rumah?", "How many hours do we have to fix a critical ICT complaint?" and "Berapa kos latihan AI TUNAS dalam bajet 2027?" were answered correctly from the files, with sources, by the Qwen 3.5 2B model.

### Documents: Word, Excel and PDF
- **Save as** (the download icon under every answer): Word (.docx), PDF, or Excel (.xlsx).
  - Excel puts each table on its own sheet, with numbers as real numbers ("1,200.50" and "RM 300" can be added up).
  - The file opens when it is saved.
- **"Your document is ready".** When you ask for a letter, memo, minutes, report, proposal, notice, table and similar (in Malay or English), the answer ends with a card holding ready buttons: Word, PDF, and Excel when there are tables.
- **Your letterhead** comes from Branding: the logo, the letterhead organisation name (or the brand name) and the address.
- Source markers such as [1] are left out of the files.
- Made inside TUNAS, offline. No office suite is needed to make them.

### Also
- The "Search" engine process has its own log: `logs\engine-search.log`.
- Third-party notices updated (Liberation Sans fonts, document libraries).
- Install over 0.2.5 directly. Everything is kept.

---

## 0.2.5 — 20 November 2026 (thinking that ends with an answer; app icon choice)

- **Fixed: with thinking on, the model thought until it ran out of room and never answered.** Small models (Qwen 3.5 4B and smaller) tend to think in circles ("Wait, is it…").
  - Each model now has a **thinking length**: Short (about 500 tokens), Normal (about 1,000, the default) or Long (about 2,000). When the length is used up, the engine closes the thinking and the model answers.
  - With thinking on, TUNAS uses the settings Qwen recommends for thinking, which stop the circling.
  - Tested on the build machine with "Apa kelebihan tenaga solar berbanding arang batu untuk Malaysia?":
    - before: about 5,900 tokens of thinking and no answer;
    - now: about 1,000 tokens of thinking, then a full answer.
  - Set the length in the chat's **Thinking** menu ("How long to think"), or in **Settings → Models** next to each model.
  - Apps using the TUNAS API get the same length by default. They can set their own with `thinking_budget_tokens`.
- **App icon: your choice.** A branding logo no longer replaces the TUNAS icon automatically. Before, it changed only when the brand was applied again (for example when the engine restarted), which looked random.
  - In **Settings → Branding**, tick **"Also use as the app icon"** to put the logo on the taskbar, desktop and Start menu.
  - Off by default: the TUNAS icon stays, and the logo shows inside the app.
  - The preview shows which icon will be used.
- Installer starting twice: when an antivirus tool scans the installer, the tool starts it again after the scan. That can't be stopped from inside the installer; signing the installer (planned before the OEM release) is the real fix. Meanwhile, if a second installer window appears, close it.
- Install over 0.2.4 directly. Everything is kept. If you had a logo set and want it as the icon, tick the new box once.

---

## 0.2.4 — 20 November 2026 (fixes from testing 0.2.3)

- **Fixed: chats got stuck after the first answer.**
  - The cause: an answer kept running in the background after you moved to another chat or to Settings. TUNAS has one engine, so every new question waited behind it. With thinking on there was no length limit, so it could keep going for a very long time on a processor.
  - Now moving to another chat or leaving the chat screen stops the answer and keeps what was written so far. The engine is free straight away.
  - Thinking now has a limit (about 6,000 tokens), so it always ends.
- **The Send button is ready as soon as the answer ends.** The chat title is now written in the background. Sending your next message cancels it, so it never makes you wait.
- **"Network error" replaced.** If the engine is stopped during an answer, the chat now says so and asks you to send again. The Stop engine button updates the Engine line right away, and only shows while the engine is running.
- **Run AI on (Processor / Graphics) now takes effect with the next message.** Before, a running engine kept its old choice until TUNAS was restarted.
- **When graphics cannot start, TUNAS says why.** The reason shows under Engine in Settings, and TUNAS stops retrying graphics on "Auto" until you choose Graphics again. Each mode keeps its own log: `logs\engine-vulkan.log` and `logs\engine-cpu.log`. Graphics may take up to 5 minutes to load the first time.
- **The graphics card check runs once per install** instead of at every start. That check is what made antivirus tools pop up a scan window showing "Available devices: Vulkan0 …" when a chat started.
- **Installer no longer runs twice.** Some antivirus tools hold the installer back and then start it again after the first run ends. A second start of the same version within 3 minutes now quits quietly instead of installing again and closing TUNAS.
- **The taskbar icon no longer changes after start.** When a branding logo is set, it becomes the app icon (that is the co-branding feature). The window now waits until the icon is ready before it appears.
- Install over 0.2.3 directly. Everything is kept.

---

## 0.2.3 — 20 November 2026 (smoother typing, thinking per model)

- **Smoother typing.** Answers now appear at an even pace instead of in jumps.
  - Before, the whole formatted answer was redrawn for every word the model sent, which made the text stutter on long answers.
  - Now words come out steadily, and the formatting is redrawn at most about 16 times a second. The model's speed is unchanged; only the display is smoother.
- **Thinking, set per model.** Some models (Qwen 3.5) can think before they answer. Each model now has three settings:
  - **Off** (default): fastest; the answer starts straight away.
  - **On, hidden**: the model thinks first for a better answer; the user sees only "Thinking… 12.4 s".
  - **On, shown**: the thinking appears live in grey italics, then folds away when the answer starts. Good for showing a class how the model reasons. Click "Thought for X s" to open or close it again.
- **Where to set it:**
  - in a chat, from the new **Thinking** chip next to the model chip; the choice is saved for that model;
  - in **Settings → Models**, next to each installed model.
- Saved answers show "Thought for X s". In "shown" mode the thinking is kept with the answer and can be opened later.
- Apps using the TUNAS API get the same per-model setting by default, and can override it per request with `chat_template_kwargs.enable_thinking`. The thinking comes back as `reasoning_content`.
- Thinking makes answers slower to start, sometimes much slower on a Processor-only machine. Chat titles and the speed test always run with thinking off.
- Install over 0.2.2 directly. Everything is kept.

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

## 0.1.4 — 19 November 2026 (cosmetic fixes from the Windows test)

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

## 0.1.2 — 19 November 2026 (fixes from the Windows test, and S.A.L.T branding)

- **Fixed: setup stuck on "Checking memory, processor and graphics…".** On some graphics drivers (seen with a Quadro K6000), the engine's device test prints the device list and then never closes, so TUNAS waited forever.
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

## 0.1.1 — 19 November 2026 (fixes after the first Windows install)

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
S.A.L.T CI · Owned by S.A.L.T. CI TECHNOLOGY · Distributed by Pemangkin Teknologi · www.salt.ci · info@salt.ci · www.pemangkin.tech · info@pemangkin.tech
S.A.L.T CI and its components are proprietary technology of S.A.L.T. CI TECHNOLOGY, distributed and commercialised by Pemangkin Teknologi.
