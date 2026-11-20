# TUNAS to TERAS: files for apps, events, and tool retries in answers

S.A.L.T CI · Owned by S.A.L.T. CI TECHNOLOGY · Distributed by Pemangkin Teknologi · 20 November 2026
For: the TERAS session. From: the TUNAS session, after CAPT ZEDs' test of TUNAS with the TERAS model `eltee`.

## 1. What TUNAS does now (0.4.0)

- TUNAS's chat calls `POST /v1/chat/completions` with the person's personal API key, `stream: true` and `"teras": {"events": true}`.
- It shows `tool.start` and `tool.done` as steps in the answer ("Making the document…", "failed, retrying"), and lists the `sources` event beside its own sources.
- It finds files in the answer text by their link, `[name](/api/files/<id>/<name>) · <size>`, and shows each as a download card. **Download** fetches `<TERAS address>/api/files/<id>/<name>` with `Authorization: Bearer <personal API key>`. On 401 or 403 it opens the same address in the browser, where the person signs in.
- TUNAS's own local knowledge passages are numbered from [51] when the model is a TERAS model, so TERAS's [1], [2]… are never confused with them.
- TUNAS no longer sends `temperature` or other sampling settings to TERAS (0.3.4): `eltee` answered "`temperature` is deprecated for this model".

## 2. Questions for TERAS

1. **Files with a personal API key.** Does `GET /api/files/<id>/<name>` accept the personal API key the file's owner made? If not, can it, for files made in that person's own chats? This is what makes the download card work without the browser.
2. **Files as an event.** Can TERAS send each answer file as an event too, for example `{"event": "file", "id", "name", "size", "mime", "url"}` (or in `answer.done`)? Reading markdown links from the answer text works today but breaks if the model words it differently.
3. **The `sources` event shape.** Please send one real example. TUNAS reads `n` / `index`, `title` / `document` / `name`, `page`, `collection` and `url` if present.

## 3. Things seen in the test (for TERAS to look at)

- **`create_document` failed three times** before it worked. The model's own words in the answer: "Panggilan tadi gagal kerana skripnya kosong", then "format argumennya rosak", then "Argumen yang dihantar rosak dan tiada kandungan sampai ke alat". The fourth call made the PDF (328.7 KB). The tool's arguments seem to arrive empty or broken on the first tries.
- **The retries are narrated into the answer.** The person sees four paragraphs about failed calls before the result. Apps can now show tool steps from the events; the answer itself would read better without the narration.
- **"PDF belum diperiksa secara visual. Alat render saya tidak dipanggil"**: the model said the render check was not run, so the layout was not checked.
- **The answer says "Files" with a relative link** (`/api/files/…`). Inside an app, a relative link goes nowhere; an absolute address, or the file event in question 2, would help every app.

## 4. Nothing needed from TERAS for TUNAS 0.4.0 to work

TUNAS 0.4.0 works with TERAS as it is today: steps, sources and file cards show, and a refused download falls back to the browser. The answers to section 2 only make it smoother.

---
S.A.L.T CI · Owned by S.A.L.T. CI TECHNOLOGY · Distributed by Pemangkin Teknologi · www.salt.ci · info@salt.ci · www.pemangkin.tech · info@pemangkin.tech
S.A.L.T CI and its components are proprietary technology of S.A.L.T. CI TECHNOLOGY, distributed and commercialised by Pemangkin Teknologi.
