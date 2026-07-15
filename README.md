# Prism

Prism is a Tauri and Angular desktop utility for monitoring and managing local workstation resources, processes, startup apps, storage, terminal sessions, and system tools.

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) + [Angular Language Service](https://marketplace.visualstudio.com/items?itemName=Angular.ng-template).

## macOS Packaging

The default macOS bundle uses ad-hoc signing so local builds work without an Apple Developer certificate:

```sh
pnpm tauri build --bundles app
pnpm tauri build --bundles dmg
```

For distribution outside the App Store, install a `Developer ID Application` certificate and override the default identity for the build:

```sh
APPLE_SIGNING_IDENTITY="Developer ID Application: Name (TEAM_ID)" pnpm tauri build --bundles dmg
```

Notarization also requires either App Store Connect API credentials (`APPLE_API_ISSUER`, `APPLE_API_KEY`, and `APPLE_API_KEY_PATH`) or Apple ID credentials (`APPLE_ID`, `APPLE_PASSWORD`, and `APPLE_TEAM_ID`). Keep signing and notarization credentials in the local environment or CI secret store rather than project files.
