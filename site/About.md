# Draku: your files, finally understood

Oct 28, 2026 · @Devansh

## One sentence

Draku is a private AI that understands every screenshot, photo and PDF on your device. It finds what you need, warns you before a date passes, hides what should stay private, and lets go of what is dead. Nothing leaves your machine.

**Your files, finally understood.**

That sounds like four tools: a finder, a reminder, a bodyguard and a janitor. It is one. Each file is read once, and everything else follows from that reading.

## The moment

You are at the pharmacy counter. The prescription is somewhere in your phone, saved as IMG-20261011-WA0004.jpg. Everyone has a version of this: the warranty receipt at the service centre, the exam form that closed yesterday, the Wi-Fi password from a screenshot three months ago.

Screenshots are the most-used notebook nobody calls one. But a screenshot stores pixels, not meaning, so whatever matters inside it is invisible to you and to every app on your device:

- **A date**: the warranty that ends next week, the coupon that dies tonight.
- **A secret**: an ID number or password sitting in a gallery that any app with photo access can read.
- **A place**: files sent by email or as documents can carry hidden location and device details.

Those are the ghosts in your files. And the files with no ghost left, like last spring's OTP screenshot, quietly fill your storage until the phone says it is full.

## The idea: one reading, four outcomes

Draku finds the ghosts. It reads each file once and notes four things: what it is, when it matters, what is private in it, and whether it is still needed. Everything the product does is a view of that single reading.

&#91;embedded content: one reading, four outcomes · 1 file in, 4 results\]

Find, Remember, Protect and Let go are not four features. They are four answers to one question: what is this file, and what should happen to it?

| Outcome | Before | After |
| --- | --- | --- |
| Find | IMG-20261011-WA0004.jpg | 2026-10-11 Dr Patel blood test prescription.jpg, found by typing "blood test" |
| Remember | A warranty receipt screenshot, one week from expiry | A calendar reminder, set with one tap |
| Protect | An Aadhaar photo with all 12 digits visible and a GPS tag inside | First 8 digits blurred, GPS tag removed |
| Let go | Old OTPs, expired tickets, four copies of one photo | Set aside in a rest folder, restorable |

## One screen, no settings

The whole product is a drop zone and one screen. Drop a folder, or point Draku at Screenshots and Downloads, and when it has read them you see three cards and a search line. In the demo folder the cards read:

- **2 deadlines.** Laptop warranty ends in 4 days. Exam form closes tomorrow. One button: Add to calendar.
- **1 exposed secret.** A screenshot shows a password in a browser tab. One button: Make a safe copy.
- **9 files ready to let go.** Expired OTPs, old QR codes, near-identical photos. One button: Set aside.

Above the cards sits one line: "Ask anything." Type "Wi-Fi password from last month" and the right screenshot comes back, cropped to the answer. Nothing changes on your device until you press a button.

## What we said no to

The vault half of the name is a promise, and these are its terms.

- **No upload.** Every file is read on your device. There is no cloud mode to switch on.
- **No deleting.** Files you let go move to a rest folder you can restore from. Only you empty it.
- **No guessing out loud.** Draku suggests and you confirm. A date becomes a reminder only when you tap it.
- **No account, subscription or ads.** It is free and open source.
- **No settings screen.** If a feature needs a setting, it is not finished.

## The 60-second demo

Four beats and one reveal. Record it on the laptop itself, with Wi-Fi visibly off.

| Time | On screen | You say |
| --- | --- | --- |
| 0:00 | Airplane mode on. A messy folder of 25 files: IMG\_0042.png, scan\_1(2).pdf, old screenshots. | "This folder is on every laptop. Airplane mode is on. Everything you are about to see happens right here." |
| 0:10 | Drop the folder. Cryptic names turn into clear ones as you watch. Type: laptop warranty. | "Draku reads each file once. Now every file has a name, and you can just ask." |
| 0:25 | A card appears: Laptop warranty ends in 4 days. Tap Add to calendar. | "That date was trapped inside a picture. Now it is in your calendar." |
| 0:35 | Pick a screenshot to share. A password in a browser tab and a hidden GPS pin light up. Tap Make a safe copy. | "Before you share it, you see what you would give away. One tap and it is gone." |
| 0:48 | The Let go card shows 9 files. Tap Set aside. | "Nine files nothing depends on. They rest in a folder you can restore from." |
| 0:55 | Cut back to the airplane-mode icon, still on. | "One more thing. That was airplane mode the whole time." |

## Why it has to be open

A product that reads your camera roll must run where the camera roll lives, and you must be able to check that it does. That roll holds IDs, prescriptions, tax forms and private photos.

- **Local needs open models.** PaddleOCR and EmbeddingGemma run on your own hardware, so reading a file never needs a server.
- **Open means checkable.** Anyone can read the code that reads their files and confirm that it never opens a network connection.
- **No storage ransom.** It costs nothing to run because the models run on your hardware, so a tool that frees your space never sells you more.

## Under the hood

Draku reads each file once — with local OCR, a rule-based reader, and an open embedding model — and every outcome works from that reading.

| Outcome | How it works | Models |
| --- | --- | --- |
| Find | Local OCR reads the pixels; a rule-based reader picks a name, category and key text. EmbeddingGemma embeds that reading, so a plain-language question finds the file. | PaddleOCR, EmbeddingGemma-300M |
| Remember | The same reading extracts dates and their meaning from the surrounding words. A future date becomes a calendar file once you confirm it. | Local rules |
| Protect | The reading flags ID numbers, passwords and card numbers. OCR gives exact pixel positions; blurs are burned into the image and location metadata is stripped. | PaddleOCR, EXIF |
| Let go | One row per file: age, category, date passed, duplicate distance, blur, size. A small weighted score turns those signals into a reason you can read. | Transparent local scoring |

Today Draku is a laptop app: drop a folder and everything runs locally. Next are a phone gallery extension and a watch folder that reads new screenshots as they arrive. For the write-up, PaddleOCR is the eyes, EmbeddingGemma is the memory, and the judgment stays a small set of rules anyone can read.

## Questions to be ready for

Six questions a sharp judge will ask, and answers that stay true.

- **Isn't this just Windows Recall?** Recall-style tools capture your screen over time so you can search it. Draku captures nothing: it reads only the files you choose, then acts on them. Check each rival's current status before you answer, because this space moves fast.

* **How do I know nothing leaves the device?** The demo runs in airplane mode, and the code is open so anyone can check.
* **Why not a cloud gallery?** Cloud galleries work by uploading your files. Draku never does, and it goes beyond search: it renames, reminds, redacts and clears.
* **What if it misses a secret?** It shows every detection before you share, and you confirm. Blurs are burned into the image, so they cannot be peeled off.
* **How does Draku decide what is dead?** It adds up a few signals you can inspect: age, category, whether a date has passed, duplicate distance, blur and size. No training run, no hidden model — just weights in the source, and a button you choose to press.
* **Is the hosted demo a contradiction?** The hosted demo and any voiced video use sample files only, so people can try it with zero setup. The real product runs on your device; say so plainly.
