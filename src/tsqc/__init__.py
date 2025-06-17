"""
Python façade voor de native Rust-extensie.

Dit bestand maakt de kernfunctionaliteit van de Rust-bibliotheek
direct beschikbaar onder de `tsqc` namespace.
"""

from._native import solve_k_py, solve_max_py, parse_dimacs_py

__all__ = [
    "solve_k_py",
    "solve_max_py",
    "parse_dimacs_py",
]

__version__ = "4.0.0"