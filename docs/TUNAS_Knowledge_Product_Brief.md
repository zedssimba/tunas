# S.A.L.T TUNAS: Product Brief

TUNAS 0.5.2 · Product brief 1.0 · November 2026 · https://www.salt.ci
S.A.L.T CI · Owned by S.A.L.T. CI TECHNOLOGY · Distributed by Pemangkin Teknologi

This is the knowledge copy of the TUNAS Product Brief for AI knowledge bases (TUNAS 0.5.2). How to use TUNAS is in the TUNAS User Guide; how to roll it out across an organisation is in the TUNAS Administrator Guide; what changed in each release is in the TUNAS Release Notes.

## 1. What TUNAS is

S.A.L.T TUNAS is the lightweight edition of TERAS, the core of the S.A.L.T CI framework, made to run on a person's own notebook or PC, locally and privately. It runs AI models on the computer itself, so chats, documents and answers stay on the machine and work without the internet.

- **On its own or with TERAS:** TUNAS works fully on its own, and connects to the organisation's TERAS when there is one.
- **The AI engine for S.A.L.T CI desktop apps:** apps such as NEXUS, WARKAH, NASKAH, Kembara and MOM Studio can use TUNAS as their private, personal AI engine on the same computer.
- **Small, yet capable:** light enough for an everyday notebook, and capable enough for daily work such as writing, summarising and finding answers in the user's own documents. It gives each person their own AI, with their own documents and assistants, on their own machine.

- **What it does:** chat in Bahasa Melayu, English or both; ready-made assistants for letters, minutes, emails and summaries; answers from the user's own folders with their sources; and finished Word, Excel and PDF documents with the organisation's letterhead.
- **Who it is for:** staff in agencies, universities and companies who handle internal documents; lecturers, students and AI classes; trainers running workshops; and notebook makers, leasing companies and resellers who want a co-branded AI app on every machine they ship.
- **How it is used:** installed like any app on Windows, macOS, Linux or Android, with no account and no administrator rights. It picks the right models for the machine, downloads them or copies them from a USB stick, and opens into a chat.
- **What it costs:** TUNAS is free to install, with no licence key, activation or expiry. OEM, leasing and reseller arrangements go through Pemangkin Teknologi.

The name: *tunas* is Malay for a young shoot, the first growth of a plant. TUNAS is where people start with S.A.L.T CI, on their own computer, before growing into TERAS, the organisation's own AI server.

## 2. TUNAS chat

- **Languages:** TUNAS replies in the language of the question and handles mixed Malay and English the way people really write. The interface is in English.
- **Files in the chat:** drop in a PDF, Word (.docx), Excel (.xlsx, .xlsm, .xls), PowerPoint (.pptx), text, Markdown, CSV, JSON, log or HTML file, and TUNAS reads it on the computer and uses it in the answer.
- **Streaming answers:** answers appear word by word as the model writes them.
- **Per-chat choices:** at the top of each chat, the assistant, the model and the thinking mode can be changed.
- **History:** chats are saved on the computer, titled automatically, and searchable from the side bar.
- **TERAS at work:** with a TERAS model, the chat shows each tool step as it happens (making a document, searching the web), lists TERAS's own sources, and gives files TERAS makes a download card.
- **Where the answer is made:** the chat shows "Offline & private" for local models, "Your organisation's server" for TERAS, and "Online · leaves this machine" for cloud models.

## 3. TUNAS About me

- **A short brief about the person** (Settings › About me): what to call them, position, organisation, what they work on, languages and how they like answers.
- **Sent with each message** to local and cloud models, so answers fit the person and documents can be signed for them; TERAS models use the person's own TERAS personalisation instead.
- **Kept on the computer only,** with a switch to turn it off.

## 4. TUNAS thinking

Models that can reason (the Qwen 3.5 models) can think before they answer.

- **Three modes per model:** Thinking off, Thinking hidden (the model thinks, only the answer shows), and Thinking shown (the reasoning appears above the answer, with the time it took).
- **Thinking length per model:** short, normal or long. The length keeps the reasoning focused and makes sure an answer always follows.
- **The choice is kept per model** across all chats, and can be changed from the chat's thinking chip or the Models page.
- **Use in teaching:** shown thinking lets a lecturer or trainer walk a class through how an answer was reached.

## 5. TUNAS assistants

TUNAS comes with seven ready-made assistants:

| Assistant | What it does |
|---|---|
| General Assistant | Everyday questions, explanations and drafting, in Malay or English |
| Surat Rasmi | Formal letters in Bahasa Melayu, in the government style |
| Official Letter | Formal letters in English |
| Minit Ringkas | Minutes and action lists in Bahasa Melayu |
| Meeting Minutes | Minutes and action lists in English |
| Email Writer | Clear emails in the right tone from a few points |
| Summariser | The key points of a long document or thread |

- **Each assistant has:** a name, an icon and colour, a persona (how it writes and what it asks), starter prompts, the knowledge sets it answers from, and a base model (local, TERAS or cloud).
- **Users make their own** for any task, unless the organisation's policy turns this off.
- **Shared by branding packs:** an organisation's assistants travel with its branding pack, so every computer has the same set.

## 6. TUNAS Knowledge

TUNAS reads the user's folders where they already are, with permission, and answers from them with their sources. Nothing is uploaded or copied elsewhere.

- **Folder permissions:** the user chooses a folder and confirms what TUNAS may read. TUNAS reads only allowed folders, never changes or deletes a file, and forgets a folder when its permission is removed. Synced cloud-drive folders work too; files kept only in the cloud are fetched by the drive's own app the first time they are read.
- **Knowledge sets:** one or more folders or files grouped under a name, such as "Pekeliling & SOP Jabatan".
- **Using a set:** an assistant answers from the sets in its Knowledge field; in any chat, type **#** to pick a set for that chat. The chat shows "Answering from" with the sets in use.
- **Keeps up to date:** each time TUNAS starts, and whenever the user presses the refresh button on a set ("Check for new and changed files"), files added, changed or removed in its folders are read again in the background.
- **Search:** by meaning and by exact words together, so reference numbers, versions and names are found as reliably as ideas.
- **Whole documents:** PDFs are read page by page, PowerPoint slide by slide, Excel sheet by sheet and Markdown section by section, so an answer keeps its place in the document.
- **Sources:** each answer shows one chip per file with the numbered points behind it; a click opens the file.
- **Documents come first:** when the user's documents speak to a question, they take priority over anything else the assistant knows, including the built-in S.A.L.T reference.
- **TERAS knowledge:** when connected to TERAS, the organisation's TERAS knowledge collections that the user may see appear beside the user's own knowledge sets, searched on the TERAS server.
- **Readiness:** if a set is still being read, the assistant says so instead of guessing.
- **Quiet in the background:** the reading model uses the graphics card when TUNAS runs on it, uses all processor cores at low priority otherwise, and pauses while a chat is answering.
- **The reading model:** Qwen 3 Embedding 0.6B, downloaded automatically the first time a knowledge set is made.

## 7. TUNAS Documents

When an assistant writes a letter, minutes, a report or a table, TUNAS offers it as a finished file.

- **Formats:** Word (.docx), Excel (.xlsx) and PDF.
- **Letterhead:** the logo, organisation name and address from the app's branding, at the top of Word and PDF documents.
- **Excel:** tables become sheets, with numbers kept as numbers.
- **Clean copy:** citation markers such as [1] are removed from the saved document; the sources stay in the chat.
- **The document itself:** when the user asks for a document, or to turn earlier content into one, the answer is the complete document, and chat around it is left out of the saved file.
- **From the user, not the AI:** letters and documents are written from the user and their organisation, using the Branding letterhead as the sender, or placeholders such as [Nama Jabatan] when it is not set. The assistant never signs as itself.
- **Richer with TERAS:** on the notebook alone, TUNAS makes clean Word, Excel and PDF files. Connected to TERAS, the same request is made by TERAS: PowerPoint, CSV and Markdown as well, the organisation's own document templates and branding, and Rich documents with a cover page, charts from the data and every page checked. The notebook sets the limit; TERAS lifts it.
- **Where:** "Your document is ready" appears under answers that look like documents, and **Save as Word, PDF or Excel** is on every answer. After saving, open the file or its folder.

## 8. TUNAS models and engine

- **Check my machine:** reads memory, processor and graphics, runs a short speed test, and recommends models that will run well.
- **The catalogue:**

| Model | Good for | Memory |
|---|---|---|
| Qwen 3.5 0.8B | Older notebooks; simple, quick tasks | 4 to 6 GB |
| Qwen 3.5 2B | The everyday default; best for 8 GB without a graphics card | 6 GB or more |
| Qwen 3.5 4B | Better writing and reasoning, with thinking | 8 GB usable, 12 GB recommended |
| Granite 4.2 3B | Steady, business-style answers; a second opinion | 8 GB |
| SEA-LION v4 4B | Tuned for Southeast Asian languages including Malay; the user downloads it and checks its own licence | 8 GB |
| Qwen 3 Embedding 0.6B | Reading documents for Knowledge | 4 GB |

- **Downloads are checked** against their published checksums before use.
- **Offline:** import models from a folder or USB stick, or copy installed models out to prepare another machine.
- **Your own models:** advanced users can add any GGUF model file.
- **Run AI on:** Automatic, Graphics or Processor. Graphics are used through Vulkan (Metal on Mac) where they are faster; the processor is always the fallback.
- **One model at a time:** chats and other apps take turns on the same loaded model, so the computer never loads two chat models at once.

## 9. TUNAS branding

TUNAS takes the name, logo and colours of whoever gives it out.

- **Where the brand shows:** the title bar, the welcome screen, the app name in the side bar, the desktop and Start menu shortcuts, the letterhead of saved documents and the About page. The taskbar and shortcut icon can also use the logo, if "Also use as the app icon" is ticked.
- **What a brand holds:** name, logo (PNG or SVG), logo background and shape, colours, welcome line, the assistant's name, and the letterhead organisation and address.
- **Layers, most specific shown first:**

| Layer | Who sets it | Typical use |
|---|---|---|
| My own brand | The user | A trainer's or consultant's own name and logo |
| Organisation or department | The user, a branding pack, or IT for the whole machine | An agency, faculty or company copy |
| OEM or reseller | The machine's image, by the computer maker or reseller | Pre-installed notebooks; users can't remove it |
| Always shown | S.A.L.T CI | "Powered by S.A.L.T CI" and credit on the About page |

- **Branding packs:** one signed .tunasbrand file carries the brand, the logo, optionally the organisation's assistants, and optionally the organisation's rules. Double-click it, or use Import branding pack.
- **Organisation rules:** a pack or a machine policy file can lock the organisation brand, stop personal brands, stop new assistants, and turn off cloud providers, cloud use with knowledge, or the local AI API.

## 10. TUNAS connections

Everything in TUNAS works on the computer alone. Connections adds more, when the user or the organisation chooses.

- **Connect to TERAS:** the organisation's TERAS address and a key. The organisation's TERAS models and knowledge collections then appear by name, and any assistant can use them.
- **Cloud AI providers:** the main cloud AI providers, or any OpenAI-compatible service, with the user's own key. Off until the user switches on "Allow cloud providers", and the organisation can block it.
- **Let other apps use TUNAS (local AI API):** the private AI engine for S.A.L.T CI desktop apps such as NEXUS, WARKAH, NASKAH, Kembara and MOM Studio, and for other programs the user allows. An OpenAI-compatible API (`/v1/models`, `/v1/chat/completions` with streaming) on this computer only (127.0.0.1, port 11435 by default). Each app gets its own key from TUNAS; keys can be removed at any time and the list shows when each was last used. Apps can ask for an assistant by name (its persona and knowledge apply) or for a model. Off by default.
- **Built-in S.A.L.T reference:** TUNAS knows the basic facts about S.A.L.T CI and its products, so it can answer questions about TERAS and the platform. The user's own documents always come first.

## 11. TUNAS privacy and security

- **On the device:** chats, assistants, knowledge indexes, branding and saved documents stay on the computer.
- **Offline:** once installed with its models, TUNAS needs no internet and no account.
- **Read-only folders:** TUNAS only reads folders the user allowed and never changes or deletes the user's files.
- **Keys protected:** TERAS and cloud keys are kept with the operating system's own protection (DPAPI on Windows), tied to the user's account. Local API keys are stored only as a hash.
- **Off by default:** cloud providers and the local AI API.
- **Local only:** the local AI API listens on this computer only, not the network.
- **No usage data:** TUNAS sends no content and no usage reports anywhere.
- **Checked models:** downloaded and imported catalogue models are checked against their checksums.

## 12. How TUNAS is built

| Part | Technology |
|---|---|
| App | Tauri 2 desktop app: a Rust core with a Svelte interface |
| AI engine | llama.cpp, shipped with the app: processor and Vulkan graphics builds, and Metal on Apple silicon |
| Models | GGUF model files, kept in the user's TUNAS data folder |
| Chats and settings | A local SQLite database |
| Knowledge | A separate local database with full-text and vector search; a second engine runs the reading model |
| Documents | Word, Excel and PDF written by TUNAS itself, with bundled fonts |
| Local AI API | An OpenAI-compatible gateway on 127.0.0.1 |
| Installers | Per-user installers for Windows, macOS (signed and notarised) and Linux, and an Android app; silent install with a settings file |

## 13. Deploying TUNAS

| Item | Details |
|---|---|
| Devices | Windows 10 and 11 (64-bit); macOS on Apple silicon and Intel; Linux (AppImage and .deb); Android phones and tablets |
| Memory | 8 GB recommended; 4 GB runs the smallest model |
| Graphics | Optional; used where faster, with the processor as fallback |
| Disk | About 2 to 4 GB per model, plus the app |
| Install | Per user, with no administrator rights; or silent, with one settings file that presets the brand, models, TERAS address and policy |
| Updates | A quiet in-app update check, or an offline update file for networks without internet |
| Models | Downloaded in the app, or imported from a folder or USB stick |
| Organisation set-up | A branding pack, or machine-wide brand and policy files placed by IT |
| TERAS | Optional; address and key in Connections |

## 14. TUNAS trainer kit

For workshops and trainings, the TUNAS trainer kit is one folder with the installers, the models on USB, branding packs and a handout, so every seat runs its own AI without the internet. It is arranged through Pemangkin Teknologi.

## 15. TUNAS limits

| Limit | Details |
|---|---|
| One person, one computer | Sharing AI and knowledge across people is what TERAS does |
| Local API on the same computer | Other computers on the network can't reach it |
| Interface in English | Chats and assistants work in Malay and English |
| Android | The local AI API for other apps is on Windows, macOS and Linux |
| Installer checks | Some antivirus tools scan or delay the installer the first time |
| Small models | Answers from small local models can be wrong; check important facts |

## 16. Questions and answers

**Does TUNAS need the internet?**
Only to download models. After that it works fully offline, or models can be copied from a USB stick.

**Adakah TUNAS memerlukan internet?**
Hanya untuk memuat turun model. Selepas itu ia berfungsi sepenuhnya tanpa internet, atau model boleh disalin daripada pemacu USB.

**Are my documents sent anywhere?**
No. TUNAS reads allowed folders on the computer and answers on the computer. Only if an assistant uses TERAS or a cloud model are the question and the relevant passages sent to that service.

**Adakah dokumen saya dihantar ke mana-mana?**
Tidak. TUNAS membaca folder yang dibenarkan dan menjawab di komputer itu sendiri. Hanya jika pembantu menggunakan TERAS atau model awan, soalan dan petikan berkaitan dihantar ke perkhidmatan itu.

**Is TUNAS really free?**
Yes, with no licence key, activation or expiry. Co-branded OEM, leasing and reseller images are arranged through Pemangkin Teknologi.

**What is the difference between TUNAS and TERAS?**
TUNAS is the lightweight edition of TERAS, for one person on one computer. TERAS is the organisation's own AI server, shared by everyone, with team knowledge, permissions, sign-in, audit and the S.A.L.T apps. Connect to TERAS links the two.

**Apakah beza TUNAS dan TERAS?**
TUNAS ialah edisi ringan TERAS, untuk seorang pengguna pada satu komputer. TERAS ialah pelayan AI organisasi sendiri yang dikongsi semua, dengan pengetahuan pasukan, kebenaran, log masuk, audit dan aplikasi S.A.L.T. Connect to TERAS menghubungkan kedua-duanya.

**Can S.A.L.T CI desktop apps use TUNAS?**
Yes. NEXUS, WARKAH, NASKAH, Kembara, MOM Studio and other S.A.L.T CI desktop apps can use TUNAS as their private AI engine through its local AI API, with a key made in TUNAS.

**Bolehkah aplikasi desktop S.A.L.T CI menggunakan TUNAS?**
Boleh. NEXUS, WARKAH, NASKAH, Kembara, MOM Studio dan aplikasi desktop S.A.L.T CI lain boleh menggunakan TUNAS sebagai enjin AI peribadi mereka melalui API AI tempatan, dengan kunci yang dibuat dalam TUNAS.

**Can we put our own name and logo on it?**
Yes, in Settings › Branding, or for a whole department with a branding pack.

**Which model should I use?**
Check my machine recommends one. On most 8 GB notebooks, Qwen 3.5 2B is the everyday choice; Qwen 3.5 4B writes better where there is enough memory or a graphics card.

## 17. Words used

| Word | Meaning |
|---|---|
| Assistant | A helper with its own persona, starters, knowledge and model |
| Knowledge set | Folders or files grouped under a name, that assistants answer from |
| Folder permission | A folder the user allowed TUNAS to read |
| Branding pack | A .tunasbrand file with a brand, assistants and rules |
| Layer | One of the brands on a copy: personal, organisation, OEM |
| Local AI API | TUNAS's AI engine offered to other apps on the same computer |
| TERAS | S.A.L.T CI's organisation-wide AI server |
| Engine | llama.cpp, the program that runs the models |
| GGUF | The model file format TUNAS uses |

---
S.A.L.T CI · Owned by S.A.L.T. CI TECHNOLOGY · Distributed by Pemangkin Teknologi · www.salt.ci · info@salt.ci · www.pemangkin.tech · info@pemangkin.tech
S.A.L.T CI and its components are proprietary technology of S.A.L.T. CI TECHNOLOGY, distributed and commercialised by Pemangkin Teknologi.
