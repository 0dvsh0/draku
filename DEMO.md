# Draku — The 60-Second Demo Guide

This guide details the exact steps and script for demonstrating Draku in a 60-second airplane-mode walk-through.

---

## 1. Environment & Setup

1. **Hardware / OS**: Windows 10/11 laptop.
2. **Network State**: **Airplane mode enabled** (Wi-Fi and Ethernet disconnected).
3. **App Build**:
   - Built with full offline bundled resources:
     ```pwsh
     cd app
     npm run tauri:release
     ```
   - Or launch the development build:
     ```pwsh
     cd app
     npm run tauri dev
     ```
4. **Demo Directory**: Prepare an 8-file folder containing sample real-world documents:
   - `IMG_0042.png` — Router settings screenshot with Wi-Fi network & password (`tiger-lily-0429`).
   - `warranty_card.jpg` — Purchase invoice with warranty expiration date (`2026-11-04`).
   - `electricity_bill.pdf` — Utility bill due on `2026-11-15` with account details.
   - `blood_test_report.png` — Diagnostic lab report with patient ID number and repeat test date (`2027-09-15`).
   - `boarding_pass.pdf` — Flight pass dated in the past (`2026-09-10`).
   - `otp_screenshot.png` — SMS screenshot of a 6-digit one-time code.
   - `photo_with_gps.jpg` — Photo containing EXIF GPS coordinates (`19.0760, 72.8777`).
   - `blood_test_report_copy.png` — Near-identical duplicate copy of the lab report.

---

## 2. Storyboard & Talking Points

| Timestamp | Visual on Screen | Voiceover / Script |
|---|---|---|
| **0:00 - 0:10** | Start with the Windows taskbar showing Airplane Mode ON. Show the messy demo folder. Drop the folder onto the Draku window. | *"This folder is on every laptop. Airplane mode is on. Everything you are about to see happens right here on this machine."* |
| **0:10 - 0:25** | Scanning progress completes quietly. The cryptic file names resolve to clear descriptive titles. In the Ask bar, type `"internet password"`. | *"Draku reads each file once. Now every file has a human name, and you can just ask in plain language. Even without matching keywords, local EmbeddingGemma surfaces the router login."* |
| **0:25 - 0:35** | The **3 Deadlines** card is in view. Hover over *Warranty ends in 6 days*. Click **Add to calendar**. | *"That date was trapped inside a picture. Now it's a real calendar reminder in your system with one tap."* |
| **0:35 - 0:48** | Switch to the **4 Exposed Secrets** card. The lab report's ID number and the photo's GPS coordinates are highlighted. Click **Make a safe copy**. | *"Before you share a document, you see what you'd give away. One tap pixelates the private numbers and strips the GPS tag, leaving the original file safe."* |
| **0:48 - 0:55** | Switch to the **3 files ready to let go** card. Click **Set aside**. The files disappear from the view, moving to `.draku/rest/`. | *"Three files nothing depends on — old OTPs, past boarding passes, duplicates. They rest safely in a folder you can restore from. Draku never deletes."* |
| **0:55 - 1:00** | Pan down to the Windows system tray showing Airplane Mode still active. | *"One more thing: that was offline the entire time. No uploads, no accounts, no subscriptions."* |

---

## 3. Verifying the Actions On Disk

1. **Remember**: Look in `.draku/calendar/` inside the scanned folder. You will find valid `.ics` iCalendar files generated with proper summary and date fields.
2. **Protect**: Look in `.draku/safe/`. The safe copies will have redacted pixel boxes and 0 EXIF GPS metadata.
3. **Let go**: Look in `.draku/rest/`. The set-aside files are preserved in their relative folder structures with an audit log in `draku.db`.
