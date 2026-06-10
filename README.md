<p align="center">
  <img src="assets/logo.svg" alt="ccgotchi" width="680">
</p>

<p align="center">
  <b>English</b> · <a href="README.zh-CN.md">简体中文</a> · <a href="README.ja.md">日本語</a> · <a href="README.ko.md">한국어</a>
</p>

<p align="center">
  A <a href="https://docs.claude.com/en/docs/claude-code">Claude Code</a> <b>statusline</b> with usage progress bars and an animated ASCII <b>pet</b> — a little Tamagotchi for your terminal.
</p>

The pet stays happy while you have headroom and gets sick as you burn through your quota. 18 species, plus a hidden **shiny** (rainbow) mode.

<p align="center">
  <img src="assets/demo.svg" alt="ccgotchi statusline demo" width="900">
</p>

It reads the JSON Claude Code hands to status-line commands and prints one (multi-)line. No daemon, no telemetry — a single tiny binary.

## Features

- **Usage bars** for the **5-hour** and **7-day (weekly)** rate-limit windows (Pro/Max), with a **reset countdown**.
- **Context window** bar (works on API mode too, where there are no 5h/weekly windows — those segments are auto-omitted).
- **An animated pet** pinned to the right. Its face/health = `100 − your most-used window`, so it visibly reacts as you work. Eyes blink, mouth moves; it re-animates every second.
- **18 species** (the Claude Buddy roster): cat, chonk, rabbit, duck, goose, owl, penguin, turtle, snail, dragon, octopus, axolotl, ghost, robot, blob, cactus, mushroom, capybara.
- **✨ Shiny mode** — a per-character flowing rainbow (truecolor), the hidden variant.
- **Pick the pet's colour** — auto (by health) or a fixed colour (orange, pink, blue, …).
- **Model name** on the left (e.g. `Opus 4.8`) — always see which model you're on.
- **Show/hide any segment** — toggle model / 5h / 7d / context independently.
- **Themeable bars**: dots / block / shade / square / slant / battery, in color or mono.

## Install

### macOS app (menu-bar tray) — recommended

Build the app, then configure everything by clicking the menu-bar icon — no
commands to remember:

```bash
git clone https://github.com/yunyang906/ccgotchi
cd ccgotchi
cargo build --release --workspace
./package_macos.sh
open build/ccgotchi.app
```

Launching it wires the statusline into Claude Code automatically and adds a 🐈
menu-bar icon. From the menu: pick a pet, toggle ✨ shiny, change bar style /
colors / language — changes apply live. "Restore (uninstall)" undoes it.

### CLI (any platform)

```bash
cargo install --git https://github.com/yunyang906/ccgotchi ccgotchi
ccgotchi setup       # wire the statusline into ~/.claude/settings.json
ccgotchi restore     # undo (restores your previous statusLine)
```

`setup` writes (backing up any existing statusLine):

```json
{ "statusLine": { "type": "command", "command": "ccgotchi statusline", "refreshInterval": 1 } }
```

`refreshInterval: 1` lets the pet keep animating while idle. Open a new Claude Code session (or wait a second) to see it.

> Prebuilt downloads are on the [Releases](https://github.com/yunyang906/ccgotchi/releases) page:
> - **Tray app** — `ccgotchi-app-macos-arm64`, `ccgotchi-app-macos-intel`, `ccgotchi-app-windows-x64`
> - **CLI** — `ccgotchi-cli-macos-arm64`, `ccgotchi-cli-macos-intel`, `ccgotchi-cli-windows-x64`, `ccgotchi-cli-linux-x64`

The downloaded macOS app isn't notarized (no paid Apple Developer cert), so Gatekeeper flags it as *"…is damaged and can't be opened"*. Clear the quarantine attribute once, then open it:

```bash
xattr -dr com.apple.quarantine /path/to/ccgotchi.app
```

**Windows tray app:** download `ccgotchi-app-windows-x64.zip`, unzip (keep both `.exe` files in the same folder), and run `ccgotchi-app.exe` — it adds a tray icon and wires up Claude Code. Prefer the CLI? `ccgotchi.exe` is a command-line tool: run it from a terminal (`ccgotchi.exe setup`), not by double-clicking — double-clicking only flashes a console (it printed `--help` and exited, not a crash). **Linux** is CLI-only for now.

## Configuration

Use the **menu-bar app** to point-and-click, or set anything from the CLI:

```bash
ccgotchi pet cat            # cat|chonk|rabbit|...|capybara|off (18 species)
ccgotchi petcolor auto      # auto (by health) | orange|pink|red|yellow|green|cyan|blue|purple|white|gray
ccgotchi shiny on           # rainbow pet (on|off)
ccgotchi barstyle dots      # dots|block|shade|square|slant|battery
ccgotchi barcolor auto      # auto (green/yellow/red by usage) | mono
ccgotchi resetfmt eta       # eta | arrow (↻) | paren | cn (余) | off
ccgotchi show ctx off       # hide/show a segment: model|5h|7d|ctx  (on|off)
ccgotchi lang en            # en | zh | ja | ko (auto-detected from $LANG)
ccgotchi config             # print current settings
```

### Internationalization

Segment labels and the tray menu are localized. The language follows your
**system locale** by default — read from the OS, so it's correct even for the
app launched from Finder/Explorer (the CLI also honors `$LANG` / `$LC_ALL`).
Override it anytime with `ccgotchi lang <code>`:

| lang | 5h window | 7-day window | context |
|------|-----------|--------------|---------|
| `en` | `5h`      | `7d`         | `ctx`   |
| `zh` | `5h`      | `周`         | `上下文` |
| `ja` | `5h`      | `週`         | `文脈`   |
| `ko` | `5h`      | `주`         | `컨텍스트` |

Adding a language is a one-line PR: add a match arm to `labels()` in `src/lib.rs`.

## Pet health

`health = 100 − max(5h%, weekly%, context%)`:

| health | mood | example |
|--------|------|---------|
| ≥ 60   | happy | `( ^.^ )` |
| 30–60  | meh   | `( o.o )` |
| 10–30  | sick  | `( T.T )` |
| < 10   | dying | `( x.x )` |

## Notes / how it works

- Terminal width comes from the `COLUMNS` env var, which Claude Code sets before running the command (v2.1.153+). That's how the pet right-aligns.
- Claude Code trims leading whitespace from statusline output ([#29206](https://github.com/anthropics/claude-code/issues/29206)), so right-alignment padding uses braille-blank `U+2800` (not whitespace, survives the trim, renders blank).

## Credits

Inspired by [ccstatusline](https://github.com/sirmalloc/ccstatusline), [ccpet](https://github.com/terryso/ccpet), and Anthropic's Claude Buddy. Extracted from the [ClaudeLight](https://github.com/yunyang906/ClaudeLight) project (the hardware traffic-light client).

## Star History

<a href="https://star-history.com/#yunyang906/ccgotchi&Date">
  <img src="https://api.star-history.com/svg?repos=yunyang906/ccgotchi&type=Date" alt="Star History Chart" width="100%">
</a>

## License

MIT — see [LICENSE](LICENSE).
