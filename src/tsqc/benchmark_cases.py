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

   BenchmarkCase("brock200_4.clq",  0.85,  39, 1_000),
    BenchmarkCase("p_hat300-3.clq",  0.85, 180, 1_000),
    BenchmarkCase("brock400_2.clq",  0.85, 100, 1_000),
    BenchmarkCase("brock400_4.clq",  0.85, 102, 1_000),
    BenchmarkCase("p_hat700-1.clq",  0.85,  19, 1_000),
    BenchmarkCase("p_hat700-2.clq",  0.85, 223, 1_000),
    BenchmarkCase("p_hat700-3.clq",  0.85, 430, 1_000),
    BenchmarkCase("p_hat1500-2.clq", 0.85, 487, 1_000),
    BenchmarkCase("p_hat1500-3.clq", 0.85, 943, 1_000),
    BenchmarkCase("keller5.clq",     0.85, 286, 1_000),

]