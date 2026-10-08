# Dense asteroid fit span measurement (issue #201)

Date: 2026-10-08

## Kernels

| File | SHA-256 |
| --- | --- |
| `de440.bsp` | `a4ce9bf9b3282becc9f4b2ac3cebe03a2ae7599981aabd7265fd8482fff7c4b5` |
| `sb441-n373s.bsp` | `2143113282bfc2b2a0b0b4626125d4f84362339b5a8ae7eea40f4120ca8da10b` |

## Command

```bash
PLEIADES_DE_KERNEL=/workspace/.kernels/de440.bsp \
PLEIADES_AST_KERNEL=/workspace/.kernels/sb441-n373s.bsp \
  cargo test --release -p pleiades-data --lib asteroid_fit -- --ignored --nocapture
```

Window 1900-2100 (`CoverageWindow::default()`), degree 8, 27 samples per
segment, error sampled every 0.5 d against the kernel. Longitude error is
scaled by cos(latitude). The sweep ran one process per span with a temporary
local span override (not committed); the final row set is the committed
configuration. Segments are tagged TT, so the artifact is queried in TT and
the kernel in TDB (a ~2 ms difference).

## Rule

Both maxima at most 1 arcsecond; the longest power-of-two span that meets it.

## Per-run table (max lon / max lat in arcseconds, JD of each)

Bytes are the added encoded size over a Sun-only artifact.

### Ceres

| Span (d) | Segments | Bytes | Max lon | Lon JD | Max lat | Lat JD | Meets |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 16 | 4566 | 1118676 | 0.0006 | 2434829 | 0.0002 | 2429692 | yes |
| 32 | 2283 | 559341 | 0.0007 | 2434829 | 0.0002 | 2429692 | yes |
| 64 | 1142 | 279796 | 0.0007 | 2434829 | 0.0002 | 2429692 | yes |
| 128 | 571 | 139901 | 0.0010 | 2421260 | 0.0003 | 2429692 | yes |
| 256 | 286 | 70076 | 0.0052 | 2456740.5 | 0.0013 | 2476828 | yes |
| 512 | 143 | 35041 | 0.4884 | 2423196.5 | 0.3005 | 2475421.5 | yes |
| 1024 | 72 | 17646 | 61.1183 | 2438543.5 | 46.1134 | 2475408 | no |

### Pallas

| Span (d) | Segments | Bytes | Max lon | Lon JD | Max lat | Lat JD | Meets |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 16 | 4566 | 1118676 | 0.0009 | 2426412 | 0.0005 | 2416108 | yes |
| 32 | 2283 | 559341 | 0.0009 | 2426412 | 0.0005 | 2416108 | yes |
| 64 | 1142 | 279796 | 0.0009 | 2426412 | 0.0005 | 2416108 | yes |
| 128 | 571 | 139901 | 0.0075 | 2438056.5 | 0.0037 | 2438056.5 | yes |
| 256 | 286 | 70076 | 2.5750 | 2443173 | 0.7244 | 2422708 | no |
| 512 | 143 | 35041 | 182.6909 | 2453405.5 | 97.6601 | 2448286 | no |
| 1024 | 72 | 17646 | 3521.3167 | 2437578 | 2374.0797 | 2419382.5 | no |

### Juno

| Span (d) | Segments | Bytes | Max lon | Lon JD | Max lat | Lat JD | Meets |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 16 | 4566 | 1118676 | 0.0009 | 2415308 | 0.0003 | 2475997 | yes |
| 32 | 2283 | 559341 | 0.0009 | 2415308 | 0.0003 | 2475997 | yes |
| 64 | 1142 | 279796 | 0.0009 | 2423341 | 0.0003 | 2475997 | yes |
| 128 | 571 | 139901 | 0.0021 | 2423343 | 0.0013 | 2463152.5 | yes |
| 256 | 286 | 70076 | 0.5131 | 2485428.5 | 0.3682 | 2415284 | yes |
| 512 | 143 | 35041 | 58.9668 | 2424764.5 | 41.1261 | 2437565 | no |
| 1024 | 72 | 17646 | 1645.3548 | 2440892.5 | 1384.1314 | 2458458 | no |

### Vesta

| Span (d) | Segments | Bytes | Max lon | Lon JD | Max lat | Lat JD | Meets |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 16 | 4566 | 1118676 | 0.0009 | 2437117 | 0.0002 | 2442685 | yes |
| 32 | 2283 | 559341 | 0.0009 | 2472909 | 0.0002 | 2442685 | yes |
| 64 | 1142 | 279796 | 0.0009 | 2437117 | 0.0002 | 2442685 | yes |
| 128 | 571 | 139901 | 0.0013 | 2480957 | 0.0003 | 2475421 | yes |
| 256 | 286 | 70076 | 0.0071 | 2439603.5 | 0.0056 | 2425012 | yes |
| 512 | 143 | 35041 | 1.1339 | 2476956.5 | 1.3870 | 2445213 | no |
| 1024 | 72 | 17646 | 96.7090 | 2468864.5 | 152.8933 | 2472396 | no |

### asteroid:433-Eros

| Span (d) | Segments | Bytes | Max lon | Lon JD | Max lat | Lat JD | Meets |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 16 | 4566 | 1118700 | 0.0048 | 2442461 | 0.0020 | 2428877 | yes |
| 32 | 2283 | 559365 | 0.0048 | 2442461 | 0.0019 | 2428877 | yes |
| 64 | 1142 | 279820 | 0.0053 | 2485533 | 0.0044 | 2428910.5 | yes |
| 128 | 571 | 139925 | 0.2812 | 2455976.5 | 1.1144 | 2426408.5 | no (lat) |
| 256 | 286 | 70100 | 60.6241 | 2455972.5 | 106.5386 | 2426354.5 | no |
| 512 | 143 | 35065 | 3124.9771 | 2485545 | 5724.9238 | 2442459 | no |
| 1024 | 72 | 17670 | 49572.8125 | 2458478 | 44806.1650 | 2455945 | no |

## Chosen spans (final run at these spans)

| Body | Span (d) | Segments | Bytes | Max lon (") | Lon JD | Max lat (") | Lat JD |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Ceres | 512 | 143 | 35041 | 0.4884 | 2423196.5 | 0.3005 | 2475421.5 |
| Pallas | 128 | 571 | 139901 | 0.0075 | 2438056.5 | 0.0037 | 2438056.5 |
| Juno | 256 | 286 | 70076 | 0.5131 | 2485428.5 | 0.3682 | 2415284 |
| Vesta | 256 | 286 | 70076 | 0.0071 | 2439603.5 | 0.0056 | 2425012 |
| asteroid:433-Eros | 64 | 1142 | 279820 | 0.0053 | 2485533 | 0.0044 | 2428910.5 |

## Projected artifact size

committed 10,491,298 - old Eros 665,083 + added 594,914 = **10,421,129 bytes**.
Headroom against 12,000,000: 1,578,871 bytes. The stop rule does not trigger.

Note: Ceres at 512 d and Juno at 256 d meet the rule with margin
(0.49" and 0.51") that is far larger than their shorter-span errors; the
rule picks the longest passing power of two as written.

## Machine load

`/proc/loadavg` at the final run: `132.92 99.50 80.26 119/4890 286980`
(shared machine; timing is not a claim here).
