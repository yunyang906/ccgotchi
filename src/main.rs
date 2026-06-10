//! ccgotchi CLI.
//!
//!   ccgotchi statusline        read Claude Code JSON on stdin, print the line
//!   ccgotchi setup             install into ~/.claude/settings.json
//!   ccgotchi restore           undo setup (restore previous statusLine)
//!   ccgotchi config            show current settings
//!   ccgotchi pet <name>        cat | rabbit | duck | ... | off
//!   ccgotchi shiny on|off      rainbow (shiny) pet
//!   ccgotchi barstyle <s>      dots | block | shade | square | slant | battery
//!   ccgotchi barcolor auto|mono
//!   ccgotchi resetfmt <f>      eta | arrow | paren | cn | off
//!   ccgotchi show <seg> on|off model | 5h | 7d | ctx
//!   ccgotchi petcolor <c>      auto | orange | pink | blue | ...
//!   ccgotchi lang <l>          en | zh | ja | ko
//!
//! For point-and-click config, use the ccgotchi tray app instead.

use ccgotchi as cc;
use std::io::Read;

const PETS: &[&str] = &[
    "off", "cat", "chonk", "rabbit", "duck", "goose", "owl", "penguin", "turtle", "snail", "dragon",
    "octopus", "axolotl", "ghost", "robot", "blob", "cactus", "mushroom", "capybara",
];

const SEGS: &[&str] = &["model", "5h", "7d", "ctx"];

fn print_config() {
    let on = |b: bool| if b { "on" } else { "off" };
    println!("ccgotchi config ({}):", cc::base_dir().display());
    println!("  pet       = {}", cc::get_pet());
    println!("  petcolor  = {}", cc::get_pet_color());
    println!("  shiny     = {}", on(cc::get_pet_shiny()));
    println!("  barstyle  = {}", cc::get_bar_style());
    println!("  barcolor  = {}", cc::get_bar_color());
    println!("  resetfmt  = {}", cc::get_reset_fmt());
    println!("  lang      = {}", cc::get_lang());
    let segs: Vec<String> = SEGS.iter().map(|s| format!("{}={}", s, on(cc::get_show(s)))).collect();
    println!("  segments  = {}", segs.join(" "));
}

fn help() {
    println!(
        "ccgotchi — Claude Code statusline with usage bars + an animated pet\n\n\
         USAGE:\n  \
           ccgotchi setup                 install into ~/.claude/settings.json\n  \
           ccgotchi restore               undo setup\n  \
           ccgotchi config                show current settings\n  \
           ccgotchi pet <name>            {pets}\n  \
           ccgotchi shiny on|off          rainbow (shiny) pet\n  \
           ccgotchi barstyle <s>          dots|block|shade|square|slant|battery\n  \
           ccgotchi barcolor auto|mono\n  \
           ccgotchi resetfmt <f>          eta|arrow|paren|cn|off\n  \
           ccgotchi show <seg> on|off     model|5h|7d|ctx (hide/show a segment)\n  \
           ccgotchi petcolor <c>          auto|orange|pink|red|yellow|green|cyan|blue|purple|white|gray\n  \
           ccgotchi lang <l>              en|zh|ja|ko (auto-detected from $LANG)\n  \
           ccgotchi statusline            (called by Claude Code; reads JSON on stdin)\n\n\
         Tip: install the ccgotchi tray app for a click-to-configure menu.",
        pets = PETS.join("|")
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("statusline") => {
            let mut input = String::new();
            let _ = std::io::stdin().read_to_string(&mut input);
            let v: serde_json::Value = serde_json::from_str(&input).unwrap_or(serde_json::json!({}));
            let data = cc::parse(&v);
            println!("{}", cc::format_statusline(&data, cc::now_unix()));
        }
        Some("setup") => {
            let exe = std::env::current_exe()
                .ok()
                .and_then(|p| p.to_str().map(String::from))
                .unwrap_or_else(|| "ccgotchi".to_string());
            cc::install_statusline(&exe);
            println!("✅ Installed ccgotchi into {}", cc::claude_settings_path().display());
            println!("   Open a new Claude Code session (or wait a second) to see it.");
        }
        Some("restore") => {
            cc::restore_statusline();
            println!("✅ Restored the previous statusLine.");
        }
        Some("config") | Some("doctor") => print_config(),
        Some("pet") => match args.get(2).map(|s| s.as_str()) {
            Some(name) => {
                cc::set_pet(name);
                println!("pet = {}", name);
            }
            None => println!("pet = {} (options: {})", cc::get_pet(), PETS.join("|")),
        },
        Some("shiny") => match args.get(2).map(|s| s.as_str()) {
            Some(v @ ("on" | "off")) => {
                cc::set_pet_shiny(v == "on");
                println!("shiny = {}", v);
            }
            _ => println!(
                "shiny = {} (usage: shiny on|off)",
                if cc::get_pet_shiny() { "on" } else { "off" }
            ),
        },
        Some("barstyle") => match args.get(2).map(|s| s.as_str()) {
            Some(s) => {
                cc::set_bar_style(s);
                println!("barstyle = {}", s);
            }
            None => println!(
                "barstyle = {} (options: dots|block|shade|square|slant|battery)",
                cc::get_bar_style()
            ),
        },
        Some("barcolor") => match args.get(2).map(|s| s.as_str()) {
            Some(c) => {
                cc::set_bar_color(c);
                println!("barcolor = {}", c);
            }
            None => println!("barcolor = {} (options: auto|mono)", cc::get_bar_color()),
        },
        Some("resetfmt") => match args.get(2).map(|s| s.as_str()) {
            Some(f) => {
                cc::set_reset_fmt(f);
                println!("resetfmt = {}", f);
            }
            None => println!(
                "resetfmt = {} (options: eta|arrow|paren|cn|off)",
                cc::get_reset_fmt()
            ),
        },
        Some("show") => match (args.get(2).map(|s| s.as_str()), args.get(3).map(|s| s.as_str())) {
            (Some(seg), Some(v @ ("on" | "off"))) if SEGS.contains(&seg) => {
                cc::set_show(seg, v == "on");
                println!("show {} = {}", seg, v);
            }
            _ => println!("usage: show <{}> on|off", SEGS.join("|")),
        },
        Some("petcolor") => match args.get(2).map(|s| s.as_str()) {
            Some(c) => {
                cc::set_pet_color(c);
                println!("petcolor = {}", c);
            }
            None => println!(
                "petcolor = {} (auto|orange|pink|red|yellow|green|cyan|blue|purple|white|gray)",
                cc::get_pet_color()
            ),
        },
        Some("lang") => match args.get(2).map(|s| s.as_str()) {
            Some(l) => {
                cc::set_lang(l);
                println!("lang = {}", l);
            }
            None => println!(
                "lang = {} (built-in: en|zh|ja|ko; auto-detected from $LANG if unset)",
                cc::get_lang()
            ),
        },
        Some("-h") | Some("--help") | Some("help") | None => help(),
        Some(other) => {
            eprintln!("unknown command: {}\n", other);
            help();
        }
    }
}
