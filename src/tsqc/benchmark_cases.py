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

    BenchmarkCase("frb30-15-1.clq",  0.95,  60, 1000),
    BenchmarkCase("frb30-15-2.clq",  0.95,  58, 1000),
    BenchmarkCase("frb40-19-1.clq",  0.95,  114, 1000),
    BenchmarkCase("frb40-19-2.clq",  0.95,  105, 1000),

]