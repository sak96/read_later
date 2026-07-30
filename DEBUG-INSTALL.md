# Debug Install

Instructions for installing debug and release APKs without losing your data.

## Exporting and Importing Data

**Exporting:**

- Open the app
- Tap Settings
- Scroll to the **Restore** section for saving articles
- Tap the **Export** button (download icon)
- Choose where to save the file on your device
- For pronunciation rules: scroll to **Speech** section
- Tap the **Export** button in the **Pronunciation Rules** row
- Choose where to save the file on your device

**Importing:**

- Open the app
- Tap Settings
- Scroll to the **Restore** section for importing articles
- Tap the **Import** button (upload icon)
- Select the previously exported file
- For pronunciation rules: scroll to **Speech** section
- Tap the **Import** button in the **Pronunciation Rules** row
- Select the previously exported file

## Configuring WebDAV

- Open the app
- Tap Settings
- Go to the **WebDAV** section
- Enable the toggle
- Fill in:
  - **URL**: Your WebDAV server address
  - **Auth Type**: Basic, Digest, or Anonymous
  - **Username**: Your WebDAV username
  - **Password**: Your WebDAV password
  - **Path**: Directory on server
- Tap the sync button or restart the app to trigger a sync

## Switching from Release to Debug APK

- Export your data (see [Exporting and Importing Data](#exporting-and-importing-data))
- Uninstall the release app via Android UI (long-press app icon -> Uninstall, or Settings -> Apps -> "Read Later" -> Uninstall)
- Download and install the debug APK
- Import your data (see [Exporting and Importing Data](#exporting-and-importing-data))
- Reconfigure WebDAV (see [Configuring WebDAV](#configuring-webdav))

## Switching from Debug to Release APK

- Export your data (see [Exporting and Importing Data](#exporting-and-importing-data))
- Uninstall the debug app via Android UI
- Download and install the release APK from Google Play Store, IzzyOnDroid, or GitHub Releases
- Import your data (see [Exporting and Importing Data](#exporting-and-importing-data))
- Reconfigure WebDAV (see [Configuring WebDAV](#configuring-webdav))
