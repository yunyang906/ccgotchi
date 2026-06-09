<p align="center">
  <img src="assets/logo.svg" alt="ccgotchi" width="680">
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
- **Context window** bar + token count, and **session cost** `$` — these also work on **API mode**, where there are no 5h/weekly windows (those segments are auto-omitted).
- **An animated pet** pinned to the right. Its face/health = `100 − your most-used window`, so it visibly reacts as you work. Eyes blink, mouth moves; it re-animates every second.
- **18 species** (the Claude Buddy roster): cat, chonk, rabbit, duck, goose, owl, penguin, turtle, snail, dragon, octopus, axolotl, ghost, robot, blob, cactus, mushroom, capybara.
- **✨ Shiny mode** — a per-character flowing rainbow (truecolor), the hidden variant.
- **Themeable bars**: dots / block / shade / square / slant / battery, in color or mono.

## Install

```bash
cargo install --git https://github.com/yunyang906/ccgotchi
```

Or build from source:

```bash
git clone https://github.com/yunyang906/ccgotchi
cd ccgotchi && cargo build --release
# binary at target/release/ccgotchi
```

## Setup

```bash
ccgotchi setup       # writes the statusLine into ~/.claude/settings.json
ccgotchi restore     # undo it (restores your previous statusLine)
```

`setup` adds (and backs up any existing statusLine):

```json
{
  "statusLine": {
    "type": "command",
    "command": "ccgotchi statusline",
    "refreshInterval": 1
  }
}
```

`refreshInterval: 1` lets the pet keep animating while idle. Open a new Claude Code session (or wait a second) to see it.

## Configuration

Run **`ccgotchi config`** for an interactive menu (type a number to change a
setting) — the no-tray equivalent of a settings panel. Or set anything directly:

```bash
ccgotchi pet cat            # cat|chonk|rabbit|duck|goose|owl|penguin|turtle|snail|
                            # dragon|octopus|axolotl|ghost|robot|blob|cactus|mushroom|capybara|off
ccgotchi shiny on           # rainbow pet (on|off)
ccgotchi barstyle dots      # dots|block|shade|square|slant|battery
ccgotchi barcolor auto      # auto (green/yellow/red by usage) | mono
ccgotchi resetfmt eta       # eta | arrow (↻) | paren | cn (余) | off
ccgotchi meter both         # both | tokens | cost | off
ccgotchi lang en            # en | zh | ja | ko (auto-detected from $LANG)
ccgotchi config             # interactive menu (or `config show` to just print)
```

### Internationalization

Segment labels are localized. The language is auto-detected from `$LANG` /
`$LC_ALL` (falls back to English), or set explicitly with `ccgotchi lang <code>`:

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
