"""A warm prompt cache paints its token count and one-hour expiry estimate left of the context gauge."""

from __future__ import annotations

from e2e.app_server.context import stats_updated
from e2e.app_server.events import assistant_msg, turn_completed, turn_started, user_msg
from e2e.app_server.scenario import Timeline, advance, take_snapshot

timeline: Timeline = [
    "hello\r",
    turn_started(),
    user_msg("hello"),
    assistant_msg("Hi there."),
    stats_updated(last_turn_cached=182_340),
    turn_completed(),
    take_snapshot(),
    advance(1000),
    take_snapshot(),
]
screen_contains = ("cache ▇ 182k·59m", "ctx █▎     42.3k/200k")
