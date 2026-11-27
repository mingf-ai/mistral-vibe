//! PageUp/PageDown page the transcript; Shift+PageUp/PageDown page the input caret.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use vibe_rs::app::App;

use crate::support::{live, message, redraw, send_key};

/// An app with a 40-message transcript rendered at least once, so the page
/// keys know the viewport they page by.
fn paged_app() -> App {
    let mut app = App::default();
    app.view.transcript = live(
        &(0..40)
            .map(|index| {
                let role = if index % 2 == 0 { "user" } else { "assistant" };
                message(&format!("m{index}"), role, &format!("line {index}"))
            })
            .collect::<Vec<_>>(),
    );
    redraw(&mut app);
    app
}

fn page_key(code: KeyCode, shift: bool) -> KeyEvent {
    KeyEvent::new(
        code,
        if shift {
            KeyModifiers::SHIFT
        } else {
            KeyModifiers::NONE
        },
    )
}

#[test]
fn page_up_moves_the_viewport_one_page() {
    let mut app = paged_app();
    let page = app.view.transcript_height;
    assert!(page > 1, "a rendered transcript has a viewport");
    send_key(&mut app, page_key(KeyCode::PageUp, false));
    assert_eq!(app.view.scroll_target, page);
    send_key(&mut app, page_key(KeyCode::PageUp, false));
    assert_eq!(app.view.scroll_target, page * 2);
}

#[test]
fn page_down_returns_to_the_newest_content() {
    let mut app = paged_app();
    send_key(&mut app, page_key(KeyCode::PageUp, false));
    send_key(&mut app, page_key(KeyCode::PageDown, false));
    assert_eq!(app.view.scroll_target, 0);
    send_key(&mut app, page_key(KeyCode::PageDown, false));
    assert_eq!(app.view.scroll_target, 0);
}

#[test]
fn repeated_page_ups_clamp_at_the_document_top() {
    let mut app = paged_app();
    for _ in 0..20 {
        send_key(&mut app, page_key(KeyCode::PageUp, false));
    }
    redraw(&mut app);
    assert_eq!(
        app.view.scroll_target,
        app.view.last_total - app.view.transcript_height
    );
}

#[test]
fn page_keys_do_not_scroll_a_short_transcript() {
    let mut app = App::default();
    app.view.transcript = live(&[message("m0", "user", "hi")]);
    redraw(&mut app);
    send_key(&mut app, page_key(KeyCode::PageUp, false));
    redraw(&mut app);
    assert_eq!(app.view.scroll_target, 0);
}

#[test]
fn shift_page_keys_move_the_input_caret_not_the_transcript() {
    let mut app = App::default();
    app.chat_input.input = "line\n".repeat(40);
    app.chat_input.cursor = app.chat_input.input.len();
    redraw(&mut app);
    let end = app.chat_input.cursor;
    send_key(&mut app, page_key(KeyCode::PageUp, true));
    assert!(
        app.chat_input.cursor < end,
        "the caret paged up in the input"
    );
    assert_eq!(app.view.scroll_target, 0);
    send_key(&mut app, page_key(KeyCode::PageDown, true));
    assert!(app.chat_input.cursor > 0);
    assert_eq!(app.view.scroll_target, 0);
}

#[test]
fn bare_page_keys_do_not_move_the_input_caret() {
    let mut app = App::default();
    app.chat_input.input = "line\n".repeat(40);
    app.chat_input.cursor = app.chat_input.input.len();
    redraw(&mut app);
    let end = app.chat_input.cursor;
    send_key(&mut app, page_key(KeyCode::PageUp, false));
    assert_eq!(app.chat_input.cursor, end);
    send_key(&mut app, page_key(KeyCode::PageDown, false));
    assert_eq!(app.chat_input.cursor, end);
}
