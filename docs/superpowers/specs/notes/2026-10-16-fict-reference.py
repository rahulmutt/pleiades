#!/usr/bin/env python3
"""Independent reference for the pleiades-fict mutant triage (issue #162).

Regenerates every expected-position literal in the `pleiades-fict` reference
tests and every per-mutant displacement row in
`docs/superpowers/specs/2026-10-16-fu9-fict-mutant-triage-design.md`. It shares
no code with the crate: textbook formulas only (Kepler's equation by bisection,
explicit Rz Rx Rz rotation matrices, Meeus *Astronomical Algorithms* ch. 21
ecliptic precession). Pure standard library.

Usage:  python3 -I 2026-10-16-fict-reference.py <command>

  values         expected positions, speeds and hand values used by the tests
  state-disp     per-mutant displacement rows for KeplerElements::state_at
  other-disp     rows for earth/geocentric, motion, spherical/cartesian,
                 orbital_plane_position
  frame-disp     rows for frame.rs and the solve_kepler `<`->`>` residuals
  kepler-equiv   measured bound for the four equivalent solve_kepler mutants

The mutant-swap harness re-implements the arithmetic of one mutated site with
the operator swapped (`line:col` as cargo-mutants reports it) and prints
|mutant - reference| for the inputs the tests use.
"""
import itertools
import math
import sys

J2000 = 2451545.0
GAUSS_DEG = 0.9856076686
SUN_EARTH = 332946.050895
R = math.radians


# --------------------------------------------------------------------------
# Reference model
# --------------------------------------------------------------------------

def poly(c, t):
    return c[0] + c[1] * t + c[2] * t * t


def kepler_bisect(M, e):
    """Solve E - e sin E = M by bisection on [0, 2 pi]."""
    M = M % (2 * math.pi)
    lo, hi = 0.0, 2 * math.pi
    for _ in range(200):
        mid = (lo + hi) / 2
        if mid - e * math.sin(mid) - M < 0:
            lo = mid
        else:
            hi = mid
    return (lo + hi) / 2


def state(el, jd):
    t = (jd - el["epoch"]) / 36525.0
    a = poly(el["a"], t)
    e = poly(el["e"], t)
    i = R(poly(el["i"], t))
    node = R(poly(el["node"], t))
    w = R(poly(el["w"], t))
    M = poly(el["M"], t)
    if el["M"][1] == 0 and el["M"][2] == 0:
        n = GAUSS_DEG / a ** 1.5
        if el["geo"]:
            n /= math.sqrt(SUN_EARTH)
        M += n * (jd - el["epoch"])
    E = kepler_bisect(R(M), e)
    xo = a * (math.cos(E) - e)
    yo = a * math.sqrt(1 - e * e) * math.sin(E)
    # Rz(node) Rx(i) Rz(w) applied to (xo, yo, 0)
    x1 = math.cos(w) * xo - math.sin(w) * yo
    y1 = math.sin(w) * xo + math.cos(w) * yo
    x2 = x1
    y2 = math.cos(i) * y1
    z2 = math.sin(i) * y1
    x3 = math.cos(node) * x2 - math.sin(node) * y2
    y3 = math.sin(node) * x2 + math.cos(node) * y2
    return (x3, y3, z2)


def lonlat(x, y, z):
    r = math.sqrt(x * x + y * y + z * z)
    return (math.degrees(math.atan2(y, x)) % 360.0, math.degrees(math.asin(z / r)), r)


def sph2cart(lon, lat, r):
    return (r * math.cos(R(lat)) * math.cos(R(lon)),
            r * math.cos(R(lat)) * math.sin(R(lon)),
            r * math.sin(R(lat)))


def precess_to_j2000(x, y, z, jd):
    """Meeus 21.5 / 21.7: ecliptic of epoch jd -> ecliptic J2000."""
    T = (jd - J2000) / 36525.0  # epoch0 = jd, centuries from J2000
    t = -T                      # target J2000 is t centuries after epoch0
    s = 1.0 / 3600.0
    eta = R(((47.0029 - 0.06603 * T + 0.000598 * T * T) * t
             + (-0.03302 + 0.000598 * T) * t * t + 0.000060 * t ** 3) * s)
    Pi = R(174.876384 + (3289.4789 * T + 0.60622 * T * T
                         - (869.8089 + 0.50491 * T) * t + 0.03536 * t * t) * s)
    p = R(((5029.0966 + 2.22226 * T - 0.000042 * T * T) * t
           + (1.11113 - 0.000042 * T) * t * t - 0.000006 * t ** 3) * s)
    lon, lat, r = lonlat(x, y, z)
    lam, beta = R(lon), R(lat)
    A = math.cos(eta) * math.cos(beta) * math.sin(Pi - lam) - math.sin(eta) * math.sin(beta)
    B = math.cos(beta) * math.cos(Pi - lam)
    C = math.cos(eta) * math.sin(beta) + math.sin(eta) * math.cos(beta) * math.sin(Pi - lam)
    lam2 = p + Pi - math.atan2(A, B)
    beta2 = math.asin(C)
    return sph2cart(math.degrees(lam2) % 360.0, math.degrees(beta2), r)


def sdelta(b, a):
    d = (a - b) % 360.0
    return d - 360.0 if d > 180 else d


def mk(a, e, i, node, w, M, geo, epoch=J2000):
    return dict(a=a, e=e, i=i, node=node, w=w, M=M, geo=geo, epoch=epoch)


# Element sets (mirror crates/pleiades-fict/src/test_support.rs).
ELA = mk([5, 0.1, 0.01], [0.3, 0.02, 0.01], [10, 2, 0.5], [100, 3, 1], [45, 4, 2], [30, 0, 0], False)
ELB = dict(ELA, geo=True)
ELC = mk([1.5, 0.05, 0], [0.3, 0.01, 0], [20, 1, 0], [40, 2, 0], [60, 3, 0], [10, 109575, 0], True)
ELD = mk([1.5, 0.05, 0], [0.3, 0.01, 0], [20, 1, 0], [40, 2, 0], [60, 3, 0], [10, 0, 50], True)

OPS = {
    "+": lambda a, b: a + b,
    "-": lambda a, b: a - b,
    "*": lambda a, b: a * b,
    "/": lambda a, b: a / b if b != 0 else math.nan,
    "%": lambda a, b: math.fmod(a, b) if b != 0 else math.nan,
}


def fmt(v):
    return "NaN" if v != v else f"{v:.4g}"


def show(name, v):
    print(name, [repr(c) for c in v])


# --------------------------------------------------------------------------
# values
# --------------------------------------------------------------------------

def cmd_values():
    for nm, el in (("ELA", ELA), ("ELB", ELB), ("ELC", ELC), ("ELD", ELD)):
        for dt in (0.5 * 36525.0, -0.3 * 36525.0, 1000.0):
            show(f"{nm} dt={dt}", state(el, J2000 + dt))
    jd = J2000 + 1000.0
    b = lonlat(*state(ELC, jd - 0.5))
    a = lonlat(*state(ELC, jd + 0.5))
    print("ELC motion", repr(sdelta(b[0], a[0])), repr(a[1] - b[1]), repr(a[2] - b[2]))
    print("ELC place", [repr(c) for c in lonlat(*state(ELC, jd))])
    sun = sph2cart(120.0, 30.0, 2.0)
    earth = tuple(-c for c in sun)
    print("earth", [repr(c) for c in earth])
    g = (3 - earth[0], 0 - earth[1], 0 - earth[2])
    print("geo", [repr(c) for c in lonlat(*g)])
    print("s2c 60/30/2", [repr(c) for c in sph2cart(60, 30, 2)])
    print("s2c 359/-45/3", [repr(c) for c in sph2cart(359, -45, 3)])
    for v in ((1, 2, 2), (-1, -2, -2), (3, -4, 12)):
        print("c2e", v, [repr(c) for c in lonlat(*v)])
    E = 1.0
    print("opp", repr(2 * (math.cos(E) - 0.6)), repr(2 * math.sqrt(1 - 0.36) * math.sin(E)))
    for jd0 in (2415020.0, 2433282.42345905):
        print("prec", jd0, [repr(c) for c in precess_to_j2000(1.2, -0.7, 0.4, jd0)])
        print("prec2", jd0, [repr(c) for c in precess_to_j2000(-2.0, 1.5, -1.1, jd0)])
    # seam: the first half-day-stencil where ELC's longitude crosses 360
    found = None
    for k in range(0, 40000):
        j = J2000 + 1000.0 + k * 0.0625
        b = lonlat(*state(ELC, j - 0.5))
        a = lonlat(*state(ELC, j + 0.5))
        if b[0] > 350.0 and a[0] < 10.0 and b[0] > a[0]:
            found = j
            break
    b, a = lonlat(*state(ELC, found - 0.5)), lonlat(*state(ELC, found + 0.5))
    print("seam jd", repr(found), "lon", b[0], a[0])
    print("seam motion", repr(sdelta(b[0], a[0])), repr(a[1] - b[1]), repr(a[2] - b[2]))
    # helio ELA against a fixed Earth (FixedSun 120/30/2) and (StubSun) one-sided
    for label, earth_v in (("FixedSun", earth), ("StubSun", (-1.0, 0.0, 0.0))):
        def geo_at(j, earth_v=earth_v):
            s = state(ELA, j)
            return lonlat(s[0] - earth_v[0], s[1] - earth_v[1], s[2] - earth_v[2])
        j = J2000 + 18262.5
        mid, a, b = geo_at(j), geo_at(j + 0.5), geo_at(j - 0.5)
        if label == "FixedSun":
            print("ELA geo motion", repr(sdelta(b[0], a[0])), repr(a[1] - b[1]), repr(a[2] - b[2]))
            print("ELA geo place", [repr(c) for c in mid])
        else:
            print("after-only", repr(sdelta(mid[0], a[0]) / 0.5), repr((a[1] - mid[1]) / 0.5), repr((a[2] - mid[2]) / 0.5))
            print("before-only", repr(sdelta(b[0], mid[0]) / 0.5), repr((mid[1] - b[1]) / 0.5), repr((mid[2] - b[2]) / 0.5))


# --------------------------------------------------------------------------
# state-disp: mutant-swap harness for KeplerElements::state_at
# --------------------------------------------------------------------------

class Mut:
    def __init__(self, site=None, op=None):
        self.site, self.op = site, op

    def o(self, site, default, a, b):
        return OPS[self.op](a, b) if self.site == site else OPS[default](a, b)


def state_mut(el, jd, m):
    d = m.o("73:24", "-", jd, el["epoch"])
    t = m.o("73:44", "/", d, 36525.0)
    a = poly(el["a"], t)
    e = poly(el["e"], t)
    incl = R(poly(el["i"], t))
    node = R(poly(el["node"], t))
    argp = R(poly(el["w"], t))
    M = poly(el["M"], t)
    cond_and = (el["M"][1] == 0) and (el["M"][2] == 0)
    cond_or = (el["M"][1] == 0) or (el["M"][2] == 0)
    cond = cond_or if m.site == "81:41" else cond_and
    if cond:
        den = m.o("83:57", "*", a, math.sqrt(a))
        dmot = m.o("83:52", "/", GAUSS_DEG, den)
        geo = el["geo"]
        if m.site == "84:28":
            geo = not geo
        if geo:
            dmot = m.o("85:22", "/", dmot, math.sqrt(SUN_EARTH))
        dd = m.o("87:44", "-", jd, el["epoch"])
        step = m.o("87:35", "*", dmot, dd)
        M = m.o("87:27", "+", M, step)
    E = kepler_bisect(R(M), e)
    xo = a * (math.cos(E) - e)
    yo = a * math.sqrt(1 - e * e) * math.sin(E)
    sa, ca = math.sin(argp), math.cos(argp)
    si, ci = math.sin(incl), math.cos(incl)
    sn, cn = math.sin(node), math.cos(node)
    xp = m.o("99:26", "-", m.o("99:21", "*", ca, xo), m.o("99:31", "*", sa, yo))
    yp = m.o("100:26", "+", m.o("100:21", "*", sa, xo), ca * yo)
    x = m.o("101:25", "-", cn * xp, m.o("101:35", "*", m.o("101:30", "*", sn, ci), yp))
    y = m.o("102:25", "+", m.o("102:20", "*", sn, xp), m.o("102:35", "*", m.o("102:30", "*", cn, ci), yp))
    z = m.o("103:20", "*", si, yp)
    return (x, y, z)


