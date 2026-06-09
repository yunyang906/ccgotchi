<p align="center">
  <img src="assets/logo.svg" alt="ccgotchi" width="680">
</p>

<p align="center">
  A <a href="https://docs.claude.com/en/docs/claude-code">Claude Code</a> <b>statusline</b> with usage progress bars and an animated ASCII <b>pet</b> — a little Tamagotchi for your terminal.
</p>

The pet stays happy while you have headroom and gets sick as you burn through your quota. 18 species, plus a hidden **shiny** (rainbow) mode.

```
5h ●●●○○○○○○○ 25% 2h8m  ·  7d ●●●●●●●●○○ 77% 1d19h  ·  ctx ●●●●●●○○○○ 63% (634k)  ·  $84.44        /\_/\
                                                                                                    ( ^o^ )
                                                                                                     > w <
```

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

All settings are single files under `~/.config/ccgotchi/`, set via subcommands:

```bash
ccgotchi pet cat            # cat|chonk|rabbit|duck|goose|owl|penguin|turtle|snail|
                            # dragon|octopus|axolotl|ghost|robot|blob|cactus|mushroom|capybara|off
ccgotchi shiny on           # rainbow pet (on|off)
ccgotchi barstyle dots      # dots|block|shade|square|slant|battery
ccgotchi barcolor auto      # auto (green/yellow/red by usage) | mono
ccgotchi resetfmt eta       # eta | arrow (↻) | paren | cn (余) | off
ccgotchi meter both         # both | tokens | cost | off
ccgotchi config             # show current settings
```

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

## License

MIT — see [LICENSE](LICENSE).
