# Bestand: tsqc/cli.py
"""
Command Line Interface voor TSQC:
- `solve`       : lost een γ-quasi-clique op (fixed-k of max-k)
- `benchmark`   : voert batch-benchmarks uit
- `gridsearch`  : voert grid-search uit voor MCTS-parameters
- `predefined`  : voert vooraf gedefinieerde benchmark-cases uit
"""
import os
import typer
from typing import Optional, List, Union
from pathlib import Path

from tsqc.api import solve_fixed, solve_max
from tsqc.config import get_benchmark_config, get_grid_config
from tsqc.benchmarks import run_all, run_predefined
from tsqc.grid_search import grid_search

app = typer.Typer(help="TSQC CLI: solve, benchmark, gridsearch en predefined benchmarks")

@app.command()
def solve(
    instance: Path = typer.Argument(..., help="Pad naar DIMACS .clq-bestand"),
    k: Optional[int] = typer.Option(
        None, "-k", "--k", help="Target clique-grootte (fixed-k). Als afwezig, gaat in max-k mode."),
    gamma: float = typer.Option(
        0.85, "-g", "--gamma", help="Dichtheidsdrempel γ (0 < γ ≤ 1)"),
    seed: int = typer.Option(
        42, "-s", "--seed", help="Random seed voor reproducibiliteit"),
    runs: int = typer.Option(
        1, "-r", "--runs", help="Aantal runs (neem de beste oplossing)"),
    threads: Optional[int] = typer.Option(
        None, "--threads", help="Aantal threads voor parallelle uitvoering (zet RAYON_NUM_THREADS)"),
    use_mcts: bool = typer.Option(
        False, "--mcts", help="Gebruik MCTS-LNS voor intensificatie"),
    mcts_budget: int = typer.Option(
        100, "--mcts-budget", help="MCTS call-budget (aantal iteraties)"),
    mcts_uct: float = typer.Option(
        1.414, "--mcts-uct", help="UCT-exploratie-parameter voor MCTS"),
    mcts_depth: int = typer.Option(
        5, "--mcts-depth", help="Diepte-limiet voor MCTS"),
    lns_repair_depth: int = typer.Option(
        10, "--lns-depth", help="Repair-diepte voor LNS na MCTS"),
):
    """
    Los een γ-quasi-clique op. Voor fixed-k geef je -k; anders max-k.
    """
    if threads is not None:
        os.environ["RAYON_NUM_THREADS"] = str(threads)

    if k is not None:
        res = solve_fixed(
            instance_path=str(instance),
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
    else:
        res = solve_max(
            instance_path=str(instance),
            gamma=gamma,
            seed=seed,
            runs=runs,
            use_mcts=use_mcts,
            mcts_budget=mcts_budget,
            mcts_uct=mcts_uct,
            mcts_depth=mcts_depth,
            lns_repair_depth=lns_repair_depth,
        )
    typer.echo(res)

@app.command()
def benchmark(
    instances: Path = typer.Option(
        ..., "-i", "--instances", help="Directory met .clq-bestanden"
    ),
    ks: List[int] = typer.Option(
        ..., "-k", "--ks", help="Lijst met target-k waarden voor fixed-k mode"
    ),
    gamma: float = typer.Option(
        0.85, "-g", "--gamma", help="Dichtheidsdrempel γ"
    ),
    mode: str = typer.Option(
        "fixed", "-m", "--mode", help="Mode: 'fixed' of 'max'"
    ),
    time_limit: Optional[float] = typer.Option(
        None, "-t", "--time-limit", help="Time limit per run (seconden)"
    ),
    output: Path = typer.Option(
        Path("benchmarks.csv"), "-o", "--output", help="CSV output pad"
    ),
):
    """
    Voer batch-benchmarks uit over alle .clq-bestanden in een map.

    Geef direct de instances-map, k-waarden, gamma, mode, time_limit en output.
    """
    run_all(
        inst_dir=str(instances),
        ks=ks,
        gamma=gamma,
        mode=mode,
        time_limit=time_limit,
        output=str(output),
    )

@app.command()
def gridsearch(
    config: Path = typer.Option(
        Path("tsqc.toml"), "-c", "--config", help="Pad naar configuratie TOML"),
    time_limit: Optional[float] = typer.Option(
        None, "-t", "--time-limit", help="Time limit per run (optioneel, niet doorgegeven aan Rust)"),
    output: Path = typer.Option(
        Path("grid_results.json"), "-o", "--output", help="Pad voor JSON output"),
):
    """
    Voer grid-search uit voor MCTS-parameters volgens de 'grid'-sectie in de TOML.
    """
    bench_cfg = get_benchmark_config(config)
    grid_cfg = get_grid_config(config)
    grid_search(
        inst_dir=bench_cfg.instances,
        gamma=bench_cfg.gamma,
        grid={"C": grid_cfg.C, "alpha": grid_cfg.alpha, "iters": grid_cfg.iters},
        time_limit=time_limit,
        output_json=str(output),
    )

@app.command()
def predefined(
    inst_dir: Path = typer.Argument(..., help="Directory met .clq-instanties"),
    time_limit: Optional[float] = typer.Option(
        None, "-t", "--time-limit", help="Time limit per run"),
    output: Path = typer.Option(
        Path("benchmarks_predefined.csv"), "-o", "--output", help="CSV output path"),
    runs: int = typer.Option(
        1, "--runs", "-r", help="Aantal runs per instance"),
    seed: int = typer.Option(
        42, "--seed", "-s", help="Random seed"),
    threads: Optional[int] = typer.Option(
        None, "--threads", help="Aantal threads (zet RAYON_NUM_THREADS)"),
    use_mcts: bool = typer.Option(
        False, "--mcts", help="Gebruik MCTS-LNS"),
    mcts_budget: int = typer.Option(
        100, "--mcts-budget", help="MCTS call-budget"),
    mcts_uct: float = typer.Option(
        1.414, "--mcts-uct", help="UCT-exploratie-parameter"),
    mcts_depth: int = typer.Option(
        5, "--mcts-depth", help="Diepte-limiet voor MCTS"),
    lns_repair_depth: int = typer.Option(
        10, "--lns-depth", help="Repair-diepte voor LNS"),
):
    """
    Voer de vooraf gedefinieerde BENCHMARK_CASES uit met de juiste γ en target-k.
    """
    if threads is not None:
        os.environ["RAYON_NUM_THREADS"] = str(threads)
    run_predefined(
        inst_dir=str(inst_dir),
        time_limit=time_limit,
        output=str(output),
        runs=runs,
        seed=seed,
        use_mcts=use_mcts,
        mcts_budget=mcts_budget,
        mcts_uct=mcts_uct,
        mcts_depth=mcts_depth,
        lns_repair_depth=lns_repair_depth,
    )

if __name__ == "__main__":
    app()
