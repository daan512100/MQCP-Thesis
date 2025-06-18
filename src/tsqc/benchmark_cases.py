# Bestand: tsqc/benchmark_cases.py
"""
Gedefinieerde benchmarks uit de oude project­configuratie:
Elke case specificeert een DIMACS-bestand, een γ-waarde en een target-k.
"""
from pathlib import Path
from typing import NamedTuple, List


class BenchmarkCase(NamedTuple):
    """Eén benchmark-run: instantie, γ-waarde en target-k."""
    instance: str  # bestandsnaam van .clq
    gamma: float   # dichtheidsdrempel
    k: int         # target quasi-clique grootte


BENCHMARK_CASES: List[BenchmarkCase] = [
    # γ = 0.85
    BenchmarkCase("p_hat300-1.clq",  0.85,  12),
    BenchmarkCase("p_hat300-2.clq",  0.85,  85),
    BenchmarkCase("brock200_2.clq",  0.85,  19),
    BenchmarkCase("hamming8-4.clq",  0.85,  35),
    BenchmarkCase("keller4.clq",     0.85,  31),
    BenchmarkCase("brock200_4.clq",  0.85,  39),
    BenchmarkCase("p_hat300-3.clq",  0.85, 180),
    BenchmarkCase("brock400_2.clq",  0.85, 100),
    BenchmarkCase("brock400_4.clq",  0.85, 102),
    BenchmarkCase("p_hat700-1.clq",  0.85,  19),
    BenchmarkCase("p_hat700-2.clq",  0.85, 223),
    BenchmarkCase("p_hat700-3.clq",  0.85, 430),
    BenchmarkCase("p_hat1500-2.clq", 0.85, 487),
    BenchmarkCase("p_hat1500-3.clq", 0.85, 943),
    BenchmarkCase("keller5.clq",     0.85, 286),
    # γ = 0.95
    BenchmarkCase("p_hat300-1.clq",  0.95,   9),
    BenchmarkCase("p_hat300-2.clq",  0.95,  41),
    BenchmarkCase("brock200_2.clq",  0.95,  13),
    BenchmarkCase("hamming8-4.clq",  0.95,  17),
    BenchmarkCase("keller4.clq",     0.95,  15),
    BenchmarkCase("brock200_4.clq",  0.95,  21),
    BenchmarkCase("p_hat300-3.clq",  0.95,  71),
    BenchmarkCase("brock400_2.clq",  0.95,  40),
    BenchmarkCase("brock400_4.clq",  0.95,  39),
    BenchmarkCase("p_hat700-1.clq",  0.95,  13),
    BenchmarkCase("p_hat700-2.clq",  0.95,  96),
    BenchmarkCase("p_hat700-3.clq",  0.95, 176),
    BenchmarkCase("p_hat1500-2.clq", 0.95, 193),
    BenchmarkCase("p_hat1500-3.clq", 0.95, 351),
    BenchmarkCase("keller5.clq",     0.95,  47),
    # γ = 1.00
    BenchmarkCase("p_hat300-1.clq",  1.00,   8),
    BenchmarkCase("p_hat300-2.clq",  1.00,  25),
    BenchmarkCase("brock200_2.clq",  1.00,  12),
    BenchmarkCase("hamming8-4.clq",  1.00,  16),
    BenchmarkCase("keller4.clq",     1.00,  11),
    BenchmarkCase("brock200_4.clq",  1.00,  17),
    BenchmarkCase("p_hat300-3.clq",  1.00,  36),
    BenchmarkCase("brock400_2.clq",  1.00,  29),
    BenchmarkCase("brock400_4.clq",  1.00,  33),
    BenchmarkCase("p_hat700-1.clq",  1.00,  11),
    BenchmarkCase("p_hat700-2.clq",  1.00,  44),
    BenchmarkCase("p_hat700-3.clq",  1.00,  62),
    BenchmarkCase("p_hat1500-2.clq", 1.00,  65),
    BenchmarkCase("p_hat1500-3.clq", 1.00,  94),
    BenchmarkCase("keller5.clq",     1.00,  27),
]