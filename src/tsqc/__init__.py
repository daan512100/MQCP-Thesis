# src/tsqc/__init__.py
# Dit bestand maakt van de 'tsqc' directory een Python-package.
# Het fungeert ook als het publieke toegangspunt tot de bibliotheek.
"""
TSQC: Tabu Search for Quasi-Clique

Een high-performance solver voor het Maximum Quasi-Clique Probleem,
geïmplementeerd in Rust met een gebruiksvriendelijke Python API.
"""

# We importeren de specifieke objecten uit onze interne api.py module.
# Dit zijn de enige objecten die we publiekelijk willen tonen aan de gebruiker.
from .api import (
    solve_fixed,
    solve_max,
    parse_dimacs,
    SolutionData,
    Params,
)

# De __all__ variabele is een conventie in Python die expliciet definieert
# welke namen geïmporteerd worden wanneer een gebruiker `from tsqc import *`
# zou gebruiken. Het is ook een duidelijke indicatie van de publieke API.
__all__ = [
    "solve_fixed",
    "solve_max",
    "parse_dimacs",
    "SolutionData",
    "Params",
]