# Serein

<p align="center">
  <img src="docs/preview.png" alt="Serein showing a server channel" width="900" />
</p>

Serein is an unofficial native Discord client for existing accounts, written in Rust with
egui and wgpu. It runs as one process without a web runtime.

<p align="center">
  <a href="https://github.com/ViceVerse-cz/Serein/releases"><img src="https://img.shields.io/github/v/release/ViceVerse-cz/Serein?label=release&color=blue" alt="Latest release" /></a>
  <a href="Cargo.toml"><img src="https://img.shields.io/badge/rust-1.99.1-blue.svg?logo=rust" alt="Rust 1.99.1" /></a>
  <a href="docs/platform-support.md"><img src="https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-informational.svg" alt="macOS, Linux and Windows" /></a>
  <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green.svg" alt="MIT or Apache-2.0" /></a>
  <a href="https://discord.gg/UqTDGCENaN"><img src="https://img.shields.io/badge/Discord-project%20server-5865F2?logo=discord&logoColor=white" alt="Project Discord server" /></a>
</p>

> [!WARNING]
> Serein is not made or endorsed by Discord. It talks to Discord's gateway and REST endpoints
> as your account. Using a normal account outside the official OAuth2 and bot APIs breaks
> Discord's Terms of Service and can get the account terminated. Read the
> [compatibility notes](docs/discord-compatibility.md) and the
> [authentication guide](docs/authentication.md) first.

## Install

