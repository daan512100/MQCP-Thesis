# Bestand: tsqc/api.py
"""
Python-API wrappers voor de TSQC-native module (PyO3-extensie).
Biedt:
- `parse_dimacs`: DIMACS-header parser
- `solve_fixed`: fixed-k TSQC (optioneel met MCTS-LNS)
- `solve_max`: max-k TSQC (optioneel met MCTS-LNS)

LET OP: de `time_limit` parameter wordt momenteel genegeerd omdat de
native implementatie dit (nog) niet ondersteunt.
"""
import time
from pathlib import Path
from typing import NamedTuple, Union, Optional

from tsqc._native import solve_k_py, solve_max_py, parse_dimacs_py  # PyO3-extensie


class SolutionData(NamedTuple):
    """Resultaat van één TSQC-run."""
    size: int
    edges: int
    density: float
    time: float  # looptijd in seconden


def parse_dimacs(instance_path: Union[str, Path]) -> (int, int):
    """
    Parseert de header van een DIMACS .clq-bestand en retourneert (n, m).
    """
    return parse_dimacs_py(str(instance_path))


def solve_fixed(
    instance_path: Union[str, Path],
    k: int,
    gamma: float,
    time_limit: Optional[float] = None,
    seed: int = 0,
    runs: int = 1,
    use_mcts: bool = False,
    mcts_budget: int = 100,
    mcts_uct: float = 1.414,
    mcts_depth: int = 5,
    lns_repair_depth: int = 10,
) -> SolutionData:
    """
    Voer fixed-k TSQC uit. Parameters:
    - instance_path: .clq-bestand
    - k: target clique-grootte
    - gamma: dichtheidsdrempel γ
    - time_limit: genegeerd (niet ondersteund natively)
    - seed, runs: RNG en aantal herhalingen
    - use_mcts + mcts_*: MCTS-LNS configuratie
    - lns_repair_depth: LNS repair-diepte

    Retourneert SolutionData(size, edges, density, time).
    """
    start = time.perf_counter()
    size, edges, density = solve_k_py(
        instance_path=str(instance_path),
        k=k,
        gamma=gamma,
        seed=seed,
        runs=runs,
        use_mcts=use_mcts,
        mcts_budget=mcts_budget,
        mcts_uct=mcts_uct,
        mcts_depth=mcts_depth,
        lns_repair_depth=lns_repair_depth,
    )
    elapsed = time.perf_counter() - start
    return SolutionData(size=size, edges=edges, density=density, time=elapsed)


def solve_max(
    instance_path: Union[str, Path],
    gamma: float,
    time_limit: Optional[float] = None,
    seed: int = 0,
    runs: int = 1,
    use_mcts: bool = False,
    mcts_budget: int = 100,
    mcts_uct: float = 1.414,
    mcts_depth: int = 5,
    lns_repair_depth: int = 10,
) -> SolutionData:
    """
    Voer max-k TSQC uit. Parameters:
    - instance_path: .clq-bestand
    - gamma: dichtheidsdrempel γ
    - time_limit: genegeerd (niet ondersteund natively)
    - seed, runs: RNG en aantal herhalingen
    - use_mcts + mcts_*: MCTS-LNS configuratie
    - lns_repair_depth: LNS repair-diepte

    Retourneert SolutionData(size, edges, density, time).
    """
    start = time.perf_counter()
    size, edges, density = solve_max_py(
        instance_path=str(instance_path),
        gamma=gamma,
        seed=seed,
        runs=runs,
        use_mcts=use_mcts,
        mcts_budget=mcts_budget,
        mcts_uct=mcts_uct,
        mcts_depth=mcts_depth,
        lns_repair_depth=lns_repair_depth,
    )
    elapsed = time.perf_counter() - start
    return SolutionData(size=size, edges=edges, density=density, time=elapsed)