STATE_MUTANTS = [
    ("73:44", "%"), ("73:44", "*"), ("73:24", "+"), ("73:24", "/"),
    ("81:41", "||"),
    ("83:52", "*"), ("83:57", "+"), ("83:57", "/"),
    ("84:28", "!="), ("85:22", "%"), ("85:22", "*"),
    ("87:27", "-"), ("87:35", "+"), ("87:44", "+"),
    ("99:26", "+"), ("99:31", "+"),
    ("100:26", "-"), ("100:26", "*"), ("100:21", "/"),
    ("101:25", "+"), ("101:35", "+"), ("101:30", "+"), ("101:30", "/"),
    ("102:25", "-"), ("102:25", "*"), ("102:20", "/"), ("102:30", "/"),
    ("103:20", "+"),
]


def disp_vec(a, b):
    return max(abs(p - q) if not (math.isnan(p) or math.isnan(q)) else math.inf for p, q in zip(a, b))


def cmd_state_disp():
    sets = [ELA, ELB, ELC, ELD]
    dts = [("t=+0.5", 18262.5), ("t=-0.3", -10957.5), ("dt=1000", 1000.0)]
    overall = (math.inf, None)
    print("| Mutant | " + " | ".join(n for n, _ in dts) + " |")
    for site, op in STATE_MUTANTS:
        row = []
        for dn, dt in dts:
            best = 0.0
            for el in sets:
                try:
                    mv = state_mut(el, J2000 + dt, Mut(site, None if op in ("||", "!=") else op))
                except (ValueError, ZeroDivisionError, OverflowError):
                    mv = (math.nan,) * 3
                best = max(best, disp_vec(mv, state(el, J2000 + dt)))
            row.append(best)
            if math.isfinite(best) and best < overall[0]:
                overall = (best, (site, op, dn))
        print(f"| {site} `{op}` | " + " | ".join("NaN" if v == math.inf else f"{v:.4g}" for v in row) + " |")
    print("true minimum", overall)


# --------------------------------------------------------------------------
# other-disp
# --------------------------------------------------------------------------

