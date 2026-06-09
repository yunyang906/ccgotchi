// ccgotchi — menu-bar tray app. Configure the Claude Code pet statusline by
// clicking; on launch it wires the statusline into ~/.claude/settings.json.
#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use ccgotchi as cc;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::TrayIconBuilder,
    Manager,
};

// (key, menu label). Keys match the core config values.
const PETS: [(&str, &str); 19] = [
    ("off", "off"),
    ("cat", "🐈 cat"),
    ("chonk", "🐈 chonk"),
    ("rabbit", "🐇 rabbit"),
    ("duck", "🦆 duck"),
    ("goose", "🦢 goose"),
    ("owl", "🦉 owl"),
    ("penguin", "🐧 penguin"),
    ("turtle", "🐢 turtle"),
    ("snail", "🐌 snail"),
    ("dragon", "🐉 dragon"),
    ("octopus", "🐙 octopus"),
    ("axolotl", "🦎 axolotl"),
    ("ghost", "👻 ghost"),
    ("robot", "🤖 robot"),
    ("blob", "🫧 blob"),
    ("cactus", "🌵 cactus"),
    ("mushroom", "🍄 mushroom"),
    ("capybara", "🦫 capybara"),
];
const STYLES: [(&str, &str); 6] = [
    ("dots", "dots ●○"),
    ("block", "block █░"),
    ("shade", "shade █▒"),
    ("square", "square ▮▯"),
    ("slant", "slant ▰▱"),
    ("battery", "battery ▕█░▏"),
];
const RESETS: [(&str, &str); 5] = [
    ("eta", "countdown 4h23m"),
    ("arrow", "arrow ↻4h23m"),
    ("paren", "paren (4h23m)"),
    ("cn", "中文 余4h23m"),
    ("off", "hidden"),
];
const METERS: [(&str, &str); 4] = [
    ("both", "tokens + cost $"),
    ("tokens", "tokens only"),
    ("cost", "cost $ only"),
    ("off", "hidden"),
];
const LANGS: [(&str, &str); 4] = [("en", "English"), ("zh", "中文"), ("ja", "日本語"), ("ko", "한국어")];

fn mark(cur: &str, key: &str) -> &'static str {
    if cur == key {
        "✓ "
    } else {
        "    "
    }
}
fn label_of(table: &[(&str, &str)], key: &str) -> String {
    table
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, l)| (*l).to_string())
        .unwrap_or_default()
}
fn color_label(mono: bool) -> &'static str {
    if mono {
        "bar color: mono · click for color"
    } else {
        "bar color: color · click for mono"
    }
}
fn shiny_label(on: bool) -> &'static str {
    if on {
        "✨ shiny: on · click to turn off"
    } else {
        "✨ shiny: off · click to turn on"
    }
}

/// Path to the bundled `ccgotchi` CLI (sibling of this binary), used as the
/// statusline command.
fn cli_path() -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    let name = if cfg!(windows) { "ccgotchi.exe" } else { "ccgotchi" };
    let cand = dir.join(name);
    if cand.exists() {
        cand.to_str().map(String::from)
    } else {
        None
    }
}

