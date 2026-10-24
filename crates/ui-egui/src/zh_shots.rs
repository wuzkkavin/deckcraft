//! Temporary zh-Hant review screenshots (owner's verification gate, 2026-10-24).
//! `#[ignore]`d; run with `cargo test -p deckcraft-ui-egui zh_shots -- --ignored --test-threads=1`.
//! Deleted after the v0.10.0-zh.1 release screenshots are captured and reviewed.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::{Services, SlideApp, dialogs};
use deckcraft_engine::Session;
use egui_kittest::Harness;

const OUT: &str = "/tmp/zhshots-deckcraft";

fn shoot(name: &str, w: f32, h: f32, setup: impl FnOnce(&mut SlideApp)) {
    let _guard = crate::i18n::TEST_LANG_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut app = SlideApp::new(Session::with_new(), Services::default());
    app.ui.language = "zh-Hant".to_string();
    setup(&mut app);
    let mut hh = Harness::builder().with_size(egui::vec2(w, h)).build_ui_state(
        |ui, app: &mut SlideApp| {
            app.logic(ui.ctx());
            app.ui(ui);
        },
        app,
    );
    // The app keeps asking for frames, so step instead of run().
    hh.run_steps(6);
    let img = hh.render().unwrap();
    std::fs::create_dir_all(OUT).unwrap();
    let path = format!("{OUT}/{name}");
    img.save(&path).unwrap();
    crate::i18n::set_lang(crate::i18n::Lang::En);
}

#[test]
#[ignore = "writes zh-Hant review screenshots"]
fn main_window() {
    shoot("main-zh-v0.10.0.png", 1600.0, 1000.0, |_| {});
}

#[test]
#[ignore = "writes zh-Hant review screenshots"]
fn preferences_dialog() {
    shoot("preferences-zh-v0.10.0.png", 1600.0, 1000.0, |app| {
        app.dialog = Some(dialogs::Dialog::new("preferences"));
    });
}

#[test]
#[ignore = "writes zh-Hant review screenshots"]
fn about_dialog() {
    shoot("about-zh-v0.10.0.png", 1600.0, 1000.0, |app| {
        app.dialog = Some(dialogs::Dialog::new("about"));
    });
}

#[test]
#[ignore = "writes zh-Hant review screenshots"]
fn account_page() {
    shoot("backstage-account-zh-v0.10.0.png", 1600.0, 1000.0, |app| {
        app.open_backstage(backstage::Page::Account);
    });
}
