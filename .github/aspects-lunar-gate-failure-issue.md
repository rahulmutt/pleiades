---
title: "Lunar-point aspects gate failing"
labels: aspects-lunar-gate-failure
---
The scheduled **lunar-point exact-aspect gate** (`mise run gate-aspects-lunar`; the 22 lunar-point pairs: true node and mean node with seven planets, true apogee and mean apogee with four) failed.

- Failing run: {{ env.RUN_URL }}
- Commit: {{ sha }}

This issue is auto-managed: it is updated on each consecutive failure and
closed automatically the next time the lunar-point aspects gate passes. Do not
close it by hand — a green run will close it.
