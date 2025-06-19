# src/tsqc/grid_search.py
import csv
from itertools import product
from pathlib import Path
from typing import List, Optional, Dict, Any
import time

from .api import parse_dimacs, solve_fixed, solve_max, Params, SolutionData
from .benchmark_cases import BENCHMARK_CASES
from .reporter import Reporter


def run_grid_search(
    instances_dir: Path,
    output_file: Path,
    runs_per_combination: int,
    base_seed: int,
    timeout_seconds: Optional[float],
    mcts_budgets: List[int],
    mcts_exploration_consts: List[float],
    mcts_max_depths: List[int],
    lns_repair_depths: List[int],
    stagnation_iters: List[int],
    lns_rcl_alphas: List[float],
    reporter: Reporter,
    base_params: Params,
):
    """
    Voert een grid search uit over de opgegeven parameters.
    """
    param_combinations = list(product(
        mcts_budgets,
        mcts_exploration_consts,
        mcts_max_depths,
        lns_repair_depths,
        stagnation_iters,
        lns_rcl_alphas,
    ))

    reporter.console.print(f"[bold blue]Starting Grid Search with {len(param_combinations)} combinations.[/bold blue]")
    reporter.console.print(f"Results will be saved to [bold green]{output_file}[/bold green]")
    if timeout_seconds is not None and timeout_seconds > 0:
        reporter.console.print(f"[bold yellow]Timeout per run ingesteld op: {timeout_seconds:.1f} seconden.[/bold yellow]")


    # --- AANGEPAST ---
    # Voeg 'combination_id' toe aan de header van het CSV-bestand.
    fieldnames = [
        "combination_id", "instance_name", "gamma", "k",
        "mcts_budget", "mcts_exploration_const", "mcts_max_depth", "lns_repair_depth",
        "stagnation_iter", "lns_rcl_alpha",
        "avg_solution_size", "max_solution_size",
        "avg_solution_density", "max_solution_density",
        "avg_run_time_seconds", "min_run_time_seconds", "max_run_time_seconds",
        "timeout_count", "total_runs"
    ]
    # --- EINDE AANPASSING ---

    with open(output_file, 'w', newline='', encoding='utf-8') as csvfile:
        writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
        writer.writeheader()

        for i, (mb, me, md, lr, si, ra) in enumerate(param_combinations):
            current_params = base_params.copy()
            
            current_params.use_mcts = True
            current_params.mcts_budget = mb
            current_params.mcts_exploration_const = me
            current_params.mcts_max_depth = md
            current_params.lns_repair_depth = lr
            current_params.max_time_seconds = timeout_seconds if timeout_seconds is not None else 0.0
            current_params.stagnation_iter = si
            current_params.lns_rcl_alpha = ra

            reporter.report_combination_start(i + 1, len(param_combinations), {
                "mcts_budget": mb,
                "mcts_exploration_const": me,
                "mcts_max_depth": md,
                "lns_repair_depth": lr,
                "stagnation_iter": si,
                "lns_rcl_alpha": ra,
            })


            for case_idx, case in enumerate(BENCHMARK_CASES):
                instance_path = instances_dir / case.instance

                if not instance_path.exists():
                    reporter.report_run_error(0, 0, f"Instance file not found: {instance_path}")
                    reporter.console.print(f"[bold red]OVERGESLAGEN ({case_idx+1}/{len(BENCHMARK_CASES)}):[/bold red] Bestand niet gevonden: {instance_path}")
                    continue

                try:
                    n, m = parse_dimacs(instance_path)
                    reporter.report_case_start(case, n, m)
                except ValueError as e:
                    reporter.report_run_error(0, 0, f"Error parsing DIMACS file {instance_path}: {e}")
                    reporter.console.print(f"[bold red]OVERGESLAGEN ({case_idx+1}/{len(BENCHMARK_CASES)}):[/bold red] Fout bij parsen van DIMACS-bestand '{instance_path}': {e}")
                    continue

                results_for_this_case_combo: List[SolutionData] = []
                timeout_count = 0

                for run_idx in range(runs_per_combination):
                    current_seed = base_seed + run_idx
                    
                    run_params = current_params.copy()
                    run_params.seed = current_seed
                    run_params.gamma_target = case.gamma

                    if case.k is not None:
                        run_params.k = case.k
                        result: SolutionData = solve_fixed(instance_path, run_params)
                    else:
                        run_params.k = None 
                        result: SolutionData = solve_max(instance_path, run_params)

                    results_for_this_case_combo.append(result)
                    if result.is_timed_out:
                        timeout_count += 1

                    reporter.report_run_result(run_idx + 1, current_seed, result, case.gamma)
                
                if results_for_this_case_combo:
                    sizes = [r.size for r in results_for_this_case_combo]
                    densities = [r.density for r in results_for_this_case_combo]
                    times = [r.time for r in results_for_this_case_combo]

                    avg_size = sum(sizes) / len(sizes)
                    max_size = max(sizes)
                    avg_density = sum(densities) / len(densities)
                    max_density = max(densities)
                    avg_time = sum(times) / len(times)
                    min_time = min(times)
                    max_time = max(times)

                    # --- AANGEPAST ---
                    # Voeg de 'combination_id' toe aan elke rij die we wegschrijven.
                    writer.writerow({
                        "combination_id": i + 1,
                        "instance_name": case.instance,
                        "gamma": case.gamma,
                        "k": case.k,
                        "mcts_budget": current_params.mcts_budget,
                        "mcts_exploration_const": current_params.mcts_exploration_const,
                        "mcts_max_depth": current_params.mcts_max_depth,
                        "lns_repair_depth": current_params.lns_repair_depth,
                        "stagnation_iter": current_params.stagnation_iter,
                        "lns_rcl_alpha": current_params.lns_rcl_alpha,
                        "avg_solution_size": f"{avg_size:.2f}",
                        "max_solution_size": max_size,
                        "avg_solution_density": f"{avg_density:.4f}",
                        "max_solution_density": f"{max_density:.4f}",
                        "avg_run_time_seconds": f"{avg_time:.2f}",
                        "min_run_time_seconds": f"{min_time:.2f}",
                        "max_run_time_seconds": f"{max_time:.2f}",
                        "timeout_count": timeout_count,
                        "total_runs": runs_per_combination
                    })
                    # --- EINDE AANPASSING ---
                else:
                    reporter.console.print(f"[yellow]No runs completed for {case.instance} with current parameters due to earlier errors.[/yellow]")
                reporter.console.print("-" * 70)

    reporter.console.print("[bold green]Grid search completed! All results saved to the specified CSV file.[/bold green]")