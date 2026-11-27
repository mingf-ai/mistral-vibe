from __future__ import annotations

from e2e.app_server.scenario import Timeline, take_snapshot

_SHIFT_PAGE_UP = "\x1b[5;2~"
_SHIFT_PAGE_DOWN = "\x1b[6;2~"

timeline: Timeline = [
    "\n".join(f"line {index}" for index in range(24)),
    _SHIFT_PAGE_UP,
    "X",
    _SHIFT_PAGE_DOWN,
    "Y",
    take_snapshot(),
]
