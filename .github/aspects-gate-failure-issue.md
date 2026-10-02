---
title: "Aspects gate failing"
labels: aspects-gate-failure
---
The scheduled **full exact-aspect gate** (`mise run gate-aspects`) failed.

- Failing run: {{ env.RUN_URL }}
- Commit: {{ sha }}

This issue is auto-managed: it is updated on each consecutive failure and
closed automatically the next time the aspects gate passes. Do not close it by
hand — a green run will close it.
