---
title: "Nightly CI failing"
labels: nightly-failure
---
The scheduled **nightly CI** run failed.

- Failing run: {{ env.RUN_URL }}
- Commit: {{ sha }}

This issue is auto-managed: it is updated on each consecutive nightly failure and
closed automatically the next time nightly CI passes. Do not close it by hand —
a green nightly will close it.