Builds for Windows, macOS and Linux are on [Releases](https://github.com/ViceVerse-cz/Serein/releases).

| Platform | Formats | Architectures |
|---|---|---|
| Windows | Installer (`-Setup.exe`), zip | x86_64, aarch64 |
| macOS | Homebrew cask, zip, Nix flake | Apple silicon |
| Linux | Flatpak, apt/dnf/zypper/pacman repositories, Gentoo ebuild, Nix flake, AppImage | x86_64 |

<details open>
<summary><h3>Linux</h3></summary>

#### Flatpak (recommended)

Sandboxed, with bundled GNOME/WebKit runtimes and automatic updates.

- From the repository:
  ```sh
  flatpak install --user https://viceverse-cz.github.io/Serein/flatpak/serein.flatpakref
  ```
  GNOME Software, KDE Discover or `flatpak update` then install updates.

- From a downloaded bundle: get `Serein-linux.flatpak` from [Releases](https://github.com/ViceVerse-cz/Serein/releases), then:
  ```sh
  flatpak install --user ./Serein-linux.flatpak
  flatpak run cz.viceverse.serein
  ```

Sandbox permissions and source builds: [Flatpak guide](packaging/flatpak/README.md).

#### Package repositories (apt, dnf, zypper, pacman)

Set up the signed repository for your distribution:
```sh
curl -fsSL https://viceverse-cz.github.io/Serein/setup.sh | sh
```
The script detects Ubuntu/Debian, Fedora, openSUSE or Arch Linux, verifies the GPG signing key, adds the repository and offers to install Serein. Then install with your package manager:
```sh
# Ubuntu / Debian: sudo apt install serein
# Fedora:          sudo dnf install serein
# openSUSE:        sudo zypper install serein
# Arch Linux:      sudo pacman -S serein
```
System updates keep Serein current. Manual key verification: [signed package repositories](packaging/repositories/README.md).

#### Gentoo

Install from the [vitaly-zdanevich-overlay](https://github.com/vitaly-zdanevich/gentoo-overlay) overlay. It provides a source ebuild ([net-im/serein](https://github.com/vitaly-zdanevich/gentoo-overlay/tree/main/net-im/serein)) and a prebuilt amd64 ebuild ([net-im/serein-bin](https://github.com/vitaly-zdanevich/gentoo-overlay/tree/main/net-im/serein-bin)).

```sh
sudo eselect repository add vitaly-zdanevich-overlay git https://github.com/vitaly-zdanevich/gentoo-overlay.git
sudo emaint sync -r vitaly-zdanevich-overlay
echo 'net-im/serein ~amd64' | sudo tee /etc/portage/package.accept_keywords/serein
sudo emerge --ask net-im/serein
```

Use `net-im/serein-bin` in the keyword file and emerge command to install the prebuilt binary instead. The source ebuild needs Rust 1.99 or newer. The binary ebuild targets amd64 with glibc 2.43 or newer. Both install the same files; choose one.

#### Nix

The flake builds from source for `x86_64-linux` and `aarch64-darwin`. Nix, not the in-app updater, handles updates. On macOS the result is an unsigned, non-notarized `Serein.app`. Development shell and details: [Nix package guide](packaging/nix/README.md).

```sh
nix run github:ViceVerse-cz/Serein#serein
nix profile install github:ViceVerse-cz/Serein#serein
```

To declare it in a flake, pass `inputs` through `specialArgs` (NixOS) and `extraSpecialArgs` (Home Manager); the modules below fail to import without it.

`flake.nix`
```nix
{
    inputs = {
        nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
        home-manager = {
            url = "github:nix-community/home-manager";
            inputs.nixpkgs.follows = "nixpkgs";
        };
        serein = {
            url = "github:ViceVerse-cz/Serein";
            inputs.nixpkgs.follows = "nixpkgs";
        };
    };

    outputs = { nixpkgs, home-manager, ... }@inputs: {
        # NixOS
        nixosConfigurations.hostname = nixpkgs.lib.nixosSystem {
            system = "x86_64-linux";
            specialArgs = { inherit inputs; };
            modules = [ ./configuration.nix ];
        };

        # Standalone Home Manager
        homeConfigurations.username = home-manager.lib.homeManagerConfiguration {
            pkgs = nixpkgs.legacyPackages.x86_64-linux;
            extraSpecialArgs = { inherit inputs; };
            modules = [ ./home-manager.nix ];
        };
    };
}
```

With Home Manager as a NixOS module, pass `inputs` through its own option:

```nix
{
    nixosConfigurations.hostname = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        specialArgs = { inherit inputs; };
        modules = [
            ./configuration.nix
            home-manager.nixosModules.home-manager
            {
                home-manager.extraSpecialArgs = { inherit inputs; };
                home-manager.users.username = import ./home-manager.nix;
            }
        ];
    };
}
```

`configuration.nix`
```nix
{
    inputs,
    pkgs,
    ...
}: {
    environment.systemPackages = [
        inputs.serein.packages.${pkgs.stdenv.hostPlatform.system}.default
    ];
}
```

`home-manager.nix`
```nix
{
    inputs,
    pkgs,
    ...
}: {
    home.packages = [
        inputs.serein.packages.${pkgs.stdenv.hostPlatform.system}.default
    ];
}
```

#### AppImage

Download `serein-<version>-Linux-X64.AppImage` from [Releases](https://github.com/ViceVerse-cz/Serein/releases), then:
```sh
chmod +x ./serein-*-Linux-X64.AppImage
./serein-*-Linux-X64.AppImage
```
Keep it in a writable directory to receive updates from **Settings > Updates**. It uses the host's GTK 4 and WebKitGTK 6.0; see [AppImage requirements](packaging/appimage/README.md).

</details>

<details>
<summary><h3>Windows</h3></summary>

#### Installer (recommended)
Run `Windows-X64-Setup.exe` or `Windows-ARM64-Setup.exe` from [Releases](https://github.com/ViceVerse-cz/Serein/releases). It installs per user to `%LOCALAPPDATA%\Programs\Serein` without elevation, adds a Start Menu shortcut with the `cz.viceverse.serein` AppUserModelID for notifications, appears in Installed Apps for uninstalling, and keeps its registered version in step with in-app updates.

#### PowerShell setup
Extract `Windows-X64.zip` or `Windows-ARM64.zip` and run:
```powershell
powershell -ExecutionPolicy Bypass -File .\setup.ps1
```
To uninstall:
```powershell
powershell -ExecutionPolicy Bypass -File .\setup.ps1 -Uninstall
```

#### Portable
Extract the zip anywhere and run `serein.exe`. For desktop notifications, run this beside `serein.exe`:
```powershell
powershell -File .\install-notifications.ps1
```
If the `RemoteSigned` policy blocks it, review the script and run `Unblock-File -LiteralPath .\install-notifications.ps1` first. Installed builds do not need it.

</details>

<details>
<summary><h3>macOS</h3></summary>

#### Homebrew
```sh
brew tap ViceVerse-cz/serein
brew install --cask serein
```
The cask is in the [`homebrew-serein`](https://github.com/ViceVerse-cz/homebrew-serein) tap. If you tapped the old URL, run `brew untap ViceVerse-cz/serein` and tap again.

#### App bundle
Download `serein-<version>-macOS-ARM64.zip` from [Releases](https://github.com/ViceVerse-cz/Serein/releases), unzip it and move `Serein.app` to `/Applications`.

</details>

## Features

- Servers, folders, categories, forum channels, threads, DMs and group DMs, with Events and
  Discovery.
- Message timeline with Markdown, embeds, spoilers, replies, reactions, stickers, polls,
  unread markers and local-time timestamps.
- Composer with mentions, emoji, KLIPY GIFs, slash commands with autocomplete, multi-file
  uploads and drafts.
- Voice and video: DM, group and server calls, push-to-talk, echo cancellation (Sonora AEC3),
  noise suppression (RNNoise), DAVE end-to-end encryption, screen sharing and hardware video
  decoding. Native capture on Linux is unverified.
- Server settings: profile, roles and permissions, emoji and stickers, invites, audit log,
  integrations and webhooks, members and moderation.
- Profiles, profile editing, Rich Presence (IPC and WebSocket RPC) and game detection.
- Extensions and themes from Git repositories, with permission prompts.
- Keybinds, context menus, system tray and desktop notifications.

What works, what is restricted and what is unverified is tracked per feature in
[docs/discord-compatibility.md](docs/discord-compatibility.md).

## Screenshots

| Voice and screen sharing | User settings |
| :---: | :---: |
| <img src="docs/screenshots/voice-calls-screenshare.png" alt="Voice call with screen sharing" width="450" /> | <img src="docs/screenshots/user-settings.png" alt="User settings" width="450" /> |
| **Server settings** | **Threads and forums** |
| <img src="docs/screenshots/server-settings.png" alt="Server settings" width="450" /> | <img src="docs/screenshots/threads-forums.png" alt="Threads and forum channels" width="450" /> |
| **Uploads** | **GIFs and emoji** |
| <img src="docs/screenshots/file-uploads.png" alt="Multi-file upload" width="450" /> | <img src="docs/screenshots/gifs-and-emojis.png" alt="GIF and emoji picker" width="450" /> |

## Resource use

One measurement on macOS, browsing channels while in a voice channel and streaming the screen
at 60 FPS:

| | Discord (Electron) | Serein |
|---|---:|---:|
| Memory | 1,178.4 MB across 7 processes | 129.7 MB in 1 process |
| CPU | 22.8% | 8.1% |

| Discord | Serein |
| :---: | :---: |
| <img src="docs/screenshots/perf-discord-ram.png" alt="Discord memory use" width="450" /> | <img src="docs/screenshots/perf-serein-ram.png" alt="Serein memory use" width="450" /> |
| <img src="docs/screenshots/perf-discord-cpu.png" alt="Discord CPU use" width="450" /> | <img src="docs/screenshots/perf-serein-cpu.png" alt="Serein CPU use" width="450" /> |

## Security and storage

- Tokens are kept in the OS credential store (Keychain, Credential Manager, Secret Service).
  If saving there fails, Settings > Storage can enable private plaintext files instead;
  turning that off deletes them. Active tokens are redacted in memory.
- Sign-in uses Discord's own login page in a temporary webview (WKWebView, WebView2 or
  WebKitGTK); the webview closes after an origin-checked handoff.
- Recent history, drafts, settings, diagnostics and image metadata are stored in a bounded
  SQLite database per account. It is not encrypted. Logging out deletes the token and that
  account's local data.
- No telemetry or analytics. No CAPTCHA or MFA bypass. Every connection presents one
  consistent browser identity matching the platform webview; it is not rotated.

Details: [storage policy](docs/storage-policy.md) and [threat model](docs/threat-model.md).

## Building from source

### Requirements
Rust 1.99.1 (pinned), a C/C++ toolchain, CMake, Git and libclang; BoringSSL is built from source.
- **macOS:** Xcode command-line tools (`xcode-select --install`)
- **Linux:** GCC/Clang, ALSA development headers, `pkg-config`, GTK 4, WebKitGTK 6.0, fontconfig, and Vulkan drivers (see [Platform Support](docs/platform-support.md))
- **Windows:** Visual Studio C++ build tools and WebView2 Runtime

### Run

```sh
# Synthetic conversations only; extension browsing still downloads from public GitHub
cargo run --locked --features demo -- --demo

# Normal client: saved login or Discord's sign-in page
cargo run --locked
```

Source builds keep their data in an OS-local `serein-development` directory, separate from
installed Serein and untouched by `cargo clean`. Set `SEREIN_DATA_DIR` to an absolute path for
another data root; packaged builds use `serein`. Development and packaged builds use separate
credential-store entries.

### Checks and packaging

```sh
# Formatting, Clippy, tests and policy checks
cargo xtask check

# Reducer benchmark
cargo replay

# Login handoff tests
node tests/login-handoff.cjs

# Release package (macOS .app, Linux .deb by default)
cargo xtask package
```

Demo data exists only in builds with `--features demo` and only appears with `--demo`;
`--demo-*` flags also need `--demo`. `cargo xtask package` never includes it.

## Workspace layout

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

## Documentation

- [Architecture](docs/architecture.md)
- [Discord compatibility](docs/discord-compatibility.md)
- [Authentication](docs/authentication.md)
- [Storage policy](docs/storage-policy.md)
- [Platform support](docs/platform-support.md)
- [Nix package](packaging/nix/README.md)
- [Voice](docs/voice.md)
- [Design](docs/design.md)
- [Extensions](docs/extensions.md)
- [Extension SDK wiki](https://github.com/ViceVerse-cz/Serein/wiki)
- [SDK examples](examples/extensions/README.md)
- [Theme API](docs/theme-api.md)
- [Threat model](docs/threat-model.md)
- [Third-party notices](THIRD_PARTY_NOTICES.md)

## License

Serein's own code is available under either the [MIT License](LICENSE-MIT) or the
[Apache License 2.0](LICENSE-APACHE), at your option. Notices for third-party libraries, the
bundled Inter and Noto Sans fonts, and Twemoji graphics are in
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