fn build_radio(
    app: &tauri::AppHandle,
    prefix: &str,
    table: &[(&str, &str)],
    current: &str,
) -> Vec<(String, MenuItem<tauri::Wry>)> {
    table
        .iter()
        .map(|(k, label)| {
            let it = MenuItem::with_id(
                app,
                format!("{prefix}{k}"),
                format!("{}{}", mark(current, k), label),
                true,
                None::<&str>,
            )
            .expect("menu item");
            (k.to_string(), it)
        })
        .collect()
}
fn refresh_radio(items: &[(String, MenuItem<tauri::Wry>)], table: &[(&str, &str)], current: &str) {
    for (key, it) in items {
        let _ = it.set_text(format!("{}{}", mark(current, key), label_of(table, key)));
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // wire the statusline into Claude Code (idempotent)
            if let Some(cli) = cli_path() {
                cc::install_statusline(&cli);
            }
            let ah = app.app_handle();

            let pet_items = build_radio(&ah, "pet_", &PETS, &cc::get_pet());
            let pet_menu = Submenu::with_items(
                app,
                "Pet",
                true,
                &pet_items
                    .iter()
                    .map(|(_, it)| it as &dyn tauri::menu::IsMenuItem<tauri::Wry>)
                    .collect::<Vec<_>>(),
            )?;

            let style_items = build_radio(&ah, "style_", &STYLES, &cc::get_bar_style());
            let style_menu = Submenu::with_items(
                app,
                "Bar style",
                true,
                &style_items
                    .iter()
                    .map(|(_, it)| it as &dyn tauri::menu::IsMenuItem<tauri::Wry>)
                    .collect::<Vec<_>>(),
            )?;

            let reset_items = build_radio(&ah, "reset_", &RESETS, &cc::get_reset_fmt());
            let reset_menu = Submenu::with_items(
                app,
                "Reset time",
                true,
                &reset_items
                    .iter()
                    .map(|(_, it)| it as &dyn tauri::menu::IsMenuItem<tauri::Wry>)
                    .collect::<Vec<_>>(),
            )?;

            let meter_items = build_radio(&ah, "meter_", &METERS, &cc::get_meter());
            let meter_menu = Submenu::with_items(
                app,
                "Usage meter",
                true,
                &meter_items
                    .iter()
                    .map(|(_, it)| it as &dyn tauri::menu::IsMenuItem<tauri::Wry>)
                    .collect::<Vec<_>>(),
            )?;

            let lang_items = build_radio(&ah, "lang_", &LANGS, &cc::get_lang());
            let lang_menu = Submenu::with_items(
                app,
                "Language",
                true,
                &lang_items
                    .iter()
                    .map(|(_, it)| it as &dyn tauri::menu::IsMenuItem<tauri::Wry>)
                    .collect::<Vec<_>>(),
            )?;

            let shiny_item = MenuItem::with_id(
                app,
                "shiny_toggle",
                shiny_label(cc::get_pet_shiny()),
                true,
                None::<&str>,
            )?;
            let color_item = MenuItem::with_id(
                app,
                "color_toggle",
                color_label(cc::get_bar_color() == "mono"),
                true,
                None::<&str>,
            )?;

            let reinstall = MenuItem::with_id(app, "reinstall", "Reinstall statusline", true, None::<&str>)?;
            let uninstall = MenuItem::with_id(app, "uninstall", "Restore (uninstall)", true, None::<&str>)?;
            let about = MenuItem::with_id(app, "about", "About ccgotchi…", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let sep1 = PredefinedMenuItem::separator(app)?;
            let sep2 = PredefinedMenuItem::separator(app)?;

            let menu = Menu::with_items(
                app,
                &[
                    &pet_menu,
                    &shiny_item,
                    &style_menu,
                    &color_item,
                    &reset_menu,
                    &meter_menu,
                    &lang_menu,
                    &sep1,
                    &reinstall,
                    &uninstall,
                    &about,
                    &sep2,
                    &quit,
                ],
            )?;

            // handles for the event closure
            let pet_evt = pet_items.clone();
            let style_evt = style_items.clone();
            let reset_evt = reset_items.clone();
            let meter_evt = meter_items.clone();
            let lang_evt = lang_items.clone();
            let shiny_evt = shiny_item.clone();
            let color_evt = color_item.clone();

            let icon = Image::from_bytes(include_bytes!("../icons/icon.png")).expect("icon");
            let _tray = TrayIconBuilder::with_id("main")
                .icon(icon)
                .tooltip("ccgotchi")
                .menu(&menu)
                .on_menu_event(move |app, event| {
                    let id = event.id.as_ref();
                    match id {
                        "quit" => app.exit(0),
                        "about" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                        "reinstall" => {
                            if let Some(cli) = cli_path() {
                                cc::install_statusline(&cli);
                            }
                        }
                        "uninstall" => cc::restore_statusline(),
                        "shiny_toggle" => {
                            let on = !cc::get_pet_shiny();
                            cc::set_pet_shiny(on);
                            let _ = shiny_evt.set_text(shiny_label(on));
                        }
                        "color_toggle" => {
                            let mono = cc::get_bar_color() != "mono";
                            cc::set_bar_color(if mono { "mono" } else { "auto" });
                            let _ = color_evt.set_text(color_label(mono));
                        }
                        other => {
                            if let Some(k) = other.strip_prefix("pet_") {
                                cc::set_pet(k);
                                refresh_radio(&pet_evt, &PETS, &cc::get_pet());
                            } else if let Some(k) = other.strip_prefix("style_") {
                                cc::set_bar_style(k);
                                refresh_radio(&style_evt, &STYLES, &cc::get_bar_style());
                            } else if let Some(k) = other.strip_prefix("reset_") {
                                cc::set_reset_fmt(k);
                                refresh_radio(&reset_evt, &RESETS, &cc::get_reset_fmt());
                            } else if let Some(k) = other.strip_prefix("meter_") {
                                cc::set_meter(k);
                                refresh_radio(&meter_evt, &METERS, &cc::get_meter());
                            } else if let Some(k) = other.strip_prefix("lang_") {
                                cc::set_lang(k);
                                refresh_radio(&lang_evt, &LANGS, &cc::get_lang());
                            }
                        }
                    }
                })
                .build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .build(tauri::generate_context!())
        .expect("failed to start ccgotchi")
        .run(|_app, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        });
}
