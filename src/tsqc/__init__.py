# src/tsqc/__init__.py
# Dit bestand maakt van de 'tsqc' directory een Python-package.
# Het fungeert ook als het publieke toegangspunt tot de bibliotheek.

"""
TSQC: Tabu Search for Quasi-Clique

Een high-performance solver voor het Maximum Quasi-Clique Probleem,
geïmplementeerd in Rust met een gebruiksvriendelijke Python API.
"""

# =================================================================================
# CORRECTIE: De geïmporteerde namen zijn gesynchroniseerd met de inhoud
# van de 'api.py' module.
# - 'solve' is vervangen door de specifiekere 'solve_fixed' en 'solve_max'.
# - 'read_dimacs_graph' is vervangen door 'parse_dimacs'.
# - 'SolutionData' en 'Params' worden nu ook correct geëxporteerd.
# 'Graph' en 'Solution' zijn interne Rust-structs en worden niet direct
# aan de eindgebruiker getoond, dus die worden hier niet geïmporteerd.
# =================================================================================
from .api import (
    solve_fixed,
    solve_max,
    parse_dimacs,
    SolutionData,
    Params,
)

# Definieer __all__ om expliciet aan te geven welke objecten deel uitmaken van
# de publieke API. Dit helpt tools zoals linters en IDEs, en voorkomt dat
# interne objecten per ongeluk worden geëxporteerd.
__all__ = [
    "solve_fixed",
    "solve_max",
    "parse_dimacs",
    "SolutionData",
    "Params",
]