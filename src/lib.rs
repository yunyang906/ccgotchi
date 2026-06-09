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
pub fn get_meter() -> String {
    read_cfg("meter", "both")
}
pub fn set_meter(v: &str) {
    write_cfg("meter", v)
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

/// Compact token count: 800 / 145k / 1.5M
pub fn fmt_tokens(n: u64) -> String {
    if n < 1000 {
        n.to_string()
    } else if n < 1_000_000 {
        format!("{:.0}k", n as f64 / 1000.0)
    } else {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    }
}

// ---------- data ----------

/// Everything the statusline can show. Rate-limit fields are `None` on API
/// (pay-as-you-go) usage, which has no 5h/weekly windows — those segments are
/// then simply omitted.
#[derive(Default, Debug, Clone)]
pub struct StatusData {
    pub five: Option<f64>,
    pub five_reset: u64,
    pub seven: Option<f64>,
    pub seven_reset: u64,
    pub ctx_pct: Option<f64>,
    pub ctx_tokens: Option<u64>,
    pub cost: Option<f64>,
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
    let cw = v.get("context_window");
    let ctx_pct = cw
        .and_then(|c| c.get("used_percentage"))
        .and_then(|x| x.as_f64());
    let ctx_tokens = cw
        .and_then(|c| c.get("total_input_tokens"))
        .and_then(|x| x.as_u64());
    let cost = v
        .get("cost")
        .and_then(|c| c.get("total_cost_usd"))
        .and_then(|x| x.as_f64());
    StatusData {
        five,
        five_reset,
        seven,
        seven_reset,
        ctx_pct,
        ctx_tokens,
        cost,
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

/// 3-line ASCII art per species (eyes go in the face line; `mouth` is the cat's
/// bottom line — other species have a fixed bottom). Roster mirrors Claude Buddy.
fn pet_art(animal: &str, eyes: &str, mouth: &str) -> [String; 3] {
    let (top, face, bottom): (&str, String, &str) = match animal {
        "chonk" => (" /\\_/\\", format!("( {}  )", eyes), "(______)"),
        "rabbit" => ("(\\_/)", format!("({})", eyes), "(\")_(\")"),
        "duck" => ("  _", format!("({})>", eyes), " ~~~~"),
        "goose" => (" _", format!("({})7", eyes), "  ^^"),
        "owl" => (",_,_,", format!("({})", eyes), " \" \""),
        "penguin" => ("  _", format!("({})", eyes), "<(_)>"),
        "turtle" => (",----.", format!("({})>", eyes), "  ^^"),
        "snail" => ("  ,,", format!("({})@", eyes), " ~~~~~"),
        "dragon" => (",/\\/\\,", format!("({})~", eyes), " >vv<"),
        "octopus" => ("  ___", format!("({})", eyes), " }}}}}"),
        "axolotl" => (" \\^v^/", format!("({})", eyes), "  <><"),
        "ghost" => (" .--.", format!("({})", eyes), " ^v^v"),
        "robot" => (" _^_", format!("[{}]", eyes), " |_|"),
        "blob" => ("  __", format!("({})", eyes), " \\__/"),
        "cactus" => (" _,_", format!("({})", eyes), " |_|"),
        "mushroom" => (",----.", format!("({})", eyes), "  ||"),
        "capybara" => (" ____", format!("({}  )", eyes), " u  u"),
        // cat (default, pointy ears) — bottom line uses the animated mouth
        _ => (" /\\_/\\", format!("( {} )", eyes), mouth),
    };
    [top.to_string(), face, bottom.to_string()]
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

/// Render the statusline, reading style/color/reset/meter/pet/shiny from config
/// and terminal width from `$COLUMNS` (Claude Code sets it; v2.1.153+).
pub fn format_statusline(d: &StatusData, now: u64) -> String {
    let columns = std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|c| *c > 0)
        .unwrap_or(80);
    let frame = next_anim_frame() as usize;
    format_statusline_cfg(
        d,
        now,
        &get_bar_style(),
        &get_bar_color(),
        &get_reset_fmt(),
        &get_meter(),
        &get_pet(),
        columns,
        frame,
        get_pet_shiny(),
    )
}

/// Same as [`format_statusline`] but every option is explicit (for testing).
/// `meter`: both | tokens | cost | off. `pet`: off | cat | rabbit | ...
#[allow(clippy::too_many_arguments)]
pub fn format_statusline_cfg(
    d: &StatusData,
    now: u64,
    style: &str,
    color: &str,
    reset_fmt: &str,
    meter: &str,
    pet: &str,
    columns: usize,
    frame: usize,
    shiny: bool,
) -> String {
    const W: usize = 10;
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
    let show_tokens = matches!(meter, "both" | "tokens");
    let show_cost = matches!(meter, "both" | "cost");

    let mut parts: Vec<String> = Vec::new();
    if let Some(p) = d.five {
        parts.push(quota_seg("5h", p, d.five_reset));
    }
    if let Some(p) = d.seven {
        parts.push(quota_seg("7d", p, d.seven_reset));
    }
    if let Some(p) = d.ctx_pct {
        let mut s = format!("ctx {} {:.0}%", quota_bar_full(p, W, style, color), p);
        if show_tokens {
            if let Some(t) = d.ctx_tokens {
                s.push_str(&format!(" \x1b[2m({})\x1b[0m", fmt_tokens(t)));
            }
        }
        parts.push(s);
    }
    if show_cost {
        if let Some(c) = d.cost {
            parts.push(format!("${:.2}", c));
        }
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
            } else if color == "mono" {
                s.to_string()
            } else {
                format!("{}{}\x1b[0m", quota_color(stress), s)
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
    fn tokens_format() {
        assert_eq!(fmt_tokens(8), "8");
        assert_eq!(fmt_tokens(144893), "145k");
        assert_eq!(fmt_tokens(1_500_000), "1.5M");
    }

    #[test]
    fn meter_modes() {
        let d = StatusData {
            ctx_pct: Some(50.0),
            ctx_tokens: Some(144893),
            cost: Some(5.64),
            ..Default::default()
        };
        let f = |m: &str| strip(&format_statusline_cfg(&d, 0, "dots", "auto", "eta", m, "off", 80, 0, false));
        assert!(f("both").contains("145k") && f("both").contains("$5.64"));
        assert!(f("tokens").contains("145k") && !f("tokens").contains("$"));
        assert!(!f("cost").contains("145k") && f("cost").contains("$5.64"));
        assert!(f("off").contains("ctx"));
    }

    #[test]
    fn api_mode_omits_rate_limits() {
        // No rate_limits -> only ctx + cost, no 5h/7d.
        let d = StatusData {
            ctx_pct: Some(40.0),
            ctx_tokens: Some(200_000),
            cost: Some(1.0),
            ..Default::default()
        };
        let line = strip(&format_statusline_cfg(&d, 0, "dots", "auto", "eta", "both", "off", 80, 0, false));
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
        let out = strip(&format_statusline_cfg(&d, 0, "dots", "auto", "eta", "off", "cat", 80, 0, false));
        let rows: Vec<&str> = out.lines().collect();
        assert_eq!(rows.len(), 3); // usage+head, then 2 body rows
        assert!(rows[0].contains("5h") && rows[0].contains("/\\_/\\"));
        assert!(rows[1].contains("( x.x )"));
        let off = strip(&format_statusline_cfg(&d, 0, "dots", "auto", "eta", "off", "off", 80, 0, false));
        assert_eq!(off.lines().count(), 1);
    }

    #[test]
    fn shiny_is_rainbow() {
        let d = StatusData {
            five: Some(50.0),
            ..Default::default()
        };
        let raw = format_statusline_cfg(&d, 0, "dots", "auto", "eta", "off", "cat", 80, 0, true);
        assert!(raw.contains("\x1b[38;2;")); // truecolor
        assert_ne!(rainbow_paint("abc", 0), rainbow_paint("abc", 1)); // flows
    }

    #[test]
    fn display_width_basics() {
        assert_eq!(display_width("abc"), 3);
        assert_eq!(display_width("上下文"), 6);
        assert_eq!(display_width("\x1b[32m●\x1b[0m"), 1);
    }
}
