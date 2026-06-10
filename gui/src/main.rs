// ccgotchi — menu-bar tray app. Click to configure the Claude Code pet
// statusline; on launch it wires the statusline into ~/.claude/settings.json.
// The whole menu is localized and rebuilt in the current language on any change.
#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use ccgotchi as cc;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::TrayIconBuilder,
    AppHandle, Manager, Wry,
};

// option keys (+ emoji / symbol where applicable)
const PETS: [(&str, &str); 19] = [
    ("off", ""), ("cat", "🐈"), ("chonk", "🐈"), ("rabbit", "🐇"), ("duck", "🦆"), ("goose", "🦢"),
    ("owl", "🦉"), ("penguin", "🐧"), ("turtle", "🐢"), ("snail", "🐌"), ("dragon", "🐉"),
    ("octopus", "🐙"), ("axolotl", "🦎"), ("ghost", "👻"), ("robot", "🤖"), ("blob", "🫧"),
    ("cactus", "🌵"), ("mushroom", "🍄"), ("capybara", "🦫"),
];
const STYLES: [(&str, &str); 6] = [
    ("dots", "●○"), ("block", "█░"), ("shade", "█▒"), ("square", "▮▯"), ("slant", "▰▱"), ("battery", "▕█░▏"),
];
const RESETS: [&str; 5] = ["eta", "arrow", "paren", "cn", "off"];
const SEGS: [&str; 4] = ["model", "5h", "7d", "ctx"];
const PET_COLORS: [&str; 11] = [
    "auto", "orange", "pink", "red", "yellow", "green", "cyan", "blue", "purple", "white", "gray",
];
const LANGS: [(&str, &str); 4] = [("en", "English"), ("zh", "中文"), ("ja", "日本語"), ("ko", "한국어")];

/// Localized UI string. en is the fallback for any language not translated.
fn t(lang: &str, key: &str) -> &'static str {
    match key {
        "pet" => match lang { "zh" => "宠物", "ja" => "ペット", "ko" => "펫", _ => "Pet" },
        "style" => match lang { "zh" => "进度条样式", "ja" => "バー", "ko" => "막대 스타일", _ => "Bar style" },
        "reset" => match lang { "zh" => "重置时间", "ja" => "リセット", "ko" => "리셋 시간", _ => "Reset time" },
        "segments" => match lang { "zh" => "显示项", "ja" => "表示項目", "ko" => "표시 항목", _ => "Segments" },
        "petcolor" => match lang { "zh" => "宠物颜色", "ja" => "ペットの色", "ko" => "펫 색상", _ => "Pet color" },
        "lang" => match lang { "zh" => "语言", "ja" => "言語", "ko" => "언어", _ => "Language" },
        "shiny_on" => match lang { "zh" => "✨ 七彩:开 · 点击关闭", "ja" => "✨ シャイニー:オン", "ko" => "✨ 샤이니: 켜짐", _ => "✨ shiny: on · click to turn off" },
        "shiny_off" => match lang { "zh" => "✨ 七彩:关 · 点击开启", "ja" => "✨ シャイニー:オフ", "ko" => "✨ 샤이니: 꺼짐", _ => "✨ shiny: off · click to turn on" },
        "color_color" => match lang { "zh" => "配色:彩色 · 点击切单色", "ja" => "色:カラー", "ko" => "색: 컬러", _ => "bar color: color · click for mono" },
        "color_mono" => match lang { "zh" => "配色:单色 · 点击切彩色", "ja" => "色:モノ", "ko" => "색: 모노", _ => "bar color: mono · click for color" },
        "reinstall" => match lang { "zh" => "重装状态栏", "ja" => "再インストール", "ko" => "재설치", _ => "Reinstall statusline" },
        "restore" => match lang { "zh" => "卸载还原", "ja" => "アンインストール", "ko" => "복원(제거)", _ => "Restore (uninstall)" },
        "about" => match lang { "zh" => "关于 ccgotchi…", "ja" => "ccgotchi について…", "ko" => "ccgotchi 정보…", _ => "About ccgotchi…" },
        "quit" => match lang { "zh" => "退出", "ja" => "終了", "ko" => "종료", _ => "Quit" },
        _ => "",
    }
}

/// Pet species name (en/zh; others fall back to en).
fn pet_name(lang: &str, key: &str) -> String {
    if lang != "zh" {
        return key.to_string(); // English key (looks fine: cat, capybara, …)
    }
    match key {
        "off" => "关闭", "cat" => "猫", "chonk" => "胖猫", "rabbit" => "兔子", "duck" => "鸭子",
        "goose" => "鹅", "owl" => "猫头鹰", "penguin" => "企鹅", "turtle" => "乌龟", "snail" => "蜗牛",
        "dragon" => "龙", "octopus" => "章鱼", "axolotl" => "六角恐龙", "ghost" => "幽灵",
        "robot" => "机器人", "blob" => "史莱姆", "cactus" => "仙人掌", "mushroom" => "蘑菇",
        "capybara" => "水豚", _ => key,
    }
    .to_string()
}
fn style_name(lang: &str, key: &str) -> String {
    if lang != "zh" {
        return key.to_string();
    }
    match key {
        "dots" => "圆点", "block" => "实心", "shade" => "中阴影", "square" => "方块", "slant" => "斜纹",
        "battery" => "电池", _ => key,
    }
    .to_string()
}
fn reset_label(lang: &str, key: &str) -> &'static str {
    match (key, lang) {
        ("eta", "zh") => "倒计时 4h23m", ("eta", _) => "countdown 4h23m",
        ("arrow", "zh") => "箭头 ↻4h23m", ("arrow", _) => "arrow ↻4h23m",
        ("paren", "zh") => "括号 (4h23m)", ("paren", _) => "paren (4h23m)",
        ("cn", "zh") => "中文 余4h23m", ("cn", _) => "中文 余4h23m",
        ("off", "zh") => "隐藏", ("off", _) => "hidden",
        _ => "",
    }
}
fn seg_label(lang: &str, key: &str) -> &'static str {
    match (key, lang) {
        ("model", "zh") => "模型", ("model", "ja") => "モデル", ("model", "ko") => "모델", ("model", _) => "Model",
        ("5h", _) => "5h",
        ("7d", "zh") => "7 天", ("7d", _) => "7d",
        ("ctx", "zh") => "上下文", ("ctx", _) => "ctx",
        _ => "",
    }
}
fn color_name(lang: &str, key: &str) -> &'static str {
    match (key, lang) {
        ("auto", "zh") => "自动(按健康)", ("auto", _) => "auto (by health)",
        ("orange", "zh") => "橙", ("orange", _) => "orange",
        ("pink", "zh") => "粉", ("pink", _) => "pink",
        ("red", "zh") => "红", ("red", _) => "red",
        ("yellow", "zh") => "黄", ("yellow", _) => "yellow",
        ("green", "zh") => "绿", ("green", _) => "green",
        ("cyan", "zh") => "青", ("cyan", _) => "cyan",
        ("blue", "zh") => "蓝", ("blue", _) => "blue",
        ("purple", "zh") => "紫", ("purple", _) => "purple",
        ("white", "zh") => "白", ("white", _) => "white",
        ("gray", "zh") => "灰", ("gray", _) => "gray",
        _ => "",
    }
}

fn mark(cur: &str, key: &str) -> &'static str {
    if cur == key {
        "✓ "
    } else {
        "    "
    }
}

fn radio_submenu(
    app: &AppHandle,
    title: &str,
    prefix: &str,
    cur: &str,
    items: &[(String, String)], // (key, label)
) -> tauri::Result<Submenu<Wry>> {
    let mis: Vec<MenuItem<Wry>> = items
        .iter()
        .map(|(k, label)| {
            MenuItem::with_id(
                app,
                format!("{prefix}{k}"),
                format!("{}{}", mark(cur, k), label),
                true,
                None::<&str>,
            )
            .expect("menu item")
        })
        .collect();
    let refs: Vec<&dyn tauri::menu::IsMenuItem<Wry>> =
        mis.iter().map(|m| m as &dyn tauri::menu::IsMenuItem<Wry>).collect();
    Submenu::with_items(app, title, true, &refs)
}

/// Build the whole tray menu in `lang`, with ✓ marks reflecting current config.
fn build_menu(app: &AppHandle, lang: &str) -> tauri::Result<Menu<Wry>> {
    let pet = radio_submenu(
        app, t(lang, "pet"), "pet_", &cc::get_pet(),
        &PETS.iter().map(|(k, e)| {
            let name = pet_name(lang, k);
            (k.to_string(), if e.is_empty() { name.to_string() } else { format!("{} {}", e, name) })
        }).collect::<Vec<_>>(),
    )?;
    let style = radio_submenu(
        app, t(lang, "style"), "style_", &cc::get_bar_style(),
        &STYLES.iter().map(|(k, sym)| (k.to_string(), format!("{} {}", style_name(lang, k), sym))).collect::<Vec<_>>(),
    )?;
    let reset = radio_submenu(
        app, t(lang, "reset"), "reset_", &cc::get_reset_fmt(),
        &RESETS.iter().map(|k| (k.to_string(), reset_label(lang, k).to_string())).collect::<Vec<_>>(),
    )?;
    // Segments: per-item show/hide toggles (✓ = shown)
    let seg_items: Vec<MenuItem<Wry>> = SEGS
        .iter()
        .map(|s| {
            let m = if cc::get_show(s) { "✓ " } else { "    " };
            MenuItem::with_id(app, format!("seg_{s}"), format!("{}{}", m, seg_label(lang, s)), true, None::<&str>)
                .expect("menu item")
        })
        .collect();
    let seg_refs: Vec<&dyn tauri::menu::IsMenuItem<Wry>> =
        seg_items.iter().map(|m| m as &dyn tauri::menu::IsMenuItem<Wry>).collect();
    let segments = Submenu::with_items(app, t(lang, "segments"), true, &seg_refs)?;

    let petcolor = radio_submenu(
        app, t(lang, "petcolor"), "petcolor_", &cc::get_pet_color(),
        &PET_COLORS.iter().map(|c| (c.to_string(), color_name(lang, c).to_string())).collect::<Vec<_>>(),
    )?;
    let language = radio_submenu(
        app, t(lang, "lang"), "lang_", &cc::get_lang(),
        &LANGS.iter().map(|(k, native)| (k.to_string(), native.to_string())).collect::<Vec<_>>(),
    )?;

    let shiny = MenuItem::with_id(
        app, "shiny_toggle",
        t(lang, if cc::get_pet_shiny() { "shiny_on" } else { "shiny_off" }),
        true, None::<&str>,
    )?;
    let color = MenuItem::with_id(
        app, "color_toggle",
        t(lang, if cc::get_bar_color() == "mono" { "color_mono" } else { "color_color" }),
        true, None::<&str>,
    )?;
    let reinstall = MenuItem::with_id(app, "reinstall", t(lang, "reinstall"), true, None::<&str>)?;
    let restore = MenuItem::with_id(app, "uninstall", t(lang, "restore"), true, None::<&str>)?;
    let about = MenuItem::with_id(app, "about", t(lang, "about"), true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", t(lang, "quit"), true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;

    Menu::with_items(
        app,
        &[
            &pet, &petcolor, &shiny, &style, &color, &segments, &reset, &language,
            &sep1, &reinstall, &restore, &about, &sep2, &quit,
        ],
    )
}

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

/// Rebuild the tray menu in the current language (called after any change).
fn refresh_menu(app: &AppHandle) {
    if let Some(tray) = app.tray_by_id("main") {
        if let Ok(menu) = build_menu(app, &cc::get_lang()) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            if let Some(cli) = cli_path() {
                cc::install_statusline(&cli);
            }
            let ah = app.app_handle();
            let menu = build_menu(&ah, &cc::get_lang())?;
            // Tray icon: on macOS a white silhouette used as a *template* image
            // (auto-adapts to light/dark menu bars). Windows/Linux trays don't
            // support template mode, so use the full-color icon there.
            let mut tray = TrayIconBuilder::with_id("main")
                .tooltip("ccgotchi")
                .menu(&menu);
            #[cfg(target_os = "macos")]
            {
                let icon = Image::from_bytes(include_bytes!("../icons/tray.png")).expect("icon");
                tray = tray.icon(icon).icon_as_template(true);
            }
            #[cfg(not(target_os = "macos"))]
            {
                let icon = Image::from_bytes(include_bytes!("../icons/icon.png")).expect("icon");
                tray = tray.icon(icon);
            }
            let _tray = tray
                .on_menu_event(move |app, event| {
                    let id = event.id.as_ref();
                    match id {
                        "quit" => {
                            app.exit(0);
                            return;
                        }
                        "about" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                            return;
                        }
                        "reinstall" => {
                            if let Some(cli) = cli_path() {
                                cc::install_statusline(&cli);
                            }
                        }
                        "uninstall" => cc::restore_statusline(),
                        "shiny_toggle" => cc::set_pet_shiny(!cc::get_pet_shiny()),
                        "color_toggle" => {
                            let mono = cc::get_bar_color() != "mono";
                            cc::set_bar_color(if mono { "mono" } else { "auto" });
                        }
                        other => {
                            if let Some(k) = other.strip_prefix("petcolor_") {
                                cc::set_pet_color(k);
                            } else if let Some(k) = other.strip_prefix("pet_") {
                                cc::set_pet(k);
                            } else if let Some(k) = other.strip_prefix("style_") {
                                cc::set_bar_style(k);
                            } else if let Some(k) = other.strip_prefix("reset_") {
                                cc::set_reset_fmt(k);
                            } else if let Some(seg) = other.strip_prefix("seg_") {
                                cc::set_show(seg, !cc::get_show(seg)); // toggle
                            } else if let Some(k) = other.strip_prefix("lang_") {
                                cc::set_lang(k);
                            } else {
                                return;
                            }
                        }
                    }
                    // reflect the change (labels + ✓ marks) in the current language
                    refresh_menu(app);
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
