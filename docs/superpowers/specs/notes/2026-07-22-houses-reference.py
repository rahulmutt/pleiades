#!/usr/bin/env python3
"""Independent reference for FU-9 pleiades-houses Foundation PR.

Ports the published Swiss Ephemeris `swehouse.c` Asc1/Asc2 and the
`swe_houses_armc` chart-point set, plus the elementary trig helpers, from the
PUBLISHED formulas (Meeus ch. 26 / swehouse.c), independently of the crate, so
the pinned test literals are non-circular.
"""
import math

D2R = math.pi / 180.0
R2D = 180.0 / math.pi


def norm360(x):
    return x % 360.0


def ascendant_for(st_deg, lat_deg, obl_rad):
    """crate `ascendant_for`: atan2(cosθ, -(sinθ·cosε + tanφ·sinε))."""
    th = st_deg * D2R
    la = lat_deg * D2R
    return norm360(
        math.degrees(
            math.atan2(
                math.cos(th),
                -(math.sin(th) * math.cos(obl_rad) + math.tan(la) * math.sin(obl_rad)),
            )
        )
    )


def mc_from(armc_deg, obl_rad):
    th = armc_deg * D2R
    return norm360(math.degrees(math.atan2(math.sin(th), math.cos(th) * math.cos(obl_rad))))


def opposite(x):
    return norm360(x + 180.0)


def asc2(x, pole_height, sine, cose):
    """crate `asc2` (swehouse.c Asc2)."""
    value = -math.tan(pole_height * D2R) * sine + cose * math.cos(x * D2R)
    if abs(value) < 1.0e-12:
        value = 0.0
    sinx = math.sin(x * D2R)
    if abs(sinx) < 1.0e-12:
        lon = -1.0e-12 if value < 0.0 else 1.0e-12
    elif value == 0.0:
        lon = -90.0 if sinx < 0.0 else 90.0
    else:
        lon = math.degrees(math.atan(sinx / value))
    if lon < 0.0:
        lon += 180.0
    return lon


def asc1(x1, pole_height, sine, cose):
    """crate `asc1` (swehouse.c Asc1)."""
    x1 = norm360(x1)
    q = int(math.floor(x1 / 90.0)) + 1
    if q == 1:
        lon = asc2(x1, pole_height, sine, cose)
    elif q == 2:
        lon = 180.0 - asc2(180.0 - x1, -pole_height, sine, cose)
    elif q == 3:
        lon = 180.0 + asc2(x1 - 180.0, -pole_height, sine, cose)
    else:
        lon = 360.0 - asc2(360.0 - x1, pole_height, sine, cose)
    return norm360(lon)


def asc_mc_from(armc, lat, obl_deg):
    """crate `asc_mc_from`: the swe_houses_armc chart-point set."""
    obl = obl_deg * D2R
    asc = ascendant_for(armc, lat, obl)
    mc = mc_from(armc, obl)
    eq_asc = ascendant_for(armc, 0.0, obl)
    f_pole = 90.0 - lat if lat >= 0.0 else -90.0 - lat
    vertex = ascendant_for(armc - 180.0, f_pole, obl)
    if abs(lat) <= obl_deg:
        vemc = norm360(vertex - mc)
        if vemc > 180.0:
            vemc -= 360.0
        if vemc > 0.0:
            vertex = norm360(vertex + 180.0)
    coasc_koch = opposite(ascendant_for(armc - 180.0, lat, obl))
    coasc_munk = ascendant_for(armc, f_pole, obl)
    polasc = opposite(coasc_koch)
    return dict(asc=asc, mc=mc, vertex=vertex, eq_asc=eq_asc,
               coasc_koch=coasc_koch, coasc_munk=coasc_munk, polasc=polasc)


def spherical_cotrans(lon, lat, radius, angle_deg):
    """crate `spherical_cotrans`: rotation about the x-axis by angle_deg."""
    lo, la = lon * D2R, lat * D2R
    x = radius * math.cos(la) * math.cos(lo)
    y = radius * math.cos(la) * math.sin(lo)
    z = radius * math.sin(la)
    a = angle_deg * D2R
    y_rot = y * math.cos(a) + z * math.sin(a)
    z_rot = -y * math.sin(a) + z * math.cos(a)
    r = math.sqrt(x * x + y_rot * y_rot + z_rot * z_rot)
    return (
        math.degrees(math.atan2(y_rot, x)),
        math.degrees(math.atan2(z_rot, math.sqrt(x * x + y_rot * y_rot))),
        r,
    )


def interp(start, end, frac):
    span = norm360(end - start)
    return norm360(start + span * frac)


