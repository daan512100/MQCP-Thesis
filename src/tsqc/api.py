# src/tsqc/api.py
# DEFINITIEVE, FINALE VERSIE (na refactoring naar Params object)

import time
from pathlib import Path
from typing import NamedTuple, Union, Tuple

# We importeren nu alleen de functies die we daadwerkelijk nodig hebben uit de
# gecompileerde native module.
from tsqc._native import solve_k_py, solve_max_py, parse_dimacs_py, Params


class SolutionData(NamedTuple):
    """
    Gestructureerd resultaat van een enkele TSQC-run.
    """
    size: int
    edges: int
    density: float
    time: float
    is_timed_out: bool = False # Geeft aan of de run door een timeout is afgebroken


def parse_dimacs(instance_path: Union[str, Path]) -> Tuple[int, int]:
    """
    Parseert de header van een DIMACS .clq-bestand.
    """
    try:
        return parse_dimacs_py(str(instance_path))
    # We vangen nu de algemene `Exception` af in plaats van de
    # specifieke `PyErr`. Dit werkt omdat een fout vanuit Rust in Python
    # aankomt als een subklasse van `Exception`.
    except Exception as e:
        if "No such file or directory" in str(e):
            raise FileNotFoundError(f"Instantiebestand niet gevonden: {instance_path}") from e
        else:
            raise ValueError(f"Fout bij parsen van DIMACS-bestand '{instance_path}': {e}") from e


def solve_fixed(
    instance_path: Union[str, Path],
    params: Params, # NIEUW: Accepteer een Params object
) -> SolutionData:
    """
    Voert de fixed-k TSQC-oplosser uit.
    """
    start_time = time.perf_counter()
    try:
        # Roep de Rust-functie aan met het Params object
        size, edges, density, is_timed_out = solve_k_py(
            instance_path=str(instance_path),
            py_params=params, # Geef het Params object door
        )
        elapsed_time = time.perf_counter() - start_time
        # Inclusief is_timed_out in het geretourneerde SolutionData object
        return SolutionData(size=size, edges=edges, density=density, time=elapsed_time, is_timed_out=is_timed_out)
    except Exception as e:
        raise RuntimeError(f"Een fout is opgetreden in de native Rust-oplosser: {e}") from e


def solve_max(
    instance_path: Union[str, Path],
    params: Params, # NIEUW: Accepteer een Params object
) -> SolutionData:
    """
    Voert de max-k TSQC-oplosser uit.
    """
    start_time = time.perf_counter()
    try:
        # Roep de Rust-functie aan met het Params object
        size, edges, density, is_timed_out = solve_max_py(
            instance_path=str(instance_path),
            py_params=params, # Geef het Params object door
        )
        elapsed_time = time.perf_counter() - start_time
        # Inclusief is_timed_out in het geretourneerde SolutionData object
        return SolutionData(size=size, edges=edges, density=density, time=elapsed_time, is_timed_out=is_timed_out)
    except Exception as e:
        raise RuntimeError(f"Een fout is opgetreden in de native Rust-oplosser: {e}") from e