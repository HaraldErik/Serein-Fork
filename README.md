# Serein-Fork

<p align="center">
  <img src="docs/preview.png" alt="Serein Native Discord Client" width="900" style="max-width: 100%; height: auto; border-radius: 8px;" />
</p>

<p align="center">
  <strong>A lightweight, native Discord desktop client written in Rust (egui + wgpu), with a sandboxed WebAssembly plugin system.</strong>
</p>

<p align="center">
  <a href="#install"><strong>📦 Install</strong></a> &nbsp;•&nbsp;
  <a href="#build-from-source"><strong>🛠️ Build</strong></a> &nbsp;•&nbsp;
  <a href="#plugins"><strong>🧩 Plugins</strong></a> &nbsp;•&nbsp;
  <a href="#highlights"><strong>⚡ Highlights</strong></a> &nbsp;•&nbsp;
  <a href="#feature-matrix"><strong>📋 Features</strong></a> &nbsp;•&nbsp;
  <a href="#troubleshooting"><strong>🩺 Troubleshooting</strong></a>
</p>

<p align="center">
  <a href="rust-toolchain.toml"><img src="https://img.shields.io/badge/rust-1.98.1_pinned-blue.svg?logo=rust" alt="Rust 1.98.1 pinned" /></a>
  <a href="docs/platform-support.md"><img src="https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-informational.svg" alt="Platforms" /></a>
  <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green.svg" alt="License" /></a>
</p>

> [!WARNING]
> **Unofficial and not endorsed by Discord.** Serein talks to Discord's public gateway and REST endpoints
> for your existing account. Automating normal accounts outside the official OAuth2/bot API violates
> Discord's Terms of Service and risks account termination. Read the
> [compatibility matrix](docs/discord-compatibility.md) and [authentication guide](docs/authentication.md)
> before use.

## About this fork

Serein-Fork is a fork of [ViceVerse-cz/Serein](https://github.com/ViceVerse-cz/Serein) that adds:

- **A much more capable plugin system** (new `decorations` API; see [Plugins](#plugins) and the
  [v2 roadmap](docs/plugin-api-v2.md)), and a higher installed-plugin limit (64, was 8).
- **Automatic Linux packages** built on every code push: `.deb`, `.rpm`, Arch `.pkg.tar.zst`,
  AppImage and Flatpak ([Install](#install)).
- **An Arch source installer** that compiles on your machine and installs through `pacman`.
- **Wayland image paste** for JPEG/WebP/GIF clipboard images (KDE Spectacle, browsers).
- **Update checks that read this fork**, not upstream.

Serein sends no telemetry. The upstream project's history and credits are preserved.

## Install

### Arch Linux, CachyOS, EndeavourOS, Manjaro (build on your machine)

```sh
git clone https://github.com/HaraldErik/Serein-Fork
cd Serein-Fork
bash packaging/arch/install.sh
```

The script installs build dependencies with `pacman`, builds Serein with Cargo, installs the
resulting package with `pacman -U` and refreshes desktop entries (including KDE's menu). Run it as your
normal user. If you already have the `rust` package it is used; `rustup` is only installed when you have
neither. Remove it later with `sudo pacman -R serein`. More in [`packaging/arch`](packaging/arch/README.md).

### Prebuilt Linux packages (GitHub Actions)

Every push that changes code runs **Build Linux packages on push**.

1. Open the repository's **Actions** tab, choose the latest successful run of that workflow.
2. Download an artifact from **Artifacts** at the bottom (kept 14 days): names start with `serein-linux-`.
3. Unzip it and install with your package manager:

```sh
sudo apt install ./serein-*.deb                     # Ubuntu 26.04 (x86_64 or ARM64)
sudo dnf install ./serein-*.fc43.*.rpm              # Fedora 43 (fc44 for Fedora 44)
sudo zypper install ./serein-*.suse.*.rpm           # openSUSE Tumbleweed
sudo pacman -U ./serein-*.pkg.tar.zst               # Arch
flatpak install --user ./serein-*.flatpak           # Flatpak bundle
chmod +x serein-*.AppImage && ./serein-*.AppImage   # AppImage (needs host GTK4/WebKitGTK 6)
```

Use a directory containing only the one package you want. These packages are unsigned CI builds.
Voice needs the GStreamer Good plugin set (`gst-plugins-good` on Arch); screen sharing needs a
PipeWire-capable portal. Details: [`packaging/linux`](packaging/linux/README.md).

### Windows and macOS

CI also produces unsigned `serein-<OS>-<arch>-unsigned` artifacts on pushes to `main`. Signed installers,
Homebrew and similar channels belong to the upstream project's
[releases](https://github.com/ViceVerse-cz/Serein/releases) and are not published by this fork.

## Build from source

Requirements: a C/C++ toolchain, CMake, `pkg-config` and, on Linux, the GTK 4 / WebKitGTK 6 development
packages below. Rust **1.98.1** is pinned in [`rust-toolchain.toml`](rust-toolchain.toml); with `rustup`
it is selected automatically (a newer system Rust may also work but is untested).

| Distro | Install build dependencies |
| --- | --- |
| Ubuntu / Debian | `sudo apt install build-essential cmake pkg-config git curl libgtk-4-dev libwebkitgtk-6.0-dev libfontconfig1-dev libxkbcommon-dev libwayland-dev libx11-dev libxi-dev libxrandr-dev libxcursor-dev libvulkan-dev libasound2-dev libpulse-dev libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev` |
| Fedora | `sudo dnf install gcc gcc-c++ make cmake pkgconf-pkg-config git curl gtk4-devel webkitgtk6.0-devel fontconfig-devel libxkbcommon-devel libxkbcommon-x11-devel wayland-devel libX11-devel libXi-devel libXrandr-devel libXcursor-devel vulkan-loader-devel alsa-lib-devel pulseaudio-libs-devel gstreamer1-devel gstreamer1-plugins-base-devel` |
| Arch | `sudo pacman -S --needed base-devel cmake pkgconf git curl gtk4 webkitgtk-6.0 fontconfig libxkbcommon libxkbcommon-x11 wayland libx11 libxi libxrandr libxcursor vulkan-headers vulkan-icd-loader alsa-lib libpulse gstreamer gst-plugins-base-libs gst-plugins-good` |
| macOS | `xcode-select --install` |
| Windows | Visual Studio C++ build tools and the WebView2 Runtime |

```sh
git clone https://github.com/HaraldErik/Serein-Fork
cd Serein-Fork

cargo run --locked --release                        # run the client (saved login or official login webview)
cargo run --locked --features demo -- --demo        # synthetic offline conversations, no account needed
```

Build packages yourself (Linux, run on the target distribution as a normal user):

```sh
cargo xtask package --format deb        # or: rpm | arch | appimage
python3 packaging/flatpak/build.py target/flatpak-build   # Flatpak, needs flatpak-builder
```

Check your changes the way CI does:

```sh
cargo xtask check          # fmt, Clippy, tests, policy checks
cargo replay               # reducer benchmark
node tests/login-handoff.cjs   # only for login changes
```

Source builds store data in a separate `serein-development` directory so they never touch an installed
Serein. Set `SEREIN_DATA_DIR` to an absolute path to use another data root. Login tokens stay in the OS
credential store.

## Plugins

Plugins are small, sandboxed WebAssembly modules. Write them in Rust (SDK included), C, C++ or anything
that compiles to Wasm. They are native Serein plugins, **not** Vencord/Equicord plugins.

**Use a plugin:** *Settings → Extensions → Import*, choose a `.serein-extension` file, review the
permissions it asks for, enable it. The catalog tab lists reviewed packages. Disabling removes the
package and its data.

**What they can do (today):** show native panels, read data Serein already loaded (selected channel,
messages, members, voice state…), edit your draft, **rewrite messages when you press Send**, add **`/slash` commands**, save one small value, react to message and app
events, change appearance, add a custom presence, **put badges and text beside message authors**, and
*propose* actions (send, edit, react, navigate, moderate…) that run only after you click **Apply**.

**What they cannot do:** read your token, touch files, use the network, or run while the UI is drawing.
Each call gets a fresh sandbox: 16 MiB memory, a fixed CPU budget, 256 KiB of input/output. This is why
plugins cost almost no idle RAM. Up to 64 plugins can be installed.

**Write one in five minutes:**

```sh
rustup target add wasm32-unknown-unknown
cd examples/extensions
cargo build --release --target wasm32-unknown-unknown -p link-badges
python ../../extensions/pack.py link-badges/manifest.json \
    target/wasm32-unknown-unknown/release/link_badges.wasm link-badges.serein-extension
cd ../.. && cargo run --locked --features demo -- --demo      # then import the file in Settings → Extensions
```

Read the **[Plugin guide](docs/plugin-guide.md)** for the manifest, actions, permissions, limits, testing
and the decorations API, and browse the [example plugins](examples/extensions). The
[roadmap](docs/plugin-api-v2.md) lists what is planned next (UI buttons and menus, send hooks and
commands, timers, pull-only network access, limited auto-apply) and what will never be supported.

## Troubleshooting

| Problem | Try |
| --- | --- |
| Pasting an image does nothing on Wayland | Copy the image again; Serein reads PNG, JPEG, WebP and GIF clipboard offers. If it still fails, report which app you copied from. |
| Arch script: `rustup` and `rust` conflict | Use the current script (`git pull`); it keeps your installed Rust. |
| App is missing from the KDE menu after install | Re-run the script's last step: `kbuildsycoca6 --noincremental`, or log out and back in. |
| A false "update available" notice | Older builds compared against upstream releases. Pull the latest commit and rebuild; updates now come from this fork's releases. |
| No sound / voice fails | Install `gst-plugins-good`; screen share needs `gst-plugin-pipewire` and a ScreenCast portal. |
| Plugin import is rejected | The manifest asks for an unknown permission or an action whose surface needs a permission you did not list. See the [guide](docs/plugin-guide.md#permissions-capabilities). |
| Build fails with a missing `pango`/`gtk4` library | Install the development packages from the table above. |

---

## Highlights

- ⚡ **Pure Native Performance:** Built with pure Rust, `egui`, and `wgpu`. Immediate-mode rendering with minimal idle CPU, low memory footprint, and instantaneous launch times—zero Electron, Node.js, or web runtime overhead.
- 🌐 **Direct Gateway & REST Transports:** Direct connection to Discord's official endpoints with active rate-limiting cooldowns, heartbeat handling, reconnect/resume loops, and partial payload patching.
- 🔒 **Secure OS Credential Storage:** Session tokens use your operating system's secure vault by default (macOS Keychain, Windows Credential Manager, or Linux Secret Service). Settings › Storage offers an explicit, unencrypted plaintext fallback when the keyring does not work.
- 🛡️ **Ephemeral Authentication Webview:** Sign-in uses Discord's official hosted login page inside a temporary native webview (WKWebView, WebView2, or WebKitGTK) supporting email/password, QR login, and MFA. An origin-checked handoff secures the session credential and immediately terminates the webview.
- 💾 **Bounded Local Persistence:** Recent chat history, drafts, image previews, settings, and diagnostics are stored in an account-isolated, bounded local SQLite database. All local data is strictly cleared upon explicit logout.
- 🎙️ **Voice Calls, Video & Screen Sharing:** Complete native voice engine with 1-to-1 and group DM calls, server voice channels, push-to-talk, Sonora AEC3 acoustic echo cancellation, RNNoise noise suppression, Opus codec, and DAVE v1 end-to-end encryption. Includes native screen capture (macOS ScreenCaptureKit, Windows Graphics Capture, Linux portal/PipeWire with VA-API/NVENC hardware encoding and software fallback; Linux native capture remains unverified) and incoming stream & camera video playback with hardware-accelerated decoding (VideoToolbox, VA-API, DirectX).
- 🧵 **Forum Channels & Active Threads:** Browse forum channels, view posts sorted by recent activity, read message threads with unread indicators, and create new forum posts directly in-app.
- ⚙️ **Server Administration Suite:** Full server management interface including Server Profiles (banners, icons, traits, descriptions), custom sticker management, role editor with fine-grained permission matrix, paginated audit logs with action filters, invite manager with revocation, integrations and webhooks, and member moderation.
- ✨ **GIF & Twemoji Picker:** Instant KLIPY GIF search with favorites and one-click sending, full Twemoji picker with search and quick-reactions, plus custom guild emojis.
- 📎 **Multi-Attachment Batch Uploads:** Composer staging tray supporting multiple files of any type (PDF, ZIP, 3D STL, videos, audio, images) with file-type badges, thumbnails, size indicators, individual removal, and progress tracking.
- 👤 **Native Profile Customization:** In-app profile editor for global display names, bios / about me, pronouns, and custom accent colors with real-time live preview cards.
- 🎨 **Extensions & Theme Shop:** Git-backed plugin engine and community theme shop with preview cards, color preset toggles, permission verification, and a built-in deleted-message retention protector.
- 🎬 **Rich Media & Video Player:** Inline video playback for MOV, MP4, and WebM attachments where platform codecs are available, interactive seekable voice message waveforms, right-click media save/copy context menus, and full-resolution image viewer modals.
- ⌨️ **Keybinds & Shortcuts:** Built-in keybind reference sheet styled with raised keycaps, quick edit (`Up`), quick delete (`Backspace`), and intuitive keyboard navigation.
- 🎮 **Rich Presence & Game Detection:** Built-in Discord IPC and WebSocket RPC servers, plus executable-based detection of running games, showing live game activities in member rosters, DM lists, and user profiles, with opt-in system tray integration.

---

## Feature Showcase

| Voice Calls & Live Screen Sharing | User Settings & Profile Customizer |
| :---: | :---: |
| <img src="docs/screenshots/voice-calls-screenshare.png" alt="Voice Calls & Screen Sharing" width="450" /> | <img src="docs/screenshots/user-settings.png" alt="User Settings & Profile Customizer" width="450" /> |
| **Server Administration & Profiles** | **Threads & Forum Channels** |
| <img src="docs/screenshots/server-settings.png" alt="Server Administration & Profiles" width="450" /> | <img src="docs/screenshots/threads-forums.png" alt="Threads & Forum Channels" width="450" /> |
| **Multi-File Attachment Uploads** | **GIFs & Twemoji Picker** |
| <img src="docs/screenshots/file-uploads.png" alt="Multi-File Attachment Uploads" width="450" /> | <img src="docs/screenshots/gifs-and-emojis.png" alt="GIFs & Twemoji Picker" width="450" /> |

---

## Measured Performance vs. Official Discord

*Figures below were measured by the upstream project on macOS; they are not re-measured for this fork or its plugin additions.*

> **Testing Scenario:** Browsing channels while joined in a Voice Channel (VC) and streaming screen at 60 FPS on macOS.

| Metric | Official Discord Client (Electron) | Serein (Native Rust + egui/wgpu) | Advantage |
|---|:---:|:---:|:---:|
| **Memory (RAM)** | **1,178.4 MB** *(across 7 helper processes)* | **129.7 MB** *(single unified process)* | **~9× less memory (-89%)** |
| **CPU Usage** | **22.8%** *(Renderer + Helper processes)* | **8.1%** | **~2.8× lower CPU (-64%)** |

| Official Discord (Electron) | Serein (Native Rust) |
| :---: | :---: |
| **RAM: ~1,178.4 MB across 7 processes** | **RAM: 129.7 MB single process** |
| <img src="docs/screenshots/perf-discord-ram.png" alt="Discord RAM Usage" width="450" /> | <img src="docs/screenshots/perf-serein-ram.png" alt="Serein RAM Usage" width="450" /> |
| **CPU: 22.8% total** | **CPU: 8.1% total** |
| <img src="docs/screenshots/perf-discord-cpu.png" alt="Discord CPU Usage" width="450" /> | <img src="docs/screenshots/perf-serein-cpu.png" alt="Serein CPU Usage" width="450" /> |

---

---

## Feature Matrix

| Capability | Status | Notes |
|---|---|---|
| **Navigation & Guilds** | Implemented | Collapsible categories, cached icons, guild channels, forum channels, active threads, DM lists, People pane, and server channel context menus |
| **Message Timeline** | Implemented | Virtualized variable-height rows, inline link confirmations, spoiler text/media reveal, unread message banners, deleted message protector, and local timezone timestamps |
| **Markdown & System Messages** | Implemented | Bold, italics, code blocks, blockquotes, clickable links, and styled system events with tinted Phosphor icons and clickable member names |
| **Reactions & Emojis** | Implemented | Twemoji rendering, native reaction counts, eight-emoji quick picker, full emoji picker integration, custom guild emojis, and add/remove reaction controls |
| **GIFs & Media Search** | Implemented | KLIPY GIF picker with search, favorites category, and one-click direct sending |
| **User Mentions & Autocomplete** | Implemented | Clickable user mentions with interactive composer autocompletion and visual highlight styling |
| **Media Previews & Video Player** | Implemented | Inline MOV, MP4, and WebM playback where platform codecs are available; media copy/save context menus, inline image cards, embed cards, related embed image galleries, and full-resolution image viewer modals |
| **File & Attachment Uploads** | Implemented | Multi-attachment batch staging with file-type badges (PDF, ZIP, STL, images), thumbnail previews, individual file removal, upload progress bar, and drag-and-drop |
| **Voice Engine & Calls** | Implemented | 1-to-1/group DM calls & server channels, Opus codec, DAVE v1 E2EE, Sonora AEC3 acoustic echo cancellation, RNNoise suppression, push-to-talk (`V`), audio device selector |
| **Voice Messages** | Implemented | Inline voice message playback with interactive waveforms and bounded streaming audio buffering |
| **Screen Sharing & Video** | Implemented | Native screen capture (macOS ScreenCaptureKit, Windows Graphics Capture, Linux portal/PipeWire with VA-API/NVENC hardware encoding and software fallback; Linux native capture remains unverified), quality presets (720p/1080p, up to 60fps), and local camera/screen previews |
| **Camera Video & Stream Viewing** | Implemented | Hardware-accelerated decoding (macOS VideoToolbox, Linux VA-API, Windows DXVA/D3D11) for incoming screen streams and camera video feeds |
| **Threads & Forum Channels** | Implemented | Forum post listing, recent activity sorting, active thread browsing, and new forum post / thread creation |
| **Server Administration** | Implemented | Server profile editor (banners, icons, traits), custom sticker upload/edit/delete, role management with permissions matrix, audit log viewer, invite tracking and revocation, integrations/webhooks, and member moderation |
| **Extensions & Theme Shop** | Implemented | Git-backed plugins, community theme catalog with preview cards and color presets, permission prompt modals, and deleted-message protector |
| **Keybinds & Shortcuts** | Implemented | In-app keybind cheat sheet with raised keycaps, quick edit (`Up`), quick delete (`Backspace`), and keyboard navigation hotkeys |
| **Rich Presence & Game IPC** | Implemented | Discord IPC and WebSocket RPC servers plus running-game detection; displays activities in member rosters, DMs, and user profiles; opt-in system tray |
| **Profile Cards & Editing** | Implemented | On-demand profile popouts with banners, bios, badges, connections; native in-app editor for display name, bio, pronouns, and custom accent color with live preview |
| **Server & Group Actions** | Implemented | Server dropdown with friend invites and leave server; group DM actions (edit name/icon preview, mute, leave) |
| **Context Menus & Shortcuts** | Implemented | Right-click context menus for messages, media (save/copy), server channels, and members |
| **Typing Indicators** | Implemented | Displays incoming typing with short expiry; Serein strictly avoids emitting outgoing typing signals |
| **Persistence & Drafts** | Implemented | Bounded SQLite cache for history, drafts, settings, and diagnostics; OS credential store for auth tokens; sanitary logout |
| **Internationalization** | Partial | Bundled Inter font, CJK and Arabic font fallbacks included; full IME and bidirectional editing unverified |

---

## Architecture Overview

Serein is engineered as a clean multi-crate Cargo workspace, isolating UI rendering from networking, persistence, and service protocols:

```
serein/
├── apps/
│   └── desktop/          # Application entrypoint, CLI flags, window lifecycle
├── crates/
│   ├── client-core/      # Client state coordinator, generation tracking, events
│   ├── session-cache/    # In-memory bounded cache and state reconciliation
│   ├── ui/               # egui widgets, message virtualizer, themes, design tokens
│   ├── model/            # Strongly-typed Discord domain entities
│   ├── discord-protocol/ # Wire protocol serialization and partial payload patches
│   ├── discord-api/      # HTTP/2 REST client with rate limiting and backoff
│   ├── discord-gateway/  # WebSocket gateway client with heartbeat and resume
│   ├── discord-voice/    # Opus codecs, RTP/UDP transport, DAVE v1, Sonora AEC, RNNoise, video decoding
│   ├── local-store/      # Bounded SQLite database for history, drafts, settings
│   ├── platform/         # OS credential store (Keychain/CredManager/SecretService)
│   └── test-support/     # Deterministic synthetic fixtures and mocks
└── tools/
    ├── replay-bench/     # Benchmarking harness for state reducers
    └── xtask/            # Workspace automation tasks (packaging, checks, linting)
```

---

## Security & Storage Policy

- **Token Protection:** Tokens use the native OS credential store by default (macOS Keychain, Windows Credential Manager, Linux Secret Service). An explicit Storage setting allows private, bounded plaintext files when OS saving fails; disabling it deletes those secret files. Active tokens remain redacted in memory.
- **Local Cache Bounds:** SQLite databases store recent channel history, drafts, settings, diagnostics, and image preview metadata within bounded byte and count limits. The local SQLite store is **not** encrypted by the application.
- **Sanitary Logout:** Executing an explicit logout destroys active network sessions, purges active secrets from memory, deletes the token from the OS credential store, and erases that account's local cache and drafts.
- **Zero Telemetry:** Serein contains no analytics, telemetry, background crash collectors, or tracking beacons. Planned plugin network access (see [Plugin API v2](docs/plugin-api-v2.md)) is pull-only HTTPS GET to user-approved hosts; Serein never uploads data to a service of its own.
- **Platform Integrity:** No fingerprint spoofing, CAPTCHA/MFA bypasses, bot substitutions, token scrapers, or third-party relays.

For full details, review the [Storage Policy](docs/storage-policy.md) and [Threat Model](docs/threat-model.md).

---

## Documentation

| Topic | Where |
| --- | --- |
| **Plugin guide (start here)** | [docs/plugin-guide.md](docs/plugin-guide.md) |
| Plugin API roadmap and design | [docs/plugin-api-v2.md](docs/plugin-api-v2.md) |
| SDK overview, data and action reference | [overview](docs/extension-sdk-overview.md), [data](docs/extension-sdk-reference.md), [actions](docs/extension-sdk-actions.md), [troubleshooting](docs/extension-sdk-troubleshooting.md) |
| Capability (permission) table | [docs/extensions.md](docs/extensions.md#capability-reference) |
| Example plugins and offline authoring | [examples/extensions](examples/extensions/README.md) |
| Themes | [docs/theme-api.md](docs/theme-api.md) |
| Architecture | [docs/architecture.md](docs/architecture.md) |
| Discord compatibility and protocol | [docs/discord-compatibility.md](docs/discord-compatibility.md) |
| Authentication | [docs/authentication.md](docs/authentication.md) |
| Storage policy | [docs/storage-policy.md](docs/storage-policy.md) |
| Platform support | [docs/platform-support.md](docs/platform-support.md) |
| Voice | [docs/voice.md](docs/voice.md) |
| Linux packaging, Arch installer | [packaging/linux](packaging/linux/README.md), [packaging/arch](packaging/arch/README.md) |
| Nix | [nix/README.md](nix/README.md) |
| Threat model | [docs/threat-model.md](docs/threat-model.md) |
| Third-party licenses | [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) |

## License

MIT OR Apache-2.0. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).
