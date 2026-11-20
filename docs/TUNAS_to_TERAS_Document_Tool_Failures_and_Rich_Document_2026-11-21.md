# TUNAS to TERAS: document tool failures, and Rich document for apps

S.A.L.T CI · Owned by S.A.L.T. CI TECHNOLOGY · Distributed by Pemangkin Teknologi · 21 November 2026
For: the TERAS session. From: the TUNAS session, after the MIMPI report of 21 November.

## 1. Why this matters

CAPT ZEDs wants to demo TUNAS to customers in two steps:

- **Local model on the notebook:** a clean, basic PDF or Word file, made by TUNAS itself, offline.
- **Connected to TERAS:** a better document, made by TERAS: more formats, the organisation's templates and branding, and Rich documents (cover, charts, pages checked).

The message to the customer: the notebook sets the limit, not S.A.L.T CI. For that demo, TERAS's document tools must work first time from TUNAS.

## 2. What MIMPI and the audit log show

- MIMPI (21 Nov): `create_document` 7 errors and `doc_run` 6 errors in a day, both at their daily average. So this is a steady problem, not a new one.
- About **13** `create_document` failures so far came from the API key named **TUNAS** (model `eltee`).
- One audit line, 20 Nov 11:06 UTC:
  `route POST /v1/chat/completions · key_name TUNAS · model eltee · action tool_call · detail {"tool": "builtin:create_document", "ms": 0, "ok": false} · status 500`
- `"ms": 0` means the tool stopped before doing any work. In the same chat the model said the arguments arrived empty, then broken ("skripnya kosong", "format argumennya rosak", "tiada kandungan sampai ke alat"). The fourth call made the PDF.
- Our reading: the model sends the whole document as one long string inside the tool's JSON arguments, and on long documents those arguments arrive cut off or not valid JSON. TERAS rejects them at once.
- `doc_run` failures cannot come from TUNAS (the sandbox only starts from the Console's Rich document button). They come from Console users.

## 3. What TUNAS changed (commit b7b351b, in the next TUNAS release)

Until now TUNAS sent its own document instruction to every model, including TERAS models: "TUNAS turns your answer into a Word, PDF or Excel file … never say you cannot create files". For a TERAS model that fights `create_document`, and may be part of the cause.

TERAS models now get this instead, only when the person asks for a document:

> Make it with your document tool (create_document), choosing the format the user asked for (Word, PDF, PowerPoint, Excel, CSV or Markdown; Word if they did not say). Put the complete document in the tool's content as Markdown … Send the tool valid arguments with every quote and line break escaped … If a tool call fails, correct the arguments and call it again without describing the failure in your answer. When the file is made, answer with one or two short lines and the file link; do not repeat the document in the answer.

Local and cloud models keep TUNAS's own instruction. If TERAS gives no file, TUNAS still offers its own PDF/Word card from the answer text.

## 4. Asks for TERAS

1. **Log why a tool call failed.** Today the audit has only `ok: false`. Please add the reason (empty arguments, invalid JSON, missing `content`, too long, render error), the argument length received, and the model's finish reason for that turn (a cut-off `length` would explain broken JSON).
2. **Be forgiving with arguments.** Repair near-valid JSON (unescaped line breaks and quotes, a missing closing brace) before rejecting it, and accept `content` when the arguments arrive as a plain string.
3. **Tell the model what went wrong.** Send the model a short, specific error ("arguments were not valid JSON at character 4,812; send content again"), so the retry is right first time.
4. **Long documents.** If the cause is length, raise the output limit for tool-calling turns, or let `create_document` take the document in parts (a `part` / `append` argument).
5. **Rich document from apps.** Please let an app ask for the document sandbox on one request, for keys whose owner may use it, for example
   `"teras": {"events": true, "rich_document": true}` on `POST /v1/chat/completions`.
   Send `tool.start` / `tool.done` events for `doc_guide`, `doc_run` and `doc_render`, so TUNAS can show "Drawing the pages… checking page 3". TUNAS will then add a **Rich document** switch beside the message box, shown only for TERAS models.
6. **Templates (later, optional).** A way for an app to list the document templates the person may use and pick one per request (for example `"teras": {"document_template": "<id>"}`).
7. **Still open from the 20 Nov memo:** files downloadable with the personal API key, a `file` event (or an absolute file address), and one real example of the `sources` event.

## 5. How we'll test together

- Same prompt from TUNAS on `eltee`, 10 times, short and long documents (1, 5 and 15 pages), in Malay and English, Word, PDF and PowerPoint.
- Target: the file is made on the first call every time, no failure narration in the answer, and the audit shows `ok: true`.
- Then the same with `rich_document: true`.

---
S.A.L.T CI and its components are proprietary technology of S.A.L.T. CI TECHNOLOGY, distributed and commercialised by Pemangkin Teknologi.
www.salt.ci · info@salt.ci · www.pemangkin.tech · info@pemangkin.tech
