# Bestand: tsqc/config.py
"""
Configuratielaag voor benchmarks en grid search.
Leest defaults en TOML-configuratie voor `benchmarks` en `grid` secties.
"""

import os
from pathlib import Path
from typing import Union, List, Optional, NamedTuple, Dict

try:
    import tomllib  # Python 3.11+
except ImportError:
    import toml as tomllib  # pip install toml


class BenchmarkConfig(NamedTuple):
    instances: str
    ks: List[int]
    gamma: float
    mode: str
    time_limit: Optional[float]
    output: str


class GridConfig(NamedTuple):
    C: List[float]
    alpha: List[float]
    iters: List[int]


def _load_toml(path: Union[str, Path]) -> Dict:
    """
    Laadt een TOML-bestand en retourneert de dict-weergave.
    Als het bestand niet bestaat of niet leesbaar is, wordt een lege dict geretourneerd.
    """
    p = Path(path)
    if not p.is_file():
        return {}
    try:
        with p.open("rb") as f:
            return tomllib.load(f)
    except Exception:
        return {}


def get_benchmark_config(
    config_path: Union[str, Path] = "tsqc.toml",
    default_instances: str = "./benchmarks",
    default_ks: List[int] = [5, 10, 15],
    default_gamma: float = 0.85,
    default_mode: str = "fixed",
    default_time_limit: Optional[float] = None,
    default_output: str = "benchmarks.csv",
) -> BenchmarkConfig:
    """
    Leest de `benchmarks` sectie uit een TOML-configuratie.

    Voorbeeld `tsqc.toml`:
    [benchmarks]
    instances = "path/to/instances"
    ks = [5, 10, 15]
    gamma = 0.9
    mode = "fixed"
    time_limit = 30.0
    output = "out.csv"
    """
    cfg = _load_toml(config_path).get("benchmarks", {})
    return BenchmarkConfig(
        instances=cfg.get("instances", default_instances),
        ks=cfg.get("ks", default_ks),
        gamma=cfg.get("gamma", default_gamma),
        mode=cfg.get("mode", default_mode),
        time_limit=cfg.get("time_limit", default_time_limit),
        output=cfg.get("output", default_output),
    )


def get_grid_config(
    config_path: Union[str, Path] = "tsqc.toml",
    default_C: List[float] = [1.0],
    default_alpha: List[float] = [1.0],
    default_iters: List[int] = [100],
) -> GridConfig:
    """
    Leest de `grid` sectie uit een TOML-configuratie.

    Voorbeeld `tsqc.toml`:
    [grid]
    C = [0.5, 1.0, 2.0]
    alpha = [0.1, 0.5, 1.0]
    iters = [100, 500, 1000]
    """
    cfg = _load_toml(config_path).get("grid", {})
    return GridConfig(
        C=cfg.get("C", default_C),
        alpha=cfg.get("alpha", default_alpha),
        iters=cfg.get("iters", default_iters),
    )
