---
title: "Stations gate failing"
labels: stations-gate-failure
---
The scheduled **full planetary-stations gate** (`mise run gate-stations`; planets, true node, and Ceres–Vesta) failed.

- Failing run: {{ env.RUN_URL }}
- Commit: {{ sha }}

This issue is auto-managed: it is updated on each consecutive failure and
closed automatically the next time the stations gate passes. Do not close it by
hand — a green run will close it.