def porphyry(asc, mc):
    desc, ic = opposite(asc), opposite(mc)
    return [
        asc,
        interp(asc, ic, 1 / 3), interp(asc, ic, 2 / 3), ic,
        interp(ic, desc, 1 / 3), interp(ic, desc, 2 / 3), desc,
        interp(desc, mc, 1 / 3), interp(desc, mc, 2 / 3), mc,
        interp(mc, asc, 1 / 3), interp(mc, asc, 2 / 3),
    ]


def ra_from_lon(lon, obl_deg):
    obl = obl_deg * D2R
    lo = lon * D2R
    return math.degrees(math.atan2(math.sin(lo) * math.cos(obl), math.cos(lo)))


EPS = 23.4366  # a representative true obliquity of date (deg)
SINE, COSE = math.sin(EPS * D2R), math.cos(EPS * D2R)


def fmt(x):
    return f"{x:.12f}"


def signed_diff(a, b):
    """crate `signed_longitude_difference`: (a-b) normalized to [-180, 180)."""
    delta = norm360(a - b)
    return delta - 360.0 if delta >= 180.0 else delta


def _complete_opposite(c):
    """crate `complete_opposite_houses`: cusps 4-9 as ecliptic antipodes."""
    c[3] = opposite(c[9])
    c[4] = opposite(c[10])
    c[5] = opposite(c[11])
    c[6] = opposite(c[0])
    c[7] = opposite(c[1])
    c[8] = opposite(c[2])


def pullen_sd(asc_deg, mc_deg):
    """crate `pullen_sd_houses` == `albategnius_houses` (byte-identical): the
    published equal-quadrant (Albategnian) split of each ASC/MC quadrant.
    Elementary published arithmetic; re-derived here, not copied from the Rust."""
    asc = asc_deg
    acmc = signed_diff(asc, mc_deg)
    if acmc < 0.0:
        asc = opposite(asc)
        acmc = signed_diff(asc, mc_deg)
    c = [0.0] * 12
    c[0] = norm360(asc)
    c[9] = norm360(mc_deg)
    q1 = 180.0 - acmc
    d = (acmc - 90.0) / 4.0
    if acmc <= 30.0:
        c[10] = norm360(mc_deg + acmc / 2.0)
        c[11] = c[10]
    else:
        c[10] = norm360(mc_deg + 30.0 + d)
        c[11] = norm360(mc_deg + 60.0 + 3.0 * d)
    d = (q1 - 90.0) / 4.0
    if q1 <= 30.0:
        c[1] = norm360(asc + q1 / 2.0)
        c[2] = c[1]
    else:
        c[1] = norm360(asc + 30.0 + d)
        c[2] = norm360(asc + 60.0 + 3.0 * d)
    _complete_opposite(c)
    return c


def _sr_ratio(q):
    """Pullen SR ratio r: the positive real root of r^4 + 2 r^3 - 2 c r - c = 0,
    with c=(180-q)/q, derived from the published SR symmetric-arc property
    x*r^3*(2 + r) = 180 - q (small quadrant split xr:x:xr summing to q). Solved by
    bracketed bisection + Newton polish -- a genuinely different method than the
    crate's Ferrari closed form, so agreement cross-validates both. Sanity:
    q=90 -> c=1 -> r=1 (equal division)."""
    c = (180.0 - q) / q

    def f(r):
        return r ** 4 + 2.0 * r ** 3 - 2.0 * c * r - c

    def fp(r):
        return 4.0 * r ** 3 + 6.0 * r ** 2 - 2.0 * c

    lo, hi = 1.0, c + 3.0            # f(1)=3(1-c)<=0, f(hi)>0 for c>=1
    for _ in range(200):
        mid = 0.5 * (lo + hi)
        if f(mid) <= 0.0:
            lo = mid
        else:
            hi = mid
    r = 0.5 * (lo + hi)
    for _ in range(60):              # Newton polish to ~machine precision
        r -= f(r) / fp(r)
    return r


def pullen_sr(asc_deg, mc_deg):
    """crate `pullen_sr_houses`: Pullen Sinusoidal Ratio house division."""
    asc = asc_deg
    acmc = signed_diff(asc, mc_deg)
    if acmc < 0.0:
        asc = opposite(asc)
        acmc = signed_diff(asc, mc_deg)
    c = [0.0] * 12
    c[0] = norm360(asc)
    c[9] = norm360(mc_deg)
    q = acmc
    if q > 90.0:
        q = 180.0 - q
    if q < 1.0e-30:
        x, xr, xr3, xr4 = 0.0, 0.0, 0.0, 180.0
    else:
        r = _sr_ratio(q)
        x = q / (2.0 * r + 1.0)
        xr = r * x
        xr3 = xr * r * r
        xr4 = xr3 * r
    if acmc > 90.0:
        c[10] = norm360(mc_deg + xr3)
        c[11] = norm360(c[10] + xr4)
        c[1] = norm360(asc + xr)
        c[2] = norm360(c[1] + x)
    else:
        c[10] = norm360(mc_deg + xr)
        c[11] = norm360(c[10] + x)
        c[1] = norm360(asc + xr3)
        c[2] = norm360(c[1] + xr4)
    _complete_opposite(c)
    return c