def cmd_other_disp():
    print("## geocentric_ecliptic / earth (body (3,0,0), Earth (0.866,-1.5,-1))")
    earth = (3 ** 0.5 / 2, -1.5, -1.0)
    body = (3.0, 0.0, 0.0)
    want = lonlat(*(b - e for b, e in zip(body, earth)))
    for comp in range(3):
        for op in "+/":
            g = [OPS["-"](body[i], earth[i]) for i in range(3)]
            g[comp] = OPS[op](body[comp], earth[comp])
            got = lonlat(*g) if not any(x != x or abs(x) == math.inf for x in g) else (math.nan,) * 3
            print(f"geo[{comp}] -> {op}", [fmt(abs(a - b)) for a, b in zip(got, want)])
    for comp in range(3):
        print(f"earth delete - comp {comp}: |disp| = {fmt(2 * abs(earth[comp]))}")

    print("## motion")

    def lonlat_state(el, jd, fixed_earth=(0, 0, 0)):
        s = state(el, jd)
        return lonlat(s[0] - fixed_earth[0], s[1] - fixed_earth[1], s[2] - fixed_earth[2])

    earth_a = (-1.0, 0.0, 0.0)
    earth_b = (3 ** 0.5 / 2, -1.5, -1.0)
    cases = []
    j = J2000 + 1000.0
    cases.append(("geo lin +1000 d", lonlat_state(ELC, j - .5), lonlat_state(ELC, j + .5), 1.0))
    j = 2452598.8125
    cases.append(("seam", lonlat_state(ELC, j - .5), lonlat_state(ELC, j + .5), 1.0))
    j = J2000 + 18262.5
    cases.append(("helio fixed earth", lonlat_state(ELA, j - .5, earth_b), lonlat_state(ELA, j + .5, earth_b), 1.0))
    cases.append(("one-sided after", lonlat_state(ELA, j, earth_a), lonlat_state(ELA, j + .5, earth_a), 0.5))
    cases.append(("one-sided before", lonlat_state(ELA, j - .5, earth_a), lonlat_state(ELA, j, earth_a), 0.5))

    def speeds(before, after, full, mut=None):
        f = full
        if mut == "111*+":
            f = 0.5 + 2.0
        if mut == "111*/":
            f = 0.5 / 2.0
        if full == 0.5:
            f = 0.5

        def pick(site, default, a, b):
            return OPS[mut[1]](a, b) if mut and mut[0] == site else OPS[default](a, b)
        if mut == "105-":
            before, after = after, before
        lon = pick("117:91", "/", sdelta(before[0], after[0]), f)
        dlat = (after[1] - before[1]) if not (mut and mut[0] == "118:51") else OPS[mut[1]](after[1], before[1])
        lat = pick("118:80", "/", dlat, f)
        ddist = (after[2] - before[2]) if not (mut and mut[0] == "120:43") else OPS[mut[1]](after[2], before[2])
        dist = pick("120:48", "/", ddist, f)
        return lon, lat, dist

    mutants = [
        ("105:64 +->-", "105-"), ("111:61 *->+", "111*+"), ("111:61 *->/", "111*/"),
        ("117:91 /->%", ("117:91", "%")), ("117:91 /->*", ("117:91", "*")),
        ("118:80 /->%", ("118:80", "%")), ("118:80 /->*", ("118:80", "*")),
        ("118:51 -->+", ("118:51", "+")), ("118:51 -->/", ("118:51", "/")),
        ("120:48 /->%", ("120:48", "%")), ("120:48 /->*", ("120:48", "*")),
        ("120:43 -->+", ("120:43", "+")), ("120:43 -->/", ("120:43", "/")),
    ]
    print("| Mutant | " + " | ".join(c[0] for c in cases) + " |")
    for name, mut in mutants:
        row = []
        for _cn, b, a, full in cases:
            if mut in ("111*+", "111*/", "105-") and full == 0.5:
                row.append("n/a")
                continue
            o = speeds(b, a, full)
            m = speeds(b, a, full, mut)
            comp = {"117": 0, "118": 1, "120": 2}.get(name[:3], None)
            d = max(abs(x - y) for x, y in zip(o, m)) if comp is None else abs(o[comp] - m[comp])
            row.append(fmt(d))
        print(f"| {name} | " + " | ".join(row) + " |")

    print("## spherical_to_cartesian")
    tests = [(30, 0, 2), (60, 30, 2), (359, -45, 3), (77, 90, 4), (77, -90, 4)]

    def s2c_mut(lon, lat, r, site=None, op=None):
        lo, la = R(lon), R(lat)

        def f(s, d, a, b):
            return OPS[op](a, b) if s == site else OPS[d](a, b)
        x = f("173:23", "*", f("173:11", "*", r, math.cos(la)), math.cos(lo))
        y = f("174:23", "*", f("174:11", "*", r, math.cos(la)), math.sin(lo))
        z = f("175:11", "*", r, math.sin(la))
        return (x, y, z)
    for site, op in [("173:11", "+"), ("173:11", "/"), ("173:23", "+"), ("173:23", "/"),
                     ("174:11", "+"), ("174:11", "/"), ("174:23", "+"), ("174:23", "/"), ("175:11", "+")]:
        row = []
        for t in tests:
            o = s2c_mut(*t)
            m = s2c_mut(*t, site, op)
            row.append(fmt(max(abs(p - q) for p, q in zip(o, m))))
        print(f"| {site} `{op}` | " + " | ".join(row) + " |")
    print("tests", tests)
    # Each of the 27 constant-tuple mutants is covered by the poles: the
    # north-pole reference is (0, 0, 4) and the south pole (0, 0, -4); a
    # tuple's z is in {-1, 0, 1}, so max(|z - 4|, |z + 4|) >= 4.
    rows = []
    for tup in itertools.product((0.0, 1.0, -1.0), repeat=3):
        best = max(max(abs(p - q) for p, q in zip(s2c_mut(*t), tup)) for t in tests)
        rows.append((best, tup))
    rows.sort()
    print("27 constant tuples, weakest best-margins:", [(fmt(b), t) for b, t in rows[:3]])

    print("## cartesian_to_ecliptic")

    def c2e_r(x, y, z, ov):
        def f(s, d, a, b):
            return OPS[ov[1]](a, b) if ov and s == ov[0] else OPS[d](a, b)
        xx = f("180:16", "*", x, x)
        yy = f("180:24", "*", y, y)
        zz = f("180:32", "*", z, z)
        s1 = f("180:20", "+", xx, yy)
        s2 = f("180:28", "+", s1, zz)
        return math.sqrt(s2) if s2 >= 0 else math.nan
    for t in [(1, 2, 2), (-1, -2, -2), (3, -4, 12)]:
        print("input", t)
        for site, op in [("180:28", "-"), ("180:28", "*"), ("180:20", "*"), ("180:16", "/"), ("180:24", "/"), ("180:32", "+")]:
            r0, r1 = c2e_r(*t, None), c2e_r(*t, (site, op))
            print(f"  {site} `{op}`: r {fmt(r0)} -> {fmt(r1)}  disp {fmt(abs(r0 - r1))}")
        z, r = t[2], c2e_r(*t, None)
        print("  185:12 `/`->`%`: asin argument", z / r, "->", math.fmod(z, r))

    print("## orbital_plane_position (a=2, e=0.6, E=1)")
    a, e, E = 2.0, 0.6, 1.0
    x0, y0 = a * (math.cos(E) - e), a * math.sqrt(1 - e * e) * math.sin(E)
    print("x0,y0", x0, y0)
    print("26:49 -->+ dx", fmt(abs(a * (math.cos(E) + e) - x0)))
    print("27:18 *->/ dy", fmt(abs(a / math.sqrt(1 - e * e) * math.sin(E) - y0)))
    print("27:25 -->+ dy", fmt(abs(a * math.sqrt(1 + e * e) * math.sin(E) - y0)))
    print("27:29 *->+ dy", "NaN (sqrt of negative)" if 1 - (e + e) < 0 else "?")


