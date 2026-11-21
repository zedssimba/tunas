TUNAS for macOS - READ ME FIRST
S.A.L.T CI · Owned by S.A.L.T. CI TECHNOLOGY · Distributed by Pemangkin Teknologi

Which package?
  TUNAS_<version>_mac_apple-silicon  -  Macs with an M1 chip or later
  TUNAS_<version>_mac_intel          -  Macs with an Intel processor
  (Apple menu > About This Mac shows your chip.)
  Needs macOS 11 (Big Sur) or later.

Install
  1. Open the .dmg and drag TUNAS into Applications.
  2. This TUNAS is not yet signed by Apple, so macOS will block it the
     first time ("TUNAS is damaged" or "cannot be opened").
     Open Terminal (Applications > Utilities > Terminal) and run this once:

       xattr -dr com.apple.quarantine /Applications/TUNAS.app

  3. Open TUNAS from Applications. It opens into setup.
  4. The first time TUNAS saves a TERAS or cloud key, macOS asks to let
     TUNAS use the Keychain. Choose "Always Allow".

Update
  Drag the new TUNAS into Applications and choose Replace, then run the
  Terminal command in step 2 again. Your chats, models and settings are kept.

Uninstall
  Drag TUNAS from Applications to the Bin. To also delete your chats and
  models, delete the folder:  ~/Library/Application Support/TUNAS

Your data
  Chats, models, knowledge and settings:  ~/Library/Application Support/TUNAS
  TERAS and cloud keys:                   your macOS Keychain (service "TUNAS")

What changed in this version: see TUNAS_Release_Notes.md.

www.salt.ci · info@salt.ci · www.pemangkin.tech · info@pemangkin.tech
S.A.L.T CI and its components are proprietary technology of S.A.L.T. CI TECHNOLOGY,
distributed and commercialised by Pemangkin Teknologi.
