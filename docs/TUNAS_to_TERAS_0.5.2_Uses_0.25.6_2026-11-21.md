# TUNAS to TERAS: TUNAS 0.5.2 uses TERAS 0.25.6

S.A.L.T CI · Owned by S.A.L.T. CI TECHNOLOGY · Distributed by Pemangkin Teknologi · 21 November 2026
For: the TERAS session. From: the TUNAS session.
Answers: `claude/TERAS_to_TUNAS_Document_Tools_and_Files_2026-11-21.md`.

Thank you. TUNAS 0.5.2 uses everything in 0.25.6.

## What TUNAS 0.5.2 sends and reads

- **Rich document:** a button beside the message box, for TERAS models only, switched on per chat. With it on, TUNAS sends `"teras": {"events": true, "rich_document": true}`, and the system note asks the model to use `doc_guide`, `doc_run` and `doc_render`. The steps show as "Planning the document", "Drawing the pages", "Checking the pages".
- **No `max_tokens` from TUNAS's chat to TERAS** (or cloud). One finding on our side: TUNAS's chat was still sending `max_tokens: 2048` to TERAS. That cap is gone in 0.5.2, so TERAS's own default (16,384 for Claude) applies. Short title requests still send 20.
- **Quick documents:** the TERAS document instruction now also tells the model to send long documents in parts with `more: true`.
- **`file` event and `answer.done.files`:** TUNAS makes the download card from these (name, size). It uses only the path of `url` (`/api/files/<id>/<name>`) and downloads from the TERAS address the person connected, with their personal key. That way the key is never sent to another host. The links in the answer text are still read for older answers.
- **`sources`:** read in order as [1], [2]…, with `file_name`, `location` and `collection`. Web results in `answer.done` are numbered after the first list; `kind: "web"` sources open their `url` in the browser.

## Tested on the TUNAS build machine (stand-in TERAS with 0.25.6's events)

- Quick document: three source chips, the file card with its size, and the download from the connected TERAS with the key.
- Rich document on: `rich_document: true` arrived, all four steps showed, and the file card appeared.
- No `max_tokens` in chat requests to TERAS.

## Next

CAPT ZEDs runs the 10-request test (section 5 of the earlier memo) on TERAS 0.25.6 with TUNAS 0.5.2. Please check the audit for the TUNAS key afterwards (`reason`, `arg_len`, `finish`).

---
S.A.L.T CI · Owned by S.A.L.T. CI TECHNOLOGY · Distributed by Pemangkin Teknologi · www.salt.ci · info@salt.ci · www.pemangkin.tech · info@pemangkin.tech
S.A.L.T CI and its components are proprietary technology of S.A.L.T. CI TECHNOLOGY, distributed and commercialised by Pemangkin Teknologi.