# --------------------------------------------------------------------------
# frame-disp
# --------------------------------------------------------------------------

def frame_pipeline(x, y, z, jd, mut=None):
    """frame.rs after the identity branch, with one mutated site."""
    sq = {"12:28+-": lambda: x * x + y * y - z * z, "12:28+*": lambda: x * x + y * y * z * z,
          "12:32*+": lambda: x * x + y * y + (z + z)}.get(mut)
    r = math.sqrt(sq()) if sq else math.sqrt(x * x + y * y + z * z)
    lon = math.degrees(math.atan2(y, x)) % 360.0
    q = z / r
    if mut == "17:18/%":
        q = math.fmod(z, r)
    if mut == "17:18/*":
        q = z * r
    lo, hi = (1.0, 1.0) if mut == "17:29" else (-1.0, 1.0)
    q = min(max(q, lo), hi)
    lat = math.degrees(math.asin(q))
    ux, uy, uz = sph2cart(lon, lat, 1.0)
    px, py, pz = precess_to_j2000(ux, uy, uz, jd)
    lon2, lat2, _ = lonlat(px, py, pz)
    lon_r, lat_r = R(lon2), R(lat2)
    zc = r * math.sin(lat_r)
    if mut == "24:11*/":
        zc = r / math.sin(lat_r)
    return (r * math.cos(lat_r) * math.cos(lon_r), r * math.cos(lat_r) * math.sin(lon_r), zc)


def cmd_frame_disp():
    J1900, B1950 = 2415020.0, 2433282.42345905
    vecs = [((1.2, -0.7, 0.4), J1900), ((-2.0, 1.5, -1.1), J1900), ((1.2, -0.7, 0.4), B1950), ((-2.0, 1.5, -1.1), B1950)]
    for mut in ["12:28+-", "12:28+*", "12:32*+", "17:18/%", "17:18/*", "17:29", "24:11*/"]:
        row = []
        for v, jd in vecs:
            o = precess_to_j2000(*v, jd)
            m = frame_pipeline(*v, jd, mut)
            row.append(fmt(max(abs(a - b) for a, b in zip(o, m))))
        print(mut, row)
    for v, jd in vecs:
        m = precess_to_j2000(*v, jd)
        print("9:45 `>`/`==` at", jd, "disp", fmt(max(abs(a - b) for a, b in zip(v, m))))
    for off in (0.0, 5e-7, -5e-7, 9e-7, -9e-7):
        for v in ((1.2, -0.7, 0.4), (-2.0, 1.5, -1.1)):
            m = precess_to_j2000(*v, J2000 + off)
            print("J2000", off, v, "precession-path disp", max(abs(a - b) for a, b in zip(v, m)))

    def newton(m, e, start):
        E = start
        for _ in range(64):
            d = (E - e * math.sin(E) - m) / (1 - e * math.cos(E))
            E -= d
            if abs(d) < 1e-14:
                break
        return E
    for e in (0.8, 0.9, 0.981092, 0.99):
        for deg in (0.5, 7.5, 9.5, 15.0, 17.0, 90.0, 180.0, 270.0, 343.0, 359.5):
            m = math.radians(deg)
            E = newton(m, e, m)
            res = abs(E - e * math.sin(E) - m)
            if res > 1e-12:
                print("kepler `<`->`>`: e", e, "M", deg, "deg, residual", fmt(res))


# --------------------------------------------------------------------------
# kepler-equiv
# --------------------------------------------------------------------------

def kepler_variant(M, e, start_rule, stop_rule):
    m = M % (2 * math.pi)
    if start_rule == "orig":
        E = m if e < 0.8 else math.pi
    elif start_rule == "le":
        E = m if e <= 0.8 else math.pi
    else:
        E = m if e == 0.8 else math.pi
    for _ in range(64):
        d = (E - e * math.sin(E) - m) / (1 - e * math.cos(E))
        E -= d
        if stop_rule == "orig" and abs(d) < 1e-14:
            break
        if stop_rule == "le" and abs(d) <= 1e-14:
            break
        if stop_rule == "eq" and abs(d) == 1e-14:
            break
    return E


def cmd_kepler_equiv():
    """Max |E_mutant - E_original| for the four equivalent mutants."""
    worst = {}
    es = [i / 100 for i in range(0, 80)] + [0.8, 0.9, 0.981092, 0.99]
    for e in es:
        for k in range(0, 721):
            M = k * 2 * math.pi / 720
            base = kepler_variant(M, e, "orig", "orig")
            for name, (a, b) in {"8:33 <=": ("le", "orig"), "8:33 ==": ("eq", "orig"),
                                 "16:24 <=": ("orig", "le"), "16:24 ==": ("orig", "eq")}.items():
                dd = abs(kepler_variant(M, e, a, b) - base)
                if dd > worst.get(name, (0,))[0]:
                    worst[name] = (dd, e, k)
    for k, v in worst.items():
        print(k, v)


COMMANDS = {
    "values": cmd_values,
    "state-disp": cmd_state_disp,
    "other-disp": cmd_other_disp,
    "frame-disp": cmd_frame_disp,
    "kepler-equiv": cmd_kepler_equiv,
}

if __name__ == "__main__":
    if len(sys.argv) != 2 or sys.argv[1] not in COMMANDS:
        sys.exit(__doc__)
    COMMANDS[sys.argv[1]]()
