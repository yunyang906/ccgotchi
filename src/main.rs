//! ccgotchi CLI.
//!
//!   ccgotchi statusline        read Claude Code JSON on stdin, print the line
//!   ccgotchi setup             install into ~/.claude/settings.json
//!   ccgotchi restore           undo setup (restore previous statusLine)
//!   ccgotchi config            print current config
//!   ccgotchi pet <name>        cat | rabbit | duck | ... | off
//!   ccgotchi shiny on|off      rainbow (shiny) pet
//!   ccgotchi barstyle <s>      dots | block | shade | square | slant | battery
//!   ccgotchi barcolor <c>      auto | mono
//!   ccgotchi resetfmt <f>      eta | arrow | paren | cn | off
//!   ccgotchi meter <m>         both | tokens | cost | off

use ccgotchi as cc;
use std::io::Read;
use std::path::PathBuf;

const PETS: &[&str] = &[
    "off", "cat", "chonk", "rabbit", "duck", "goose", "owl", "penguin", "turtle", "snail", "dragon",
    "octopus", "axolotl", "ghost", "robot", "blob", "cactus", "mushroom", "capybara",
];

fn settings_path() -> PathBuf {
    cc::home_dir().join(".claude").join("settings.json")
}

fn read_json(path: &PathBuf) -> serde_json::Value {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::json!({}))
}

fn write_json(path: &PathBuf, v: &serde_json::Value) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, serde_json::to_string_pretty(v).unwrap_or_default());
}

fn setup() {
    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(String::from))
        .unwrap_or_else(|| "ccgotchi".to_string());
    let path = settings_path();
    let mut v = read_json(&path);
    if let Some(obj) = v.as_object_mut() {
        // Back up any existing statusLine once, so `restore` can undo us.
        if let Some(existing) = obj.get("statusLine") {
            let backup = cc::base_dir().join("statusLine.backup.json");
            if !backup.exists() {
                let _ = std::fs::create_dir_all(cc::base_dir());
                let _ = std::fs::write(&backup, existing.to_string());
            }
        }
        obj.insert(
            "statusLine".to_string(),
            // refreshInterval=1 so the pet keeps animating while idle
            serde_json::json!({
                "type": "command",
                "command": format!("\"{}\" statusline", exe),
                "refreshInterval": 1
            }),
        );
    }
    write_json(&path, &v);
    println!("✅ Installed ccgotchi into {}", path.display());
    println!("   Open a new Claude Code session (or wait a second) to see it.");
}

fn restore() {
    let path = settings_path();
    let backup = cc::base_dir().join("statusLine.backup.json");
    let mut v = read_json(&path);
    if let Some(obj) = v.as_object_mut() {
        match std::fs::read_to_string(&backup)
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
    write_json(&path, &v);
    let _ = std::fs::remove_file(&backup);
    println!("✅ Restored the previous statusLine.");
}

fn print_config() {
    println!("ccgotchi config ({}):", cc::base_dir().display());
    println!("  pet       = {}", cc::get_pet());
    println!("  shiny     = {}", if cc::get_pet_shiny() { "on" } else { "off" });
    println!("  barstyle  = {}", cc::get_bar_style());
    println!("  barcolor  = {}", cc::get_bar_color());
    println!("  resetfmt  = {}", cc::get_reset_fmt());
    println!("  meter     = {}", cc::get_meter());
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
           ccgotchi meter <m>             both|tokens|cost|off\n  \
           ccgotchi lang <l>              en|zh|ja|ko (auto-detected from $LANG)\n  \
           ccgotchi statusline            (called by Claude Code; reads JSON on stdin)",
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
        Some("setup") => setup(),
        Some("restore") => restore(),
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
        Some("meter") => match args.get(2).map(|s| s.as_str()) {
            Some(m) => {
                cc::set_meter(m);
                println!("meter = {}", m);
            }
            None => println!("meter = {} (options: both|tokens|cost|off)", cc::get_meter()),
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
