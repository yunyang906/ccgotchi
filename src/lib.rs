//! ccgotchi — a Claude Code statusline with usage progress bars and an
//! animated ASCII pet (a Tamagotchi for your terminal).
//!
//! Pure stdin -> stdout: Claude Code pipes its statusLine JSON on stdin, and we
//! print one (multi-)line: 5h / weekly / context / cost progress bars plus an
//! ASCII pet whose health = how much quota you have left. No hardware, no daemon.

use std::fs;
use std::path::PathBuf;

// ---------- paths & config ----------

pub fn home_dir() -> PathBuf {
    for var in ["HOME", "USERPROFILE"] {
        if let Ok(h) = std::env::var(var) {
            if !h.is_empty() {
                return PathBuf::from(h);
            }
        }
    }
    PathBuf::from(".")
}

/// Config directory: `$XDG_CONFIG_HOME/ccgotchi` or `~/.config/ccgotchi`.
pub fn base_dir() -> PathBuf {
    if let Ok(x) = std::env::var("XDG_CONFIG_HOME") {
        if !x.is_empty() {
            return PathBuf::from(x).join("ccgotchi");
        }
    }
    home_dir().join(".config").join("ccgotchi")
}

fn read_cfg(name: &str, default: &str) -> String {
    fs::read_to_string(base_dir().join(name))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| default.to_string())
}

fn write_cfg(name: &str, val: &str) {
    let _ = fs::create_dir_all(base_dir());
    let _ = fs::write(base_dir().join(name), val.trim());
}

pub fn get_bar_style() -> String {
    read_cfg("bar_style", "dots")
}
pub fn set_bar_style(v: &str) {
    write_cfg("bar_style", v)
}
pub fn get_bar_color() -> String {
    read_cfg("bar_color", "auto")
}
pub fn set_bar_color(v: &str) {
    write_cfg("bar_color", v)
}
pub fn get_reset_fmt() -> String {
    read_cfg("reset_fmt", "eta")
}
pub fn set_reset_fmt(v: &str) {
    write_cfg("reset_fmt", v)
}
/// Per-segment visibility. `seg` ∈ 5h | 7d | ctx (each default on).
pub fn get_show(seg: &str) -> bool {
    read_cfg(&format!("show_{seg}"), "on") == "on"
}
pub fn set_show(seg: &str, on: bool) {
    write_cfg(&format!("show_{seg}"), if on { "on" } else { "off" })
}
/// Pet colour: "auto" (by health) or a preset name (orange/pink/blue/…).
pub fn get_pet_color() -> String {
    read_cfg("pet_color", "auto")
}
pub fn set_pet_color(v: &str) {
    write_cfg("pet_color", v)
}
pub fn get_pet() -> String {
    read_cfg("pet", "cat")
}
pub fn set_pet(v: &str) {
    write_cfg("pet", v)
}
pub fn get_pet_shiny() -> bool {
    read_cfg("pet_shiny", "off") == "on"
}
pub fn set_pet_shiny(on: bool) {
    write_cfg("pet_shiny", if on { "on" } else { "off" })
}

/// UI language. Explicit config wins, else auto-detected from the locale, else `en`.
pub fn get_lang() -> String {
    fs::read_to_string(base_dir().join("lang"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(detect_lang)
}
pub fn set_lang(v: &str) {
    write_cfg("lang", v)
}

/// Guess the language from the system locale. Prefers the real OS locale (so it
/// works for GUI apps launched from Finder/Explorer, which don't inherit shell
/// env vars), then falls back to `$LC_ALL` / `$LC_MESSAGES` / `$LANG`
/// (e.g. `zh_CN.UTF-8`). Recognizes zh/ja/ko, else English.
fn detect_lang() -> String {
    let mut tags: Vec<String> = Vec::new();
    if let Some(l) = sys_locale::get_locale() {
        tags.push(l.to_lowercase()); // e.g. "zh-cn", "ja", "ko-kr"
    }
    for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(v) = std::env::var(var) {
            tags.push(v.to_lowercase());
        }
    }
    for v in tags {
        if v.starts_with("zh") {
            return "zh".to_string();
        }
        if v.starts_with("ja") {
            return "ja".to_string();
        }
        if v.starts_with("ko") {
            return "ko".to_string();
        }
    }
    "en".to_string()
}

/// Segment labels (five-hour, seven-day, context) per language.
/// To add a language, add a match arm — that's the whole localization surface.
fn labels(lang: &str) -> (&'static str, &'static str, &'static str) {
    match lang {
        "zh" => ("5h", "周", "上下文"),
        "ja" => ("5h", "週", "文脈"),
        "ko" => ("5h", "주", "컨텍스트"),
        _ => ("5h", "7d", "ctx"), // en (default)
    }
}

pub fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ---------- progress bars ----------

/// Bar fill color by usage: <50% green, 50-80% yellow, >=80% red.
fn quota_color(pct: f64) -> &'static str {
    if pct >= 80.0 {
        "\x1b[31m"
    } else if pct >= 50.0 {
        "\x1b[33m"
    } else {
        "\x1b[32m"
    }
}

/// style -> (filled, empty, left wall, right wall)
fn bar_glyphs(style: &str) -> (&'static str, &'static str, &'static str, &'static str) {
    match style {
        "block" => ("█", "░", "", ""),
        "shade" => ("█", "▒", "", ""),
        "square" => ("▮", "▯", "", ""),
        "slant" => ("▰", "▱", "", ""),
        "battery" => ("█", "░", "▕", "▏"),
        _ => ("●", "○", "", ""), // dots (default)
    }
}

pub fn quota_bar(pct: f64, width: usize) -> String {
    quota_bar_full(pct, width, &get_bar_style(), &get_bar_color())
}

/// Explicit style, color fixed to "auto" (handy for tests).
pub fn quota_bar_styled(pct: f64, width: usize, style: &str) -> String {
    quota_bar_full(pct, width, style, "auto")
}

/// Render a bar. `color == "mono"` => filled part is not colored by usage.
pub fn quota_bar_full(pct: f64, width: usize, style: &str, color: &str) -> String {
    let p = pct.clamp(0.0, 100.0);
    let filled = (((p / 100.0) * width as f64).round() as usize).min(width);
    let empty = width - filled;
    let (f, e, lw, rw) = bar_glyphs(style);
    let fill_col = if color == "mono" { "" } else { quota_color(p) };
    format!(
        "{}{}{}\x1b[0m\x1b[2m{}\x1b[0m{}",
        lw,
        fill_col,
        f.repeat(filled),
        e.repeat(empty),
        rw,
    )
}

// ---------- reset countdown ----------

/// Compact "time until reset": `3d2h` / `2h30m` / `15m` / `<1m`.
pub fn fmt_eta(secs: i64) -> String {
    if secs <= 0 {
        return "0m".to_string();
    }
    if secs < 60 {
        return "<1m".to_string();
    }
    let d = secs / 86400;
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    if d > 0 {
        if h > 0 {
            format!("{}d{}h", d, h)
        } else {
            format!("{}d", d)
        }
    } else if h > 0 {
        if m > 0 {
            format!("{}h{}m", h, m)
        } else {
            format!("{}h", h)
        }
    } else {
        format!("{}m", m)
    }
}

/// eta=15m / arrow=↻15m / paren=(15m) / cn=余15m / off=hidden
fn render_reset(fmt: &str, secs: i64) -> String {
    if fmt == "off" {
        return String::new();
    }
    let t = fmt_eta(secs);
    match fmt {
        "arrow" => format!("↻{}", t),
        "paren" => format!("({})", t),
        "cn" => format!("余{}", t),
        _ => t,
    }
}

// ---------- data ----------

/// The usage windows the statusline shows. Rate-limit fields are `None` on API
/// (pay-as-you-go) usage, which has no 5h/weekly windows.
#[derive(Default, Debug, Clone)]
pub struct StatusData {
    pub five: Option<f64>,
    pub five_reset: u64,
    pub seven: Option<f64>,
    pub seven_reset: u64,
    pub ctx_pct: Option<f64>,
}

/// Parse Claude Code's statusLine JSON into a [`StatusData`].
pub fn parse(v: &serde_json::Value) -> StatusData {
    let rl = v.get("rate_limits");
    let f5 = rl.and_then(|r| r.get("five_hour"));
    let five = f5
        .and_then(|x| x.get("used_percentage"))
        .and_then(|x| x.as_f64());
    let five_reset = f5
        .and_then(|x| x.get("resets_at"))
        .and_then(|x| x.as_u64())
        .unwrap_or(0);
    let s7 = rl.and_then(|r| r.get("seven_day"));
    let seven = s7
        .and_then(|x| x.get("used_percentage"))
        .and_then(|x| x.as_f64());
    let seven_reset = s7
        .and_then(|x| x.get("resets_at"))
        .and_then(|x| x.as_u64())
        .unwrap_or(0);
    let ctx_pct = v
        .get("context_window")
        .and_then(|c| c.get("used_percentage"))
        .and_then(|x| x.as_f64());
    StatusData {
        five,
        five_reset,
        seven,
        seven_reset,
        ctx_pct,
    }
}

// ---------- the pet ----------

/// Safety gap (columns) kept to the right of the pet so it isn't clipped at the
/// last column / by the renderer's built-in left padding.
const PET_RIGHT_GAP: usize = 4;

/// Per-render animation counter (persisted), so the pet visibly moves.
pub fn next_anim_frame() -> u64 {
    let p = base_dir().join("anim_frame");
    let cur = fs::read_to_string(&p)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0);
    let next = cur.wrapping_add(1);
    let _ = fs::create_dir_all(base_dir());
    let _ = fs::write(&p, next.to_string());
    next
}

/// (eyes, mouth) by health + frame. 4-frame loop = blink / open mouth / wiggle.
fn pet_frames(health: f64, frame: usize) -> (&'static str, &'static str) {
    let f = frame % 4;
    let (eyes, mouth): ([&str; 4], [&str; 4]) = if health >= 60.0 {
        (["^.^", "^o^", "-.-", "^o^"], [" > ^ <", " > w <", " > ^ <", " > w <"])
    } else if health >= 30.0 {
        (["o.o", "o.O", "O.o", "o.o"], [" > ^ <", " > o <", " > ^ <", " > o <"])
    } else if health >= 10.0 {
        (["T.T", "T_T", "T.T", ";_;"], [" > _ <", " > ~ <", " > _ <", " > ~ <"])
    } else {
        (["x.x", "X.X", "x.x", "+_+"], [" > _ <", " > . <", " > _ <", " > . <"])
    };
    (eyes[f], mouth[f])
}

/// ASCII art per species — variable height (most 3 lines; a few need 4 for the
/// signature feature). `eyes` is the animated face; `mouth` is the cat's bottom.
fn pet_art(animal: &str, eyes: &str, mouth: &str) -> Vec<String> {
    let s = |x: &str| x.to_string();
    match animal {
        "chonk" => vec![s(" /\\_/\\"), format!("( {}  )", eyes), s("(______)")],
        "rabbit" => vec![s("(\\_/)"), format!("({})", eyes), s("(\")_(\")")],
        "goose" => vec![s(" ,_"), format!("({})>", eyes), s(" <__>")],
        "owl" => vec![s(",_,_,"), format!("({})", eyes), s(" v v")],
        "penguin" => vec![s("  _"), format!("({})", eyes), s("<(_)>")],
        "turtle" => vec![s("  __"), format!("({})>", eyes), s(" m  m")],
        "dragon" => vec![s("\\/\\/"), format!("({})~", eyes), s(" >v<")],
        "ghost" => vec![s(" .--."), format!("({})", eyes), s(" ^v^v")],
        "robot" => vec![s(" _^_"), format!("[{}]", eyes), s(" |_|")],
        "blob" => vec![s("  __"), format!("({})", eyes), s(" \\__/")],
        "cactus" => vec![s(" J|L"), format!("({})", eyes), s("[___]")],
        // ---- 4-line species (signature feature needs the extra row) ----
        "duck" => vec![s("  __"), format!("({})", eyes), s(" <==>"), s("  ~~")],
        "snail" => vec![s(" 6 6"), format!("({})", eyes), s(" (@@@)"), s("~~~~~~")],
        "octopus" => vec![s(" ,---."), format!("( {} )", eyes), s(" }}}}}"), s(" }} }}")],
        "axolotl" => vec![s("\\v/ \\v/"), format!("( {} )", eyes), s("  \\_/"), s("   >")],
        "mushroom" => vec![s(" .-~-."), s("(_____)"), format!("( {} )", eyes), s("  |_|")],
        "capybara" => vec![s(" c   c"), format!("( {} )", eyes), s("(_____)"), s(" u   u")],
        // cat (default, pointy ears) — bottom line uses the animated mouth
        _ => vec![s(" /\\_/\\"), format!("( {} )", eyes), mouth.to_string()],
    }
}

/// Named pet colours -> RGB. "auto" (and any unknown) returns None so the
/// caller falls back to health-based / mono colouring.
fn pet_color_rgb(name: &str) -> Option<(u8, u8, u8)> {
    Some(match name {
        "orange" => (245, 159, 67),
        "pink" => (255, 126, 182),
        "red" => (248, 81, 73),
        "yellow" => (247, 227, 89),
        "green" => (63, 185, 80),
        "cyan" => (57, 197, 207),
        "blue" => (88, 166, 255),
        "purple" => (163, 113, 247),
        "white" => (230, 237, 243),
        "gray" => (139, 148, 158),
        _ => return None, // "auto"
    })
}

/// Smooth rainbow via phase-shifted sines; `p` is position + frame offset.
fn rainbow_rgb(p: usize) -> (u8, u8, u8) {
    let t = p as f64 * 0.55;
    let f = |phase: f64| ((t + phase).sin() * 127.0 + 128.0) as u8;
    (f(0.0), f(2.094_395), f(4.188_790)) // +120° / +240°
}

/// Colour each non-space char of a line with a flowing rainbow (the "shiny" pet).
fn rainbow_paint(s: &str, offset: usize) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c == ' ' {
            out.push(' ');
            continue;
        }
        let (r, g, b) = rainbow_rgb(i + offset);
        out.push_str(&format!("\x1b[38;2;{};{};{}m{}", r, g, b, c));
    }
    out.push_str("\x1b[0m");
    out
}

/// Approximate terminal display width (strip ANSI; CJK/emoji count as 2).
/// Used to right-align the pet's head onto the usage line.
fn display_width(s: &str) -> usize {
    let mut w = 0usize;
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            for c2 in chars.by_ref() {
                if c2 == 'm' {
                    break;
                }
            }
            continue;
        }
        let u = c as u32;
        let wide = (0x1100..=0x115F).contains(&u)
            || (0x2E80..=0xA4CF).contains(&u)
            || (0xAC00..=0xD7A3).contains(&u)
            || (0xF900..=0xFAFF).contains(&u)
            || (0xFE30..=0xFE4F).contains(&u)
            || (0xFF00..=0xFF60).contains(&u)
            || (0xFFE0..=0xFFE6).contains(&u)
            || (0x1F000..=0x1FAFF).contains(&u)
            || (0x2600..=0x27BF).contains(&u);
        w += if wide { 2 } else { 1 };
    }
    w
}

// ---------- the statusline ----------

/// Render the statusline, reading all options from config and terminal width
/// from `$COLUMNS` (Claude Code sets it; v2.1.153+). Hidden segments are
/// filtered out here by nulling their data.
pub fn format_statusline(d: &StatusData, now: u64) -> String {
    let columns = std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|c| *c > 0)
        .unwrap_or(80);
    let frame = next_anim_frame() as usize;
    // apply per-segment visibility by nulling hidden fields
    let shown = StatusData {
        five: if get_show("5h") { d.five } else { None },
        seven: if get_show("7d") { d.seven } else { None },
        ctx_pct: if get_show("ctx") { d.ctx_pct } else { None },
        ..*d
    };
    format_statusline_cfg(
        &shown,
        now,
        &get_bar_style(),
        &get_bar_color(),
        &get_reset_fmt(),
        &get_pet(),
        columns,
        frame,
        get_pet_shiny(),
        &get_lang(),
        &get_pet_color(),
    )
}

/// Same as [`format_statusline`] but every option is explicit (for testing).
/// Segment visibility is expressed by which `d` fields are `Some`.
/// `pet`: off | cat | rabbit | …  `pet_color`: auto | orange | pink | …
#[allow(clippy::too_many_arguments)]
pub fn format_statusline_cfg(
    d: &StatusData,
    now: u64,
    style: &str,
    color: &str,
    reset_fmt: &str,
    pet: &str,
    columns: usize,
    frame: usize,
    shiny: bool,
    lang: &str,
    pet_color: &str,
) -> String {
    const W: usize = 10;
    let (label_5h, label_7d, label_ctx) = labels(lang);
    let quota_seg = |label: &str, pct: f64, reset: u64| -> String {
        let mut s = format!("{} {} {:.0}%", label, quota_bar_full(pct, W, style, color), pct);
        if reset > now {
            let r = render_reset(reset_fmt, (reset - now) as i64);
            if !r.is_empty() {
                s.push_str(&format!(" \x1b[2m{}\x1b[0m", r)); // dim reset
            }
        }
        s
    };

    let mut parts: Vec<String> = Vec::new();
    if let Some(p) = d.five {
        parts.push(quota_seg(label_5h, p, d.five_reset));
    }
    if let Some(p) = d.seven {
        parts.push(quota_seg(label_7d, p, d.seven_reset));
    }
    if let Some(p) = d.ctx_pct {
        parts.push(format!("{} {} {:.0}%", label_ctx, quota_bar_full(p, W, style, color), p));
    }

    let mut line = if parts.is_empty() {
        "ccgotchi".to_string()
    } else {
        parts.join("  \x1b[2m·\x1b[0m  ") // dim separator
    };

    // Pet: head merges onto the usage line (right side), body two lines below.
    // health = 100 - the most-used window; burn quota and the pet gets sick.
    if pet != "off" {
        let stress = [d.five, d.seven, d.ctx_pct]
            .into_iter()
            .flatten()
            .fold(0.0_f64, f64::max);
        let health = (100.0 - stress).clamp(0.0, 100.0);
        let (eyes, mouth) = pet_frames(health, frame);
        let art = pet_art(pet, eyes, mouth);
        let block_w = art.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let indent = columns.saturating_sub(block_w + PET_RIGHT_GAP);
        let paint = |s: &str, row: usize| -> String {
            if shiny {
                rainbow_paint(s, frame + row * 2)
            } else if let Some((r, g, b)) = pet_color_rgb(pet_color) {
                format!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, s) // custom color
            } else if color == "mono" {
                s.to_string()
            } else {
                format!("{}{}\x1b[0m", quota_color(stress), s) // auto: by health
            }
        };
        // Pad with braille-blank U+2800 (not whitespace, survives the renderer's
        // leading-whitespace trim) so right-alignment actually sticks.
        let usage_w = display_width(&line);
        line.push_str(&"\u{2800}".repeat(indent.saturating_sub(usage_w)));
        line.push_str(&paint(&art[0], 0));
        let body_pad = "\u{2800}".repeat(indent);
        for (i, l) in art[1..].iter().enumerate() {
            line.push('\n');
            line.push_str(&body_pad);
            line.push_str(&paint(l, i + 1));
        }
    }
    line
}

// ---------- Claude Code settings.json integration ----------

pub fn claude_settings_path() -> PathBuf {
    home_dir().join(".claude").join("settings.json")
}

fn read_settings() -> serde_json::Value {
    fs::read_to_string(claude_settings_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::json!({}))
}

fn write_settings(v: &serde_json::Value) {
    let p = claude_settings_path();
    if let Some(dir) = p.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Ok(s) = serde_json::to_string_pretty(v) {
        let _ = fs::write(p, s);
    }
}

/// Point Claude Code's statusLine at `<cli_path> statusline` (refreshInterval=1
/// so the pet keeps animating). Backs up any existing statusLine once.
pub fn install_statusline(cli_path: &str) {
    let mut v = read_settings();
    if let Some(obj) = v.as_object_mut() {
        if let Some(existing) = obj.get("statusLine") {
            let backup = base_dir().join("statusLine.backup.json");
            // don't clobber a real backup with our own command
            let is_ours = existing
                .get("command")
                .and_then(|c| c.as_str())
                .map(|c| c.contains("ccgotchi"))
                .unwrap_or(false);
            if !backup.exists() && !is_ours {
                let _ = fs::create_dir_all(base_dir());
                let _ = fs::write(&backup, existing.to_string());
            }
        }
        obj.insert(
            "statusLine".to_string(),
            serde_json::json!({
                "type": "command",
                "command": format!("\"{}\" statusline", cli_path),
                "refreshInterval": 1
            }),
        );
    }
    write_settings(&v);
}

/// Undo [`install_statusline`]: restore the backed-up statusLine, else remove ours.
pub fn restore_statusline() {
    let mut v = read_settings();
    let backup = base_dir().join("statusLine.backup.json");
    if let Some(obj) = v.as_object_mut() {
        match fs::read_to_string(&backup)
            .ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        {
            Some(orig) => {
                obj.insert("statusLine".to_string(), orig);
            }
            None => {
                obj.remove("statusLine");
            }
        }
    }
    write_settings(&v);
    let _ = fs::remove_file(&backup);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strip(s: &str) -> String {
        let mut out = String::new();
        let mut chars = s.chars();
        while let Some(c) = chars.next() {
            if c == '\x1b' {
                for c2 in chars.by_ref() {
                    if c2 == 'm' {
                        break;
                    }
                }
            } else {
                out.push(c);
            }
        }
        out
    }

    #[test]
    fn bar_fill() {
        assert_eq!(strip(&quota_bar_styled(0.0, 10, "dots")), "○○○○○○○○○○");
        assert_eq!(strip(&quota_bar_styled(100.0, 10, "dots")), "●●●●●●●●●●");
        assert_eq!(strip(&quota_bar_styled(75.0, 10, "dots")), "●●●●●●●●○○");
        assert_eq!(strip(&quota_bar_styled(40.0, 10, "block")), "████░░░░░░");
        assert_eq!(strip(&quota_bar_styled(40.0, 10, "battery")), "▕████░░░░░░▏");
    }

    #[test]
    fn eta_format() {
        assert_eq!(fmt_eta(-5), "0m");
        assert_eq!(fmt_eta(30), "<1m");
        assert_eq!(fmt_eta(15 * 60), "15m");
        assert_eq!(fmt_eta(2 * 3600 + 30 * 60), "2h30m");
        assert_eq!(fmt_eta(3 * 86400 + 2 * 3600), "3d2h");
    }

    #[test]
    fn segments_render_present_fields() {
        // _cfg renders whatever fields are present; hiding a segment = its field is None.
        let f = |d: &StatusData| {
            strip(&format_statusline_cfg(d, 0, "dots", "auto", "eta", "off", 80, 0, false, "en", "auto"))
        };
        let full = StatusData {
            five: Some(20.0),
            seven: Some(60.0),
            ctx_pct: Some(50.0),
            ..Default::default()
        };
        assert!(f(&full).contains("5h") && f(&full).contains("7d") && f(&full).contains("ctx"));
        // hide ctx
        let no_ctx = StatusData { ctx_pct: None, ..full.clone() };
        assert!(!f(&no_ctx).contains("ctx") && f(&no_ctx).contains("5h"));
        // hide 7d
        let no_7d = StatusData { seven: None, ..full };
        assert!(!f(&no_7d).contains("7d") && f(&no_7d).contains("ctx"));
    }

    #[test]
    fn pet_custom_color() {
        let d = StatusData { five: Some(20.0), ..Default::default() };
        // custom colour emits a truecolor code even when not shiny / mono bars
        let raw = format_statusline_cfg(&d, 0, "dots", "mono", "eta", "cat", 80, 0, false, "en", "orange");
        assert!(raw.contains("\x1b[38;2;245;159;67m"));
        assert!(pet_color_rgb("auto").is_none()); // auto -> health colour
        assert!(pet_color_rgb("blue").is_some());
    }

    #[test]
    fn api_mode_omits_rate_limits() {
        // No rate_limits -> only ctx, no 5h/7d.
        let d = StatusData {
            ctx_pct: Some(40.0),
            ..Default::default()
        };
        let line = strip(&format_statusline_cfg(&d, 0, "dots", "auto", "eta", "off", 80, 0, false, "en", "auto"));
        assert!(line.contains("ctx") && !line.contains("5h") && !line.contains("7d"));
    }

    #[test]
    fn pet_frames_animate() {
        assert_eq!(pet_frames(100.0, 0).0, "^.^");
        assert_eq!(pet_frames(50.0, 0).0, "o.o");
        assert_eq!(pet_frames(20.0, 0).0, "T.T");
        assert_eq!(pet_frames(5.0, 0).0, "x.x");
        assert_ne!(pet_frames(100.0, 0), pet_frames(100.0, 1)); // moves
        assert_eq!(pet_frames(100.0, 0), pet_frames(100.0, 4)); // wraps
        for h in [100.0, 50.0, 20.0, 5.0] {
            for f in 0..4 {
                let (e, m) = pet_frames(h, f);
                assert_eq!(e.chars().count(), 3);
                assert_eq!(m.chars().count(), 6);
            }
        }
    }

    #[test]
    fn pet_merges_onto_usage_line() {
        let d = StatusData {
            five: Some(20.0),
            seven: Some(91.0), // worst -> health 9 -> sick face
            ..Default::default()
        };
        let out = strip(&format_statusline_cfg(&d, 0, "dots", "auto", "eta", "cat", 80, 0, false, "en", "auto"));
        let rows: Vec<&str> = out.lines().collect();
        assert_eq!(rows.len(), 3); // usage+head, then 2 body rows
        assert!(rows[0].contains("5h") && rows[0].contains("/\\_/\\"));
        assert!(rows[1].contains("( x.x )"));
        let off = strip(&format_statusline_cfg(&d, 0, "dots", "auto", "eta", "off", 80, 0, false, "en", "auto"));
        assert_eq!(off.lines().count(), 1);
    }

    #[test]
    fn shiny_is_rainbow() {
        let d = StatusData {
            five: Some(50.0),
            ..Default::default()
        };
        let raw = format_statusline_cfg(&d, 0, "dots", "auto", "eta", "cat", 80, 0, true, "en", "auto");
        assert!(raw.contains("\x1b[38;2;")); // truecolor
        assert_ne!(rainbow_paint("abc", 0), rainbow_paint("abc", 1)); // flows
    }

    #[test]
    fn i18n_labels() {
        let d = StatusData {
            five: Some(10.0),
            seven: Some(20.0),
            ctx_pct: Some(30.0),
            ..Default::default()
        };
        let line = |lang: &str| {
            strip(&format_statusline_cfg(
                &d, 0, "dots", "auto", "eta", "off", 80, 0, false, lang, "auto",
            ))
        };
        assert!(line("en").contains("7d") && line("en").contains("ctx"));
        assert!(line("zh").contains("周") && line("zh").contains("上下文"));
        assert_eq!(labels("ja").1, "週");
        assert_eq!(labels("ko").2, "컨텍스트");
        assert_eq!(labels("xx"), labels("en")); // unknown falls back to en
    }

    #[test]
    fn display_width_basics() {
        assert_eq!(display_width("abc"), 3);
        assert_eq!(display_width("上下文"), 6);
        assert_eq!(display_width("\x1b[32m●\x1b[0m"), 1);
    }
}
