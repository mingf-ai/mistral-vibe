//! The footer's prompt-cache gauge: the expiry estimate, the counts it shows, and its place between the other gauges.

use std::time::Duration;

use serde_json::{json, Value};
use vibe_rs::app::App;
use vibe_rs::clock;
use vibe_rs::post_ready::AccountReads;
use vibe_rs::ui::footer_fit::Tone;
use vibe_rs::ui::footer_gauge::{cache_forms, cache_segment, Cache};

use crate::support::bottom_bar_row;

fn app(cache: Option<Cache>) -> App {
    let mut app = App::default();
    app.session.cwd = Some("/tmp/project".into());
    app.session.tokens = (23_000, 600_000);
    app.session.startup_config.active_model = "mistral-medium-3.5".into();
    app.session.cache = cache;
    let account = json!({ "status": "ready", "budget": { "usagePercentage": 40.0 } });
    app.whoami.store(
        app.session.startup_config.active_model.clone(),
        AccountReads {
            identity: Value::Null,
            account,
        },
    );
    app
}

fn text(form: &[(String, Tone)]) -> String {
    form.iter().map(|(text, _)| text.as_str()).collect()
}

#[test]
fn a_warm_cache_shows_its_tokens_and_one_hour_of_life() {
    let cache = Cache {
        cached_tokens: 182_340,
        refreshed_at: clock::now(),
    };
    let fresh = &cache_forms(Some(cache)).remove(0);
    let minutes = [" 182k·60m", " 182k·59m"];
    assert!(
        minutes.contains(&fresh[2].0.as_str()),
        "near the minute boundary: {fresh:?}"
    );
    assert!(text(fresh).starts_with("cache "), "{fresh:?}");
    assert_eq!(fresh[1].1, Tone::Gauge, "the fill is a gauge");
}

#[test]
fn a_cold_cache_hides_the_gauge() {
    assert!(cache_forms(None).is_empty());
    let row = bottom_bar_row(&mut app(None), 120);
    assert!(!row.contains("cache"), "{row:?}");
}

#[test]
fn the_estimate_falls_with_the_minutes_left() {
    let half = cache_segment(1_000, Duration::from_secs(30 * 60)).remove(0);
    assert_eq!(text(&half), "cache ▄ 1k·30m", "{half:?}");
    let late = cache_segment(1_000, Duration::from_secs(4 * 60)).remove(0);
    assert_eq!(text(&late), "cache ▁ 1k·4m", "{late:?}");
    let gone = cache_segment(1_000, Duration::ZERO).remove(0);
    assert_eq!(text(&gone), "cache   1k·0m", "{gone:?}");
}

#[test]
fn the_gauge_narrows_by_losing_one_tail_field_each_step() {
    let forms = cache_segment(1_000, Duration::from_secs(50 * 60));
    let widths: Vec<String> = forms.iter().map(|form| text(form)).collect();
    assert_eq!(
        widths,
        ["cache ▇ 1k·50m", "cache ▇ 50m", "cache ▇"],
        "each step loses one tail field"
    );
}

#[test]
fn the_cache_sits_between_the_usage_and_the_context() {
    let mut app = app(Some(Cache {
        cached_tokens: 182_340,
        refreshed_at: clock::now(),
    }));

    let row = bottom_bar_row(&mut app, 120);

    let user_gauge = row.find("usage").expect("usage");
    let cache_gauge = row.find("cache").expect("cache");
    let ctx = row.find("ctx").expect("ctx");
    assert!(user_gauge < cache_gauge && cache_gauge < ctx, "{row:?}");
    let cache_read = &row[cache_gauge..];
    assert!(cache_read.starts_with("cache "), "{row:?}");
    assert!(
        cache_read.contains("182k·"),
        "near the minute boundary: {row:?}"
    );
    assert!(row.contains("40%"), "{row:?}");
}
