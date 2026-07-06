# pleiades-fict

Fictitious/hypothetical body backend for the pleiades astrology workspace.

Computes the Swiss-Ephemeris default `seorbel.txt` fictitious bodies (SE numbers
40–58: the Uranian/Hamburg planets, Transpluto, Vulcan, White Moon, Waldemath,
and the historical pre-discovery Neptune/Pluto predictions) from committed
osculating orbital elements via an unperturbed Kepler orbit. These bodies are
*definitional*: correctness means parity with SE's `seorbel.txt`-driven output,
enforced by the `validate-fictitious` gate.
