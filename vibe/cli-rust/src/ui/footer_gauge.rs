//! Footer gauges: the context window and the monthly Vibe Code usage, as bars on a track.

use std::time::Duration;

use crate::clock::{self, UiInstant};
use crate::utils::text::format_compact_count;

use super::footer_fit::{Form, Tone};

/// Cells of a wide gauge; each holds eight steps.
pub const BAR_CELLS: usize = 6;
const EIGHTHS: [&str; 8] = ["▏", "▎", "▍", "▌", "▋", "▊", "▉", "█"];
const LEVELS: [&str; 8] = ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
/// Without pay-as-you-go the usage fill warns, then alarms, as the allowance runs out.
const WARN_AT: f64 = 75.0;
const ALARM_AT: f64 = 90.0;

/// The monthly Vibe Code allowance from `account/read`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Usage {
    pub percent: f64,
    /// Usage past 100% is billed instead of paused.
    pub payg: bool,
}

/// A fill on a track `cells` wide, eighth-cell precise; any use lights a step and only 100% fills it.
pub fn bar(percent: f64, cells: usize) -> (String, Tone) {
    (
        fill(percent, cells * 8, |step| bar_cells(step, cells)),
        Tone::Gauge,
    )
}

/// One cell rising from the bottom; empty only at 0%.
pub fn level(percent: f64) -> (String, Tone) {
    let text = fill(percent, 8, |step| match step {
        0 => " ".to_owned(),
        step => LEVELS[step - 1].to_owned(),
    });
    (text, Tone::Gauge)
}

/// `render` the step out of `steps`, rounded but never full before 100% nor empty above 0%.
fn fill(percent: f64, steps: usize, render: impl Fn(usize) -> String) -> String {
    let percent = percent.clamp(0.0, 100.0);
    let step = (percent / 100.0 * steps as f64).round() as usize;
    let step = match percent {
        _ if steps == 0 => 0,
        p if p >= 100.0 => steps,
        p if p > 0.0 => step.clamp(1, steps.saturating_sub(1).max(1)),
        _ => 0,
    };
    render(step)
}

fn bar_cells(steps: usize, cells: usize) -> String {
    let (full, part) = (steps / 8, steps % 8);
    let mut text = EIGHTHS[7].repeat(full);
    if part > 0 {
        text.push_str(EIGHTHS[part - 1]);
    }
    let pad = cells.saturating_sub(text.chars().count());
    text.push_str(&" ".repeat(pad));
    text
}

fn percent_label(percent: f64) -> String {
    format!("{}%", percent.max(0.0).round() as u64)
}

/// Rounded down, so `100%` always means the allowance is spent.
fn spent_label(percent: f64) -> String {
    format!("{}%", percent.max(0.0).floor() as u64)
}

/// `ctx ▍      23k/600k`, `ctx ▁ 4%`, then `ctx ▁`; absent while `max == 0`.
pub fn context_forms((current, max): (u64, u64)) -> Vec<Form> {
    if max == 0 {
        return vec![];
    }
    let percent = (current as f64 / max as f64 * 100.0).min(100.0);
    let counts = format!(
        "{}/{}",
        format_compact_count(current),
        format_compact_count(max)
    );
    let label = || ("ctx ".to_owned(), Tone::Muted);
    let tail = |text: String| (format!(" {text}"), Tone::Muted);
    vec![
        vec![label(), bar(percent, BAR_CELLS), tail(counts)],
        vec![label(), level(percent), tail(percent_label(percent))],
        vec![label(), level(percent)],
    ]
}

/// A warm provider prompt cache: the tokens the last turn was served from, and when that turn finished.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cache {
    pub cached_tokens: u64,
    pub refreshed_at: UiInstant,
}

/// The provider refreshes a warm prefix's expiry on every hit; the footer's estimate follows this window.
pub const CACHE_TTL: Duration = Duration::from_secs(60 * 60);

/// `cache ▄ 182k·60m`, `cache ▄ 60m`, then `cache ▄`; absent while nothing is cached.
pub fn cache_forms(cache: Option<Cache>) -> Vec<Form> {
    let Some(Cache {
        cached_tokens,
        refreshed_at,
    }) = cache
    else {
        return vec![];
    };
    cache_segment(
        cached_tokens,
        CACHE_TTL.saturating_sub(clock::since(refreshed_at)),
    )
}

/// The cache gauge for `cached_tokens` with `remaining` of its estimated life left.
pub fn cache_segment(cached_tokens: u64, remaining: Duration) -> Vec<Form> {
    let percent = remaining.as_secs_f64() / CACHE_TTL.as_secs_f64() * 100.0;
    let minutes = remaining.as_secs() / 60;
    let label = || ("cache ".to_owned(), Tone::Muted);
    let tail = |text: String| (format!(" {text}"), Tone::Muted);
    vec![
        vec![
            label(),
            level(percent),
            tail(format!(
                "{}\u{b7}{}m",
                format_compact_count(cached_tokens),
                minutes
            )),
        ],
        vec![label(), level(percent), tail(format!("{minutes}m"))],
        vec![label(), level(percent)],
    ]
}

/// `usage ▮▮▮    40%`, `usage ▄ 40%`, then `usage ▄`; `PAYG` follows when usage past 100% is billed.
pub fn usage_forms(usage: Option<Usage>) -> Vec<Form> {
    let Some(Usage { percent, payg }) = usage else {
        return vec![];
    };
    let tone = match percent {
        _ if payg => Tone::Gauge,
        p if p >= ALARM_AT => Tone::GaugeAlarm,
        p if p >= WARN_AT => Tone::GaugeWarn,
        _ => Tone::Gauge,
    };
    let label = || ("usage ".to_owned(), Tone::Muted);
    let tail = |pct: bool| {
        let parts = [
            pct.then(|| spent_label(percent)),
            payg.then(|| "PAYG".to_owned()),
        ];
        let text = parts.into_iter().flatten().collect::<Vec<_>>().join(" · ");
        (!text.is_empty()).then(|| (format!(" {text}"), Tone::Muted))
    };
    [
        (bar(percent, BAR_CELLS), true),
        (level(percent), true),
        (level(percent), false),
    ]
    .into_iter()
    .map(|((gauge, _), pct)| {
        [Some(label()), Some((gauge, tone)), tail(pct)]
            .into_iter()
            .flatten()
            .collect()
    })
    .collect()
}
