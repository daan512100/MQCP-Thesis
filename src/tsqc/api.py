# Bestand: src/tsqc/api.py
# DEFINITIEVE VERSIE
"""
Python-API (Application Programming Interface) voor de TSQC-native module.
...
"""

import time
from pathlib import Path
from typing import NamedTuple, Union, Optional, Tuple

# Probeer de native extensie en de types die het exporteert te importeren.
try:
    # =================================================================================
    # CORRECTIE: Gebruik een relatieve import (de punt voor _native)
    # Dit is de standaard en meest robuuste manier om een C-extensie
    # vanuit een zustermodule binnen hetzelfde package te importeren.
    # =================================================================================
    from ._native import solve_k_py, solve_max_py, parse_dimacs_py, Params
    from pyo3 import PyErr
except ImportError:
    # Maak dummy-objecten als de native module niet beschikbaar is.
    # Dit voorkomt crashes bijv. tijdens het opzetten van de dev-omgeving.
    def _dummy(*args, **kwargs):
        raise ImportError("De native Rust-extensie 'tsqc._native' is niet gecompileerd of gevonden.")
    solve_k_py = solve_max_py = parse_dimacs_py = _dummy
    
    # Maak ook een dummy 'Params' klasse en 'PyErr' voor consistentie.
    class Params:
        pass
    PyErr = Exception


class SolutionData(NamedTuple):
    """
    Gestructureerd resultaat van een enkele TSQC-run.
    ...
    """
    size: int
    edges: int
    density: float
    time: float


def parse_dimacs(instance_path: Union[str, Path]) -> Tuple[int, int]:
    """
    Parseert de header van een DIMACS .clq-bestand.
    ...
    """
    try:
        return parse_dimacs_py(str(instance_path))
    except PyErr as e:
        # Vang PyO3-specifieke fouten en zet ze om naar standaard Python-fouten.
        if "No such file or directory" in str(e):
            raise FileNotFoundError(f"Instantiebestand niet gevonden: {instance_path}") from e
        else:
            raise ValueError(f"Fout bij parsen van DIMACS-bestand '{instance_path}': {e}") from e


def solve_fixed(
    instance_path: Union[str, Path],
    k: int,
    gamma: float,
    seed: int,
    use_mcts: bool,
    runs: int = 1,
    mcts_budget: int = 100,
    mcts_uct: float = 1.414,
    mcts_depth: int = 5,
    lns_repair_depth: int = 10,
) -> SolutionData:
    """
    Voert de fixed-k TSQC-oplosser uit.
    ...
    """
    start_time = time.perf_counter()
    try:
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
        elapsed_time = time.perf_counter() - start_time
        return SolutionData(size=size, edges=edges, density=density, time=elapsed_time)
    except PyErr as e:
        # Geef een duidelijke foutmelding bij een crash in de native code.
        raise RuntimeError(f"Een fout is opgetreden in de native Rust-oplosser: {e}") from e


def solve_max(
    instance_path: Union[str, Path],
    gamma: float,
    seed: int,
    use_mcts: bool,
    runs: int = 1,
    mcts_budget: int = 100,
    mcts_uct: float = 1.414,
    mcts_depth: int = 5,
    lns_repair_depth: int = 10,
) -> SolutionData:
    """
    Voert de max-k TSQC-oplosser uit.
    ...
    """
    start_time = time.perf_counter()
    try:
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
        elapsed_time = time.perf_counter() - start_time
        return SolutionData(size=size, edges=edges, density=density, time=elapsed_time)
    except PyErr as e:
        raise RuntimeError(f"Een fout is opgetreden in de native Rust-oplosser: {e}") from e