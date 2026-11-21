# S.A.L.T TUNAS: User Guide

TUNAS 0.6.0 · November 2026 · https://www.salt.ci
S.A.L.T CI · Owned by S.A.L.T. CI TECHNOLOGY · Distributed by Pemangkin Teknologi

This is the knowledge copy of the TUNAS User Guide for AI knowledge bases (TUNAS 0.6.0). What TUNAS is, and what is not available yet, is in the TUNAS Product Brief (knowledge version). Rolling TUNAS out across an organisation is in the TUNAS Administrator Guide.

This guide is for the person using TUNAS on their own Windows computer, Mac or Android phone or tablet.

## 1. Installing TUNAS

### On Windows

1. Unzip the package and run the TUNAS installer (`TUNAS_Setup_<version>.exe`). It installs for you alone and needs no administrator password.
2. Some antivirus tools scan a new installer before it runs, and may start it a second time after the scan. If a second installer window appears after the first one finishes, close it.
3. TUNAS opens into setup.

**Updating:** run the new installer over the old one. Chats, assistants, knowledge, branding and models are kept.

**Uninstalling:** from Windows Settings › Apps. The uninstaller asks whether to also delete your chats, assistants, branding and downloaded models; choose No to keep them for a later reinstall.

### On a Mac (from TUNAS 0.5.0)

TUNAS runs on macOS 11 (Big Sur) or later. The Mac package `TUNAS_<version>_mac.zip` holds a README, the release notes and two disk images; pick the one for your Mac (Apple menu › About This Mac shows the chip):
- `TUNAS_<version>_mac_apple-silicon.dmg` for Macs with an M1 chip or later (the AI runs on the graphics chip);
- `TUNAS_<version>_mac_intel.dmg` for Macs with an Intel processor (the AI runs on the processor).

1. Open the `.dmg` and drag **TUNAS** into **Applications**. The disk image also has **READ ME FIRST.txt** with these steps.
2. The Mac packages are not yet signed by Apple, so macOS blocks them the first time. Open **Terminal** and run this once:
   `xattr -dr com.apple.quarantine /Applications/TUNAS.app`
3. Open TUNAS from Applications. It opens into setup.
4. The first time TUNAS saves a TERAS or cloud key, macOS asks to let TUNAS use the Keychain. Choose **Always Allow**.

**Updating:** drag the new TUNAS into Applications and replace the old one, then run the Terminal line again. Your chats, models and settings are kept in your home folder (`Library/Application Support/TUNAS`).

**Uninstalling:** drag TUNAS from Applications to the Bin. To delete your chats and models too, delete the `TUNAS` folder in your home folder's `Library/Application Support`.

The Mac version is new: the 0.5.x packages are the first Mac builds.

### On an Android phone or tablet (from TUNAS 0.6.0)

TUNAS runs on Android 9 or later with a 64-bit processor (almost every phone and tablet from 2018 on); 6 GB of memory or more is recommended. The package `TUNAS_<version>_android.zip` holds `TUNAS_<version>_android.apk`, a README and the release notes.

1. Copy the `.apk` to the phone, or download it there, and open it.
2. Android asks to allow installing apps from that source (your Files app or browser): allow it, then tap **Install**.
3. If Play Protect says the app is unknown, choose **More details**, then **Install anyway**. TUNAS is not on Google Play yet.
4. Open TUNAS. Setup checks the phone, recommends a small model and downloads it (0.5 to 1.3 GB; use Wi-Fi).

**Updating:** open the new `.apk` and tap **Update**. Everything is kept.

**Uninstalling:** long-press TUNAS › App info › Uninstall. This also deletes TUNAS's chats, models and settings on the phone.

**Different on a phone:** the AI runs on the phone's processor (small models answer fastest; keep the phone charging for long chats). Knowledge is made from documents you pick (**New from documents**), and TUNAS keeps a copy of them inside the app. **Save as** asks where to save, for example Downloads. Models are downloaded in the app; there is no USB import. The ☰ button at the top left opens the menu with your chats, My assistants, Knowledge and Settings.

The Android version is new: 0.6.0 is the first Android build.

### Updates (from TUNAS 0.6.0)

TUNAS checks once a day whether a new version is ready (it sends nothing about you). When one is, a card at the top says so, with **What's new**, **Later** and **Update**, and a dot appears on Settings. **Update** downloads it, checks it, and starts the installer: on Windows it installs over TUNAS; on a Mac drag the new TUNAS into Applications; on Android tap **Install**. **Settings › About › Updates** has **Check now**. All TUNAS downloads are also at `https://get.saltci.app`.

## 2. First-time setup

1. **Welcome:** press **Get started**.
2. **Your machine:** TUNAS checks memory, processor and graphics (up to 15 seconds).
3. **Choose your models:** TUNAS ticks the best ones for this machine. Press **Download**, or **From a folder or USB stick** if someone gave you the models (no internet needed; each file is checked and copied in).
4. **Make it yours (optional):** **For my organisation** or **Just for me**: a name, a logo (**Choose logo…**) and a welcome line, or **I have a branding pack**. Press **Save and continue**.
5. **Start chatting.** Downloads continue in the background; you can chat as soon as one model is ready.

## 3. Chatting

- **New chat** at the top of the side bar. Type in Bahasa Melayu, English or both and press Enter. Shift+Enter makes a new line.
- **Starter prompts** on the welcome screen come from the assistant; tap one to ask it.
- **Drop in a file:** drag a PDF, Word, Excel, PowerPoint, text, CSV or HTML file into the chat, or use the paper clip. TUNAS reads it on the computer and uses it in the answer. Scanned PDFs with no text can't be read.
- **At the top of the chat:**
  - **the assistant** (who answers);
  - **the model** (which AI model it uses for this chat);
  - **the thinking chip** (section 6);
  - **Processor** or **Graphics:** where the model is running;
  - **where the answer is made:** "Offline & private" for models on this computer, "Your organisation's server" for TERAS, "Online · leaves this machine" for a cloud model.
- **Stop** ends an answer early. Moving to another chat also stops the answer in progress.
- **Under each answer:** copy, **Save as Word, PDF or Excel** (section 8), the model and the time it took.
- **Chats** are kept on the side bar under Today, Previous 7 days and Earlier, with a title made automatically. **Search chats** finds them by words. Hover a chat for its **Rename** and **Delete** buttons.
- The first answer after starting TUNAS, or after switching models, takes longer while the model loads.

### Chatting with a TERAS model
- **TERAS at work:** when the TERAS model uses a tool, the answer shows it as it happens, such as "Making the document…" or "Searching the web…", with a tick when a step is done.
- **Documents from TERAS** (from 0.5.1): ask a TERAS model for a document and TERAS makes the file with its own document tool, in the format you asked for: Word, PDF, PowerPoint, Excel, CSV or Markdown. It uses your organisation's TERAS templates and branding.
- **Rich document** (from 0.5.2): the **Rich document** button beside the message box (TERAS models only). Switch it on for a richer document: a cover, clear sections, tables and charts from the figures, and every page checked. The answer shows the steps (Planning the document, Drawing the pages, Checking the pages). It takes a few minutes, and stays on for that chat until you switch it off.
- **Files from TERAS:** a document TERAS makes appears as a card under the answer with a **Download** button. TUNAS saves it where you choose. If TERAS asks for your sign-in for that file, it opens in your browser.
- **Sources:** TERAS's own sources are listed with yours, each with where it was found. Web pages TERAS found open in your browser. With a TERAS model, passages from your own folders are numbered from [51].
- **Time:** under the answer you see how long it took and how many tool steps TERAS used.

## 4. My assistants

**My assistants** at the bottom of the side bar.

- **The seven that come with TUNAS:** General Assistant, Surat Rasmi, Official Letter, Minit Ringkas, Meeting Minutes, Email Writer and Summariser.
- **Chat** starts a chat with that assistant; **Edit** changes it; **Copy** makes your own copy to change.
- **New assistant**, or Edit, opens the editor:
  - **name, icon and colour;**
  - **persona:** how it writes, what it asks, and which language it answers in;
  - **starters:** the prompts shown on the welcome screen;
  - **Knowledge:** the knowledge sets it answers from (section 7), or **New from folder**;
  - **Base model:** a model on this computer, a TERAS model, or a cloud model.
- **Delete** removes an assistant you made.

Example persona for Surat Rasmi: "Anda membantu menulis surat rasmi dalam Bahasa Melayu mengikut format surat kerajaan Malaysia: alamat pengirim, rujukan dan tarikh, alamat penerima, salam hormat, tajuk huruf besar dan tebal, perenggan bernombor, penutup, dan tandatangan."

## 5. About me

**Settings › About me** tells TUNAS a little about you, so answers fit you and your work.

- **Fields:** what to call you, your position, organisation or department, what you work on, your languages, and how you like answers (for example "ringkas", "formal", "with examples"). Fill in only what you like.
- **Use this in my chats** turns it on or off.
- It goes with each message to the models on this machine and to cloud models. **TERAS models** use your own TERAS personalisation instead.
- Letters and documents can be signed with your name and position from here.
- It is kept on this computer only and is never given to other apps.

## 6. Thinking

The Qwen 3.5 models can think before answering. The thinking chip at the top of the chat (or the Models page) sets it **for that model**, in every chat:

- **Thinking off:** fastest; answers straight away.
- **On, hidden:** thinks first for better answers; shows only "Thinking…" and the time.
- **On, shown:** shows the thinking live as it happens, then folds it away. Good for classes.
- **How long to think:** short, normal or long. Longer thinking helps with harder questions but takes more time; the answer always follows.

Thinking is for models on this computer; TERAS and cloud models answer as their servers decide.

## 7. Knowledge: answers from your own folders

Knowledge lets assistants answer from documents already on your computer, without uploading them anywhere.

### Make a knowledge set

1. Open **Knowledge** on the side bar, then **New from folder**. Or, in an assistant's editor, **New from folder** under Knowledge.
2. Choose the folder. TUNAS asks for permission to read it: confirm. TUNAS only reads; it never changes, moves or deletes your files.
3. Name the set, for example "Pekeliling & SOP Jabatan".
4. The first time, TUNAS downloads its reading model (Qwen 3 Embedding 0.6B, about 640 MB). Then it reads each file and shows "name · 3 of 12 pieces" as it goes, and Ready when done.

### Use it

- **For an assistant:** add the set to the assistant's **Knowledge** field. Every chat with that assistant answers from it.
- **For one chat:** type **#** in the message box and pick a set. "Answering from" above the box shows the sets in use: the assistant's own sets, and sets picked for this chat, which have an ✕ to take them away.
- **Ask normally:** "Berapa hari seminggu pegawai boleh bekerja dari rumah?" The answer comes from your documents first, with numbered points.
- **Sources** under the answer show each file the answer cites, once, with the numbers of the points that came from it. Greetings and questions that have nothing to do with your documents get a normal answer with no sources. Hover a source for the pages or sections; click it to open the file.
- If a set is still being read, the assistant tells you to try again shortly instead of guessing.

### Look after your sets

On the Knowledge page, each set shows its folders and files and how far reading has gone:
- **Add folder** and **Add files** to a set;
- **Rename**;
- **Check for new and changed files** (the refresh button); TUNAS also checks every set each time it starts;
- **Stop reading** pauses a set that is being read;
- **Delete this knowledge set** (your files stay).

**Folder permissions** (on the same page) lists every folder TUNAS may read, and which sets use it. **Remove** takes a folder away from TUNAS and from every set that uses it.

### Good to know

- **Readable files:** PDF (by page), Word, PowerPoint (by slide), Excel (by sheet), Markdown (by section), text, CSV, JSON, log and HTML. Scanned PDFs need text in them.
- **Cloud drive folders** (Google Drive, OneDrive and similar synced folders) work. Files kept only online are downloaded by the drive's app the first time TUNAS reads them, which takes longer.
- **Reading runs in the background** and pauses while an answer is being written, so the computer stays responsive. On a machine running TUNAS on its graphics card, reading is much faster.
- **Exact references** such as "PK 3/2026" or a version number are searched as exact words as well as by meaning.

## 8. Documents: Word, Excel and PDF

- **Ask for the document you want** ("Tulis surat rasmi…", "Buat laporan PDF tentang…"): the answer is the document itself, ready to save.
- **Turn an earlier answer into a document** with a short request such as "jadikan laporan PDF" or "make it a Word document": the assistant writes it out again in full as a document.

- When an answer looks like a document (a letter, minutes, a report), a card says **Your document is ready**, with **Word** and **PDF** buttons.
- Any answer can be saved with **Save as Word, PDF or Excel** under it.
- **Word and PDF** have your letterhead (logo, organisation name and address from Branding) at the top.
- **Excel** turns the answer's tables into sheets, with numbers kept as numbers.
- Citation numbers such as [1] are left out of the saved file.
- **The sender:** letters are written from you and your organisation, never from the assistant. Fill in the letterhead organisation and address in **Settings › Branding › More options**; without them, the assistant writes placeholders such as [Nama Jabatan] for you to fill in.
- **File name:** taken from the document's heading or a letter's subject line (PERKARA, Subject).
- Choose where to save it; TUNAS then opens the saved file.

**With a TERAS model,** TERAS makes the file itself (section 3, Chatting with a TERAS model), with more formats and your organisation's templates; switch on **Rich document** for a cover, charts and checked pages. TUNAS's own card appears only if TERAS gives no file.

Small models can make mistakes in names, addresses and dates. Check every document before sending it.

## 9. Models

**Settings › Models** (the gear at the bottom of the side bar).

- **Check my machine:** memory, processor, graphics and free space, with the recommended models. **Check again** after changing hardware.
- **Test speed:** how fast the default model writes on this machine, on whatever Run AI on is set to. Switch Run AI on and test again to compare processor and graphics.
- **Catalogue:** Download, Delete, and **Make default** (the model new chats use). Each model shows its thinking settings.
- **Import from a folder or USB stick:** copies catalogue models from a folder, checking each one. **Prepare a USB stick** (Copy models to…) copies your installed models out for another computer.
- **Your own models:** advanced users can add any GGUF model file.

**Settings › General:**
- **Default model**;
- **Run AI on:** Automatic (graphics when they help, processor otherwise), Graphics or Processor. On some machines with built-in graphics the processor is faster; compare with Test speed;
- **Engine:** what is running now, and **Stop engine**;
- **Appearance:** Light or Dark;
- **Data folder:** where TUNAS keeps your chats, models and settings, with **Open**.

## 10. Branding: your name and logo

**Settings › Branding.**

- **Choose the layer:** My own brand, or Organisation or department. (OEM or reseller is set on the machine's image and can't be changed here. "Powered by S.A.L.T CI" is always shown.)
- **Fill in:** Name, Logo (PNG or SVG; square works best), Colour, Welcome line. **More options:** the assistant's name, the logo's background and shape, and the letterhead's organisation and address.
- **Also use as the app icon:** puts your logo on the taskbar, desktop and Start menu icons. When off, the TUNAS icon stays and the logo shows inside the app.
- **The live preview** shows the window and the desktop icon as you type. Press **Save branding**.
- **Import branding pack:** open a .tunasbrand file from your organisation (or double-click the file in Windows).
- **Branding packs › Make a pack…:** save your brand as a .tunasbrand file to give to others, with the assistants you pick, your name as "Made by", and optionally the organisation's rules.

If the taskbar still shows the old icon after a change, unpin TUNAS and pin it again; Windows keeps its own copy of icons.

## 11. Connections

**Settings › Connections.**

### Your organisation's TERAS
1. **Connect to TERAS:** type the **TERAS address** your IT department gave you (for example teras.myorg.gov.my) and your **TERAS key**.
2. The organisation's TERAS models appear. Use them as an assistant's base model or pick them in a chat. **Refresh models** fetches the list again.

### Cloud AI providers
1. Switch on **Allow cloud providers** (if your organisation allows it).
2. **Add a provider:** choose the provider, paste your own API key, and tick the models your assistants may use.
3. Cloud answers are made on the provider's servers: the chat shows "Online · leaves this machine". Use them only for material that may leave the computer.

### Apps that use TUNAS
For S.A.L.T desktop apps and other programs that should use TUNAS as their AI:
1. Switch on **Let other apps use TUNAS**. The **Address for apps** is shown (http://127.0.0.1:11435/v1 by default).
2. **Make key** for each app, give it a name, and copy the key straight away: TUNAS will not show it again.
3. In the other app, use the address as the OpenAI base URL and the key as the API key. The app can ask for an assistant by its short name (for example `surat-rasmi`), a local model (for example `qwen3.5-4b`) or a connected model (for example `teras/<model-name>`).
4. The list shows when each key was last used. **Remove** a key to stop that app.

Only programs on this computer can reach it.

## 12. Questions and answers

**The answer is slow.**
Use a smaller model (Qwen 3.5 2B or 0.8B), turn thinking off, or set Run AI on to Graphics if the computer has a graphics card. The first answer after starting is slower while the model loads.

**Jawapan lambat.**
Guna model lebih kecil (Qwen 3.5 2B atau 0.8B), matikan thinking, atau tetapkan Run AI on kepada Graphics jika komputer ada kad grafik.

**The assistant answers from general knowledge, not my documents.**
Check that the set shows Ready on the Knowledge page and that it is in the assistant's Knowledge field, or pick it with #. Ask with words that appear in the documents.

**Pembantu tidak menjawab daripada dokumen saya.**
Pastikan set itu Ready di halaman Knowledge dan ada dalam medan Knowledge pembantu, atau pilih dengan #.

**Reading a folder takes very long.**
Large PDFs and files kept only in a cloud drive take longest. Reading pauses while you chat. On a computer with a graphics card, set Run AI on to Graphics for faster reading.

**TERAS's file opens in the browser instead of downloading.**
TERAS asked for your sign-in for that file. Sign in to TERAS in the browser and the download starts.

**Thinking goes on for a long time.**
Set How long to think to short, or switch to On, hidden or Thinking off.

**The taskbar shows another icon.**
Unpin TUNAS from the taskbar and pin it again; Windows keeps old icons in a cache.

**A cloud provider or the app keys are greyed out.**
Your organisation has turned them off on this computer.

**Bolehkah TUNAS digunakan tanpa internet?**
Boleh. Selepas model dimuat turun atau disalin dari USB, TUNAS berfungsi sepenuhnya tanpa internet.

**Where are my chats kept?**
In the Data folder on this computer (Settings › General › Data folder). They are not sent anywhere.

---
S.A.L.T CI · Owned by S.A.L.T. CI TECHNOLOGY · Distributed by Pemangkin Teknologi · www.salt.ci · info@salt.ci · www.pemangkin.tech · info@pemangkin.tech
S.A.L.T CI and its components are proprietary technology of S.A.L.T. CI TECHNOLOGY, distributed and commercialised by Pemangkin Teknologi.
