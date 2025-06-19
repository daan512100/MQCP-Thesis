# src/tsqc/benchmark_cases.py
"""
Gedefinieerde benchmarks uit de oude projectconfiguratie:
Elke case specificeert een DIMACS-bestand, een γ-waarde en een target-k.
"""
from pathlib import Path
from typing import NamedTuple, List


class BenchmarkCase(NamedTuple):
    """Eén benchmark-run: instantie, γ-waarde en target-k."""
    instance: str  # bestandsnaam van .clq
    gamma: float   # dichtheidsdrempel
    k: int         # target quasi-clique grootte
    stagnation_iter: int # NIEUW VELD: Aantal iteraties zonder verbetering voordat diversificatie optreedt


BENCHMARK_CASES: List[BenchmarkCase] = [
    # γ = 0.85
    BenchmarkCase("p_hat300-1.clq",  0.85,  12, 1_000),
    BenchmarkCase("brock200_2.clq",  0.85,  19, 1_000),
    BenchmarkCase("keller4.clq",     0.85,  31, 1_000),

]