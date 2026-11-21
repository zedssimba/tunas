# TUNAS trainer card: connect a cloud provider

S.A.L.T CI · Owned by S.A.L.T. CI TECHNOLOGY · Distributed by Pemangkin Teknologi
TUNAS trainer kit · class exercise · about 10 minutes · TUNAS 0.5.2

**What students learn:** TUNAS can use an AI model on the internet as well as the one on the computer. Any service that speaks the common OpenAI-style API can be plugged in with just two things: an address and a key. They also see the difference between "Offline & private" and "Online · leaves this machine".

**Before class:** each student needs a phone number for the provider's sign-up check. Give each student their own key: free keys allow only about one request per second, so a shared key fails when the class sends at once.

## Step 1: get a free key (example: Mistral)

1. Open **console.mistral.ai** and sign up.
2. Choose the free **Experiment** plan and verify your phone number.
3. Open **API Keys**, make a new key and copy it. It is shown only once.

## Step 2: add it to TUNAS

1. **Settings › Connections**, then switch on **Allow cloud providers**.
2. **Add a provider**, and for **Provider** choose **Other (OpenAI-compatible)**.
3. Fill in:

   | Field | Type this |
   |---|---|
   | Name | Mistral |
   | Address (OpenAI-compatible base URL) | `https://api.mistral.ai/v1` |
   | API key | the key from step 1 |

4. **Save and load models**, then tick one chat model (the search box helps), for example `mistral-small-latest`. Don't tick models with "embed" or "moderation" in the name; they can't chat.

## Step 3: try it

1. Start a new chat and pick the Mistral model from the model chip at the top.
2. The badge now says **Online · leaves this machine**. Ask the same question with a local model and compare: **Offline & private**.
3. Discuss: which answer was faster? Which one could you use for a real office document?

## Rules to say out loud

- **Practice only.** On the free plan, the provider may use what you send to train its models. Never send real office documents, student records or anything internal.
- **The key is personal.** Don't share it or put it in a screenshot. Remove it in Connections after class if the computer is shared.
- **Too many requests?** Wait a few seconds and send again; that is the free plan's limit, not TUNAS.
- **Your organisation can switch cloud off.** A policy can block cloud providers; the Connections page then says so.

## Close the lesson: the TERAS way

Typing an address and a key is how any outside service is connected. In an organisation, students would not do this at all: IT gives one **TERAS** address and key, and TUNAS shows the organisation's own models, knowledge and Rich documents. Everything stays on the organisation's servers. That is what **Connect to TERAS** is for.

---
www.salt.ci · info@salt.ci · www.pemangkin.tech · info@pemangkin.tech
S.A.L.T CI and its components are proprietary technology of S.A.L.T. CI TECHNOLOGY, distributed and commercialised by Pemangkin Teknologi.
