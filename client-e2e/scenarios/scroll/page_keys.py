"""Bare PageUp/PageDown page the transcript; one page per press, PageDown back."""

from __future__ import annotations

from e2e.app_server.events import assistant_msg, turn_completed, turn_started, user_msg
from e2e.app_server.scenario import Timeline, take_snapshot

_PAGE_UP = "\x1b[5~"
_PAGE_DOWN = "\x1b[6~"
_REPLY = "\n\n".join(f"para {index:02d}" for index in range(1, 61))

timeline: Timeline = [
    "hi\r",
    turn_started(),
    user_msg("hi"),
    assistant_msg(_REPLY),
    turn_completed(),
    take_snapshot(),
    _PAGE_UP,
    take_snapshot(),
    _PAGE_UP,
    take_snapshot(),
    _PAGE_DOWN,
    take_snapshot(),
]

# Two pages up and one back down land one page above the newest content.
screen_contains = ("para 30", "para 44")
screen_excludes = ("para 29", "para 45", "para 60")
