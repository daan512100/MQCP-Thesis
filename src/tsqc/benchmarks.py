# Bestand: tsqc/benchmarks.py
"""
Benchmarks framework: loop over alle .clq-instanties en opgegeven k-waarden (fixed-k)
of max-k, en sla resultaten op in een CSV. Daarnaast: predefined cases.
Print voortgang per run.
"""
import os
import glob
import csv
import time
from typing import List, Optional, Union
from pathlib import Path

from tsqc.api import solve_fixed, solve_max

# Voor timeouts
import multiprocessing
import queue

from tsqc.benchmark_cases import BENCHMARK_CASES


def run_all(
    inst_dir: Union[str, Path],
    ks: List[int],
    gamma: float,
    mode: str = "fixed",
    time_limit: Optional[float] = None,
    output: str = "benchmarks.csv",
) -> None:
    """
    Voer alle benchmarks uit volgens de parameters:
    - inst_dir: directory met .clq-bestanden
    - ks: lijst met k-waarden (alleen bij mode=='fixed')
    - gamma: dichtheidsdrempel
    - mode: 'fixed' (loop over ks) of 'max' (max-k)
    - time_limit: optionele time limit per run
    - output: pad naar output CSV-bestand

    Print voortgang per run.
    """
    pattern = os.path.join(str(inst_dir), "*.clq")
    instances = sorted(glob.glob(pattern))
    if not instances:
        print(f"Geen .clq-bestanden gevonden in {inst_dir}")
        return

    fieldnames = ["instance", "mode", "k", "gamma", "size", "edges", "density", "time", "error"]
    with open(output, "w", newline="") as csvfile:
        writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
        writer.writeheader()

        for inst in instances:
            basename = os.path.basename(inst)
            k_list = ks if mode == "fixed" else [None]
            for k in k_list:
                print(f"Running {basename} mode={mode} k={k} γ={gamma}...")
                try:
                    start = time.perf_counter()
                    if mode == "fixed":
                        sol = solve_fixed(
                            instance_path=str(inst),
                            k=k,
                            gamma=gamma,
                            time_limit=time_limit,
                        )
                    else:
                        sol = solve_max(
                            instance_path=str(inst),
                            gamma=gamma,
                            time_limit=time_limit,
                        )
                    elapsed = time.perf_counter() - start
                    writer.writerow({
                        "instance": basename,
                        "mode": mode,
                        "k": k,
                        "gamma": gamma,
                        "size": sol.size,
                        "edges": sol.edges,
                        "density": sol.density,
                        "time": f"{elapsed:.6f}",
                        "error": "",
                    })
                    print(f"  -> size={sol.size}, edges={sol.edges}, density={sol.density:.3f}, time={elapsed:.2f}s")
                except Exception as e:
                    elapsed = time.perf_counter() - start if 'start' in locals() else None
                    writer.writerow({
                        "instance": basename,
                        "mode": mode,
                        "k": k,
                        "gamma": gamma,
                        "size": None,
                        "edges": None,
                        "density": None,
                        "time": None,
                        "error": str(e),
                    })
                    print(f"  -> error: {e}")
    print(f"Benchmarks voltooid, resultaten opgeslagen in {output}")


def run_predefined(
    inst_dir: Union[str, Path],
    time_limit: Optional[float] = None,
    output: str = "benchmarks_predefined.csv",
    runs: int = 1,
    seed: int = 42,
    use_mcts: bool = False,
    mcts_budget: int = 100,
    mcts_uct: float = 1.414,
    mcts_depth: int = 5,
    lns_repair_depth: int = 10,
) -> None:
    """
    Voer de hard-gecodeerde BENCHMARK_CASES uit:
    - inst_dir: directory met .clq-bestanden
    - time_limit: optionele time limit per run (nog niet ondersteund)
    - output: pad naar output CSV-bestand
    - runs, seed, use_mcts, mcts_*, lns_repair_depth: solver-instellingen

    Print voortgang per individuele run en records best result per case.
    """
    fieldnames = ["instance", "gamma", "k", "best_size", "best_edges", "best_density", "best_time", "error"]
    with open(output, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()

        for case in BENCHMARK_CASES:
            basename = case.instance
            inst_path = Path(inst_dir) / basename
            if not inst_path.is_file():
                print(f"Instance niet gevonden: {basename}")
                writer.writerow({
                    "instance": basename,
                    "gamma": case.gamma,
                    "k": case.k,
                    "best_size": None,
                    "best_edges": None,
                    "best_density": None,
                    "best_time": None,
                    "error": "file not found",
                })
                continue

            best_sol = None
            best_time = None
            print(f"=== Case: {basename} γ={case.gamma} k={case.k} (seed={seed}, runs={runs}) ===")
            for i in range(runs):
                run_seed = seed + i
                print(f"  Run {i+1}/{runs}, seed={run_seed}...")
                start = time.perf_counter()
                try:
                    sol = solve_fixed(
                        instance_path=str(inst_path),
                        k=case.k,
                        gamma=case.gamma,
                        seed=run_seed,
                        runs=1,
                        use_mcts=use_mcts,
                        mcts_budget=mcts_budget,
                        mcts_uct=mcts_uct,
                        mcts_depth=mcts_depth,
                        lns_repair_depth=lns_repair_depth,
                    )
                    elapsed = time.perf_counter() - start
                    print(f"    size={sol.size}, edges={sol.edges}, density={sol.density:.3f}, time={elapsed:.2f}s")

                    if best_sol is None or sol.density > best_sol.density or \
                       (sol.density == best_sol.density and elapsed < best_time):
                        best_sol = sol
                        best_time = elapsed
                except Exception as e:
                    elapsed = time.perf_counter() - start
                    print(f"    Fout in run {i+1}: {e}")

            if best_sol:
                writer.writerow({
                    "instance": basename,
                    "gamma": case.gamma,
                    "k": case.k,
                    "best_size": best_sol.size,
                    "best_edges": best_sol.edges,
                    "best_density": best_sol.density,
                    "best_time": f"{best_time:.6f}",
                    "error": "",
                })
                print(f"=== Beste resultaat: size={best_sol.size}, density={best_sol.density:.3f}, time={best_time:.2f}s ===\n")
    print(f"Predefined benchmarks voltooid, resultaten opgeslagen in {output}")
