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


if __name__ == "__main__":
    print("# spherical_cotrans([40,25,2], 15):")
    print("  ", tuple(fmt(v) for v in spherical_cotrans(40.0, 25.0, 2.0, 15.0)))

    print("# asc2 direct (pole=52, sine=sinEPS, cose=cosEPS):")
    for x in (30.0, 120.0, 210.0, 300.0):
        print(f"  asc2({x}) = {fmt(asc2(x, 52.0, SINE, COSE))}")

    print("# asc1 per quadrant (pole=52, EPS):")
    for x in (30.0, 120.0, 210.0, 300.0):
        print(f"  asc1({x}) = {fmt(asc1(x, 52.0, SINE, COSE))}")

    print("# asc_mc_from — G1 lat>obl non-flip (armc=45, lat=52, obl=EPS):")
    for k, v in asc_mc_from(45.0, 52.0, EPS).items():
        print(f"    {k:12s} = {fmt(v)}")
    print("# asc_mc_from — G2 0<lat<=obl flip branch (armc=200, lat=10, obl=EPS):")
    for k, v in asc_mc_from(200.0, 10.0, EPS).items():
        print(f"    {k:12s} = {fmt(v)}")
    print("# asc_mc_from — G3 southern -90-lat branch (armc=100, lat=-33, obl=EPS):")
    for k, v in asc_mc_from(100.0, -33.0, EPS).items():
        print(f"    {k:12s} = {fmt(v)}")

    print("# interpolate_longitude(350, 20, 0.25):", fmt(interp(350.0, 20.0, 0.25)))
    print("# porphyry(asc=100, mc=10):", [fmt(v) for v in porphyry(100.0, 10.0)])
    print("# ra_from_lon(60, EPS):", fmt(ra_from_lon(60.0, EPS)))
    print("# whole_sign first_cusp(asc=95):", fmt(math.floor(95.0 / 30.0) * 30.0))