def apc_sector(n, lat_rad, obl_rad, sid_rad):
    """crate `apc_sector`: APC in-between-sector longitude (SE 'B'/APC family).
    Independent port from the published APC algorithm; NOT copied from the Rust."""
    tan_lat = math.tan(lat_rad)
    tan_obl = math.tan(obl_rad)
    kv = math.atan2(tan_lat * tan_obl * math.cos(sid_rad),
                    1.0 + tan_lat * tan_obl * math.sin(sid_rad))
    sin_kv = math.sin(kv)
    below = n < 8
    k = float(n - 1) if below else float(n - 13)
    if below:
        a = kv + sid_rad + math.pi / 2 + k * (math.pi / 2 - kv) / 3.0
    else:
        a = kv + sid_rad + math.pi / 2 + k * (math.pi / 2 + kv) / 3.0
    y = sin_kv * math.sin(sid_rad) + math.sin(a)
    x = math.cos(obl_rad) * (sin_kv * math.cos(sid_rad) + math.cos(a)) \
        + math.sin(obl_rad) * tan_lat * math.sin(sid_rad - a)
    return norm360(math.degrees(math.atan2(y, x)))


if __name__ == "__main__":
    print("# spherical_cotrans([40,25,2], 15):")
    print("  ", tuple(fmt(v) for v in spherical_cotrans(40.0, 25.0, 2.0, 15.0)))

    print("# asc2 direct (pole=52, sine=sinEPS, cose=cosEPS):")
    for x in (30.0, 120.0, 210.0, 300.0):
        print(f"  asc2({x}) = {fmt(asc2(x, 52.0, SINE, COSE))}")

    print("# asc1 per quadrant (pole=52, EPS):")
    for x in (30.0, 120.0, 210.0, 300.0):
        print(f"  asc1({x}) = {fmt(asc1(x, 52.0, SINE, COSE))}")

    # Degenerate-guard pins (FU-9 follow-up): x on the sinx~0 axis exercises
    # asc2's sinx.abs()<1e-12 branch, which the normal-atan quadrant inputs
    # never reach. asc2(0)=+1e-12 sentinel; asc2(180) folds -1e-12 -> ~180.
    print("# asc2 degenerate sinx~0 branch (pole=52):")
    print(f"  asc2(0)   = {asc2(0.0, 52.0, SINE, COSE)!r}")
    print(f"  asc2(180) = {asc2(180.0, 52.0, SINE, COSE)!r}")

    print("# asc_mc_from — G1 lat>obl non-flip (armc=45, lat=52, obl=EPS):")
    for k, v in asc_mc_from(45.0, 52.0, EPS).items():
        print(f"    {k:12s} = {fmt(v)}")
    print("# asc_mc_from — G2 0<lat<=obl flip branch (armc=200, lat=10, obl=EPS):")
    for k, v in asc_mc_from(200.0, 10.0, EPS).items():
        print(f"    {k:12s} = {fmt(v)}")
    print("# asc_mc_from — G3 southern -90-lat branch (armc=100, lat=-33, obl=EPS):")
    for k, v in asc_mc_from(100.0, -33.0, EPS).items():
        print(f"    {k:12s} = {fmt(v)}")
    # G4 (FU-9 follow-up): a geometry where the vertex flip ACTUALLY fires
    # (vemc>0 rotates the vertex by 180). G2 enters the flip block but does not
    # flip, so the flip guards/arithmetic stay uncovered until this geometry.
    print("# asc_mc_from — G4 vertex flip fires (armc=15, lat=5, obl=EPS):")
    for k, v in asc_mc_from(15.0, 5.0, EPS).items():
        print(f"    {k:12s} = {fmt(v)}")

    # FU-9 final-review fix (2026-07-23): the equivalence sweep never sampled
    # lat=0, where the pole height is exactly +-90 deg and tan is NOT
    # 180-periodic in f64 (tan(90 deg) != tan(-90 deg) in float arithmetic).
    # These equator geometries kill mutants C1-C3 (mod.rs 192/195/204/207) and
    # I1 (mod.rs 195, southern branch at lat just below 0).
    print("# asc_mc_from — equator degeneracy, C1 (armc=45, lat=0, obl=EPS):")
    for k, v in asc_mc_from(45.0, 0.0, EPS).items():
        print(f"    {k:12s} = {v!r}")
    print("# asc_mc_from — equator degeneracy, C2 (armc=0, lat=0, obl=EPS):")
    for k, v in asc_mc_from(0.0, 0.0, EPS).items():
        print(f"    {k:12s} = {v!r}")
    print("# asc_mc_from — equator degeneracy, C3 (armc=180, lat=0, obl=EPS):")
    for k, v in asc_mc_from(180.0, 0.0, EPS).items():
        print(f"    {k:12s} = {v!r}")
    print("# asc_mc_from — I1 southern just-below-equator (armc=45, lat=-1e-16, obl=EPS):")
    for k, v in asc_mc_from(45.0, -1e-16, EPS).items():
        print(f"    {k:12s} = {v!r}")

    # asc2's guard at value.abs()<1e-12 assigns value=0.0, which makes the
    # value<0.0 comparison reachable at equality (C4/C5). pole=90-EPS=66.5634
    # is the f_pole of an observer at latitude=obliquity; it drives value to
    # exactly 0.0 at x=0.
    print("# asc2 value==0.0 guard pins (C4/C5, pole = 90-EPS and a neighbor):")
    print(f"  asc2(0, 66.5634,          sine, cose) = {asc2(0.0, 66.5634, SINE, COSE)!r}")
    print(f"  asc2(0, 66.5634000000001, sine, cose) = {asc2(0.0, 66.5634000000001, SINE, COSE)!r}")

    # I2 sanity check: independently confirm the two "periodicity" claims are
    # only sub-tolerance, not bit-identical, by sweeping a lat/armc grid and
    # taking the max circular difference (min(d, 360-d)).
    def circ_diff(a, b):
        d = abs(norm360(a) - norm360(b))
        return min(d, 360.0 - d)

    max_asc_diff = 0.0
    for armc in range(0, 360, 3):
        for lat in (-66.0, -33.0, -10.0, 10.0, 33.0, 66.0):
            obl = EPS * D2R
            a = ascendant_for(armc - 180.0, lat, obl)
            b = ascendant_for(armc + 180.0, lat, obl)
            max_asc_diff = max(max_asc_diff, circ_diff(a, b))
    print(f"# I2 sanity: max circular diff ascendant_for(armc-180) vs (armc+180) = {max_asc_diff!r}")

    max_fold_diff = 0.0
    for x in (v * 0.1 for v in range(0, 3600)):
        max_fold_diff = max(max_fold_diff, circ_diff((x + 180.0) % 360.0, (x - 180.0) % 360.0))
    print(f"# I2 sanity: max circular diff (x+180)%360 vs (x-180)%360 = {max_fold_diff!r}")

    max_asc2_x_diff = 0.0
    for x in (v * 0.3 for v in range(0, 1200)):
        a = asc2(x - 180.0, -52.0, SINE, COSE)
        b = asc2(x + 180.0, -52.0, SINE, COSE)
        max_asc2_x_diff = max(max_asc2_x_diff, abs(a - b))
    print(f"# I2 sanity: max diff asc2(x-180) vs asc2(x+180) (pole=-52) = {max_asc2_x_diff!r}")

    print("# interpolate_longitude(350, 20, 0.25):", fmt(interp(350.0, 20.0, 0.25)))
    print("# porphyry(asc=100, mc=10):", [fmt(v) for v in porphyry(100.0, 10.0)])
    print("# ra_from_lon(60, EPS):", fmt(ra_from_lon(60.0, EPS)))

    print("# apc_sector — lat=52, obl=EPS, sidereal=45 (non-degenerate):")
    _lat, _obl, _sid = math.radians(52.0), math.radians(EPS), math.radians(45.0)
    for _n in range(1, 13):
        print(f"    apc_sector({_n:2d}) = {fmt(apc_sector(_n, _lat, _obl, _sid))}")

    print("# whole_sign first_cusp(asc=95):", fmt(math.floor(95.0 / 30.0) * 30.0))

    print("# pullen_sd / albategnius (byte-identical) -- all 12 cusps:")
    for name, a, m in [("200/100", 200.0, 100.0), ("120/100", 120.0, 100.0),
                       ("260/100", 260.0, 100.0), ("10/100(flip)", 10.0, 100.0),
                       ("100/100(deg)", 100.0, 100.0)]:
        print(f"  [{name}]", [fmt(v) for v in pullen_sd(a, m)])
    print("# pullen_sr -- all 12 cusps:")
    for name, a, m in [("200/100", 200.0, 100.0), ("140/100", 140.0, 100.0),
                       ("10/100(flip)", 10.0, 100.0), ("100/100(guard)", 100.0, 100.0)]:
        print(f"  [{name}]", [fmt(v) for v in pullen_sr(a, m)])
    print("# _sr_ratio(90) sanity (must be 1.0):", _sr_ratio(90.0))
