# src/tsqc/cli.py

import time
from pathlib import Path
from typing import Optional, List

import typer

from .api import solve_fixed, solve_max, parse_dimacs, SolutionData, Params
from .reporter import Reporter
from .benchmark_cases import BENCHMARK_CASES
from .grid_search import run_grid_search
from .analyzer import analyze_and_print_top_3  # --- NIEUW ---

app = typer.Typer(
    name="tsqc",
    help="Een robuuste oplosser voor het Maximum Quasi-Clique Probleem, met een Rust-core.",
    add_completion=False,
    no_args_is_help=True,
    rich_markup_mode="markdown",
)
reporter = Reporter()


@app.command()
def solve(
    instance: Path = typer.Argument(
        ...,
        help="Pad naar het DIMACS .clq-instantiebestand.",
        exists=True,
        file_okay=True,
        dir_okay=False,
        readable=True,
        show_default=False,
    ),
    gamma: float = typer.Option(
        0.9, "-g", "--gamma", help="Dichtheidsdrempel γ (een getal tussen 0 en 1)."
    ),
    k: Optional[int] = typer.Option(
        None, "-k", "--k", help="Doelgrootte voor fixed-k modus. Indien afwezig, wordt max-k modus gebruikt."
    ),
    seed: int = typer.Option(42, "-s", "--seed", help="Random seed voor de oplosser."),
    runs: int = typer.Option(1, "-r", "--runs", help="Aantal onafhankelijke runs."),
    use_mcts: bool = typer.Option(False, "--mcts", help="Schakel MCTS-LNS diversificatie in."),
    mcts_budget: int = typer.Option(100, help="MCTS: Aantal simulatieruns (budget)."),
    mcts_uct: float = typer.Option(1.414, help="MCTS: UCT exploratieconstante."),
    mcts_depth: int = typer.Option(5, help="MCTS: Maximale diepte van de zoekboom."),
    lns_repair_depth: int = typer.Option(10, help="LNS: Aantal verfijningsstappen in de herstelfase."),
    lns_rcl_alpha: float = typer.Option(1.0, "--lns-alpha", help="GRASP: Willekeur voor LNS-reconstructie (1.0=greedy)."),
    timeout_seconds: float = typer.Option(
        0.0, "--timeout", "-t",
        help="Maximale tijd in seconden voor de run. 0.0 = geen timeout."
    ),
    stagnation_iter: int = typer.Option(1_000, help="Aantal stagnatie-iteraties voor diversificatie."),
    max_iter: int = typer.Option(100_000_000, help="Maximaal aantal iteraties voor de solver."),
    tenure_u: int = typer.Option(1, help="Tabu tenure voor toevoegen (u)."),
    tenure_v: int = typer.Option(1, help="Tabu tenure voor verwijderen (v)."),
):
    """
    Lost een enkele quasi-clique instantie op (fixed-k of max-k).
    """
    solver_params = Params(
        gamma_target=gamma,
        stagnation_iter=stagnation_iter,
        max_iter=max_iter,
        tenure_u=tenure_u,
        tenure_v=tenure_v,
        use_mcts=use_mcts,
        mcts_budget=mcts_budget,
        mcts_exploration_const=mcts_uct,
        mcts_max_depth=mcts_depth,
        lns_repair_depth=lns_repair_depth,
        lns_rcl_alpha=lns_rcl_alpha,
        max_time_seconds=timeout_seconds,
        seed=seed,
        runs=runs,
        k=k,
    )

    params_for_report = {
        "instance": instance.name,
        "gamma": gamma,
        "mode": "fixed-k" if k else "max-k",
        "k": k if k else "N/A",
        "seed": seed,
        "runs": runs,
        "use_mcts": use_mcts,
        "mcts_budget": mcts_budget,
        "mcts_uct": mcts_uct,
        "mcts_depth": mcts_depth,
        "lns_repair_depth": lns_repair_depth,
        "lns_rcl_alpha": lns_rcl_alpha,
        "timeout_seconds": timeout_seconds,
        "stagnation_iter": stagnation_iter,
    }
    
    reporter.console.print(f"Starting solver for [cyan]{instance.name}[/cyan]...")
    
    try:
        if k:
            result = solve_fixed(instance, params=solver_params)
        else:
            result = solve_max(instance, params=solver_params)
        
        reporter.report_solve_result(result, params_for_report)
    except Exception as e:
        reporter.console.print(f"[bold red]Er is een fout opgetreden tijdens het oplossen: {e}[/bold red]")


@app.command(name="run-benchmarks")
def run_predefined_benchmarks(
    instances_dir: Path = typer.Argument(
        ...,
        help="Pad naar de map met de .clq benchmark-bestanden.",
        exists=True,
        file_okay=False,
        dir_okay=True,
        readable=True,
        show_default=False,
    ),
    runs: int = typer.Option(10, "-r", "--runs", help="Aantal onafhankelijke runs per benchmark-case."),
    base_seed: int = typer.Option(99, "-s", "--base-seed", help="Basis-seed voor de reeks van runs."),
    use_mcts: bool = typer.Option(False, "--mcts", help="Schakel MCTS-LNS diversificatie in voor alle runs."),
    mcts_budget: int = typer.Option(100, help="MCTS: Aantal simulatieruns (budget)."),
    mcts_uct: float = typer.Option(1.414, help="MCTS: UCT exploratieconstante."),
    mcts_depth: int = typer.Option(5, help="MCTS: Maximale diepte van de zoekboom."),
    lns_repair_depth: int = typer.Option(10, help="LNS: Aantal verfijningsstappen in de herstelfase."),
    lns_rcl_alpha: float = typer.Option(1.0, "--lns-alpha", help="GRASP: Willekeur voor LNS-reconstructie (1.0=greedy)."),
    timeout_seconds: float = typer.Option(
        0.0, "--timeout", "-t",
        help="Maximale tijd in seconden per run. 0.0 = geen timeout."
    ),
    stagnation_iter: int = typer.Option(1_000, help="Aantal stagnatie-iteraties voor diversificatie."),
    max_iter: int = typer.Option(100_000_000, help="Maximaal aantal iteraties voor de solver."),
    tenure_u: int = typer.Option(1, help="Tabu tenure voor toevoegen (u)."),
    tenure_v: int = typer.Option(1, help="Tabu tenure voor verwijderen (v)."),
):
    """
    Voert de volledige, vooraf gedefinieerde benchmark-suite uit.
    """
    overall_start_time = time.perf_counter()
    total_cases = len(BENCHMARK_CASES)
    reporter.console.print(f"[bold]Benchmark suite gestart: {total_cases} cases, {runs} runs per case.[/bold]")
    if timeout_seconds > 0:
        reporter.console.print(f"[bold yellow]Timeout per run ingesteld op: {timeout_seconds:.1f} seconden.[/bold yellow]")
    reporter.console.print("-" * 70)

    base_solver_params = Params(
        gamma_target=0.0,
        stagnation_iter=stagnation_iter,
        max_iter=max_iter,
        tenure_u=tenure_u,
        tenure_v=tenure_v,
        use_mcts=use_mcts,
        mcts_budget=mcts_budget,
        mcts_exploration_const=mcts_uct,
        mcts_max_depth=mcts_depth,
        lns_repair_depth=lns_repair_depth,
        lns_rcl_alpha=lns_rcl_alpha,
        max_time_seconds=timeout_seconds,
        seed=0,
        runs=1,
        k=None,
    )

    for i, case in enumerate(BENCHMARK_CASES):
        instance_path = instances_dir / case.instance
        
        if not instance_path.exists():
            reporter.console.print(f"[bold red]OVERGESLAGEN ({i+1}/{total_cases}):[/bold red] Bestand niet gevonden: {instance_path}")
            reporter.console.print("-" * 70)
            continue

        try:
            n, m = parse_dimacs(instance_path)
        except Exception as e:
            reporter.console.print(f"[bold red]OVERGESLAGEN ({i+1}/{total_cases}):[/bold red] Kon '{case.instance}' niet parsen: {e}")
            reporter.console.print("-" * 70)
            continue

        reporter.report_case_start(case, n, m)
        
        best_solution_for_case: Optional[SolutionData] = None
        case_start_time = time.perf_counter()

        for run_idx in range(1, runs + 1):
            run_seed = base_seed + run_idx - 1
            
            current_run_params = base_solver_params.copy()
            current_run_params.seed = run_seed
            current_run_params.gamma_target = case.gamma
            current_run_params.stagnation_iter = case.stagnation_iter
            
            try:
                if case.k is not None:
                    current_run_params.k = case.k
                    solution = solve_fixed(instance_path=instance_path, params=current_run_params)
                else:
                    current_run_params.k = None
                    solution = solve_max(instance_path=instance_path, params=current_run_params)
                
                reporter.report_run_result(run_idx, run_seed, solution, case.gamma)
                
                if best_solution_for_case is None or solution.density > best_solution_for_case.density:
                    best_solution_for_case = solution

            except Exception as e:
                reporter.report_run_error(run_idx, run_seed, str(e))

        case_total_time = time.perf_counter() - case_start_time
        reporter.report_case_summary(best_solution_for_case, case_total_time)
        reporter.console.print("-" * 70)

    overall_total_time = time.perf_counter() - overall_start_time
    reporter.console.print(f"[bold green]✨ Alle {total_cases} benchmark cases zijn voltooid in {overall_total_time:.2f} seconden. ✨[/bold green]")


@app.command(name="grid-search")
def grid_search_command(
    instances_dir: Path = typer.Argument(
        ...,
        help="Pad naar de map met de .clq instantiebestanden voor de grid search.",
        exists=True,
        file_okay=False,
        dir_okay=True,
        readable=True,
        show_default=False,
    ),
    output_file: Path = typer.Option(
        "grid_search_results.csv", "-o", "--output",
        help="Pad naar het CSV-bestand waar de resultaten worden opgeslagen.",
        show_default=True,
    ),
    runs_per_combination: int = typer.Option(
        3, "-r", "--runs-per-combo",
        help="Aantal onafhankelijke runs per parametercombinatie op elke instance.",
    ),
    base_seed: int = typer.Option(
        1000, "-s", "--base-seed",
        help="Basis random seed; wordt verhoogd voor elke run.",
    ),
    timeout_seconds: float = typer.Option(
        0.0, "--timeout", "-t",
        help="Maximale tijd in seconden per run. 0.0 = geen timeout."
    ),
    mcts_budgets: List[int] = typer.Option(
        ...,
        "--mcts-budget",
        help="Lijst van MCTS budget waarden om te testen. Meerdere waarden mogelijk.",
        show_default=False,
    ),
    mcts_exploration_consts: List[float] = typer.Option(
        [1.414],
        "--mcts-uct",
        help="Lijst van MCTS UCT exploratieconstanten om te testen. Meerdere waarden mogelijk.",
        show_default=True,
    ),
    mcts_max_depths: List[int] = typer.Option(
        ...,
        "--mcts-depth",
        help="Lijst van MCTS maximale diepte waarden om te testen. Meerdere waarden mogelijk.",
        show_default=False,
    ),
    lns_repair_depths: List[int] = typer.Option(
        ...,
        "--lns-repair-depth",
        help="Lijst van LNS repair diepte waarden om te testen. Meerdere waarden mogelijk.",
        show_default=False,
    ),
    stagnation_iters: List[int] = typer.Option(
        [1000],
        "--stagnation-iter",
        help="Lijst van stagnation iteratie waarden om te testen. Meerdere waarden mogelijk.",
        show_default=True,
    ),
    lns_rcl_alphas: List[float] = typer.Option(
        [1.0], "--lns-alpha",
        help="Lijst van LNS-reconstructie alpha-waarden om te testen. Meerdere waarden mogelijk.",
        show_default=True,
    ),
    max_iter: int = typer.Option(100_000_000, help="Maximaal aantal iteraties voor de solver."),
    tenure_u: int = typer.Option(1, help="Tabu tenure voor toevoegen (u)."),
    tenure_v: int = typer.Option(1, help="Tabu tenure voor verwijderen (v)."),
):
    """
    Voert een grid search uit om optimale MCTS-LNS parameters te vinden.
    """
    reporter.console.print("[bold]Starten van de MCTS Grid Search...[/bold]")
    reporter.console.print("Dit kan enige tijd duren, afhankelijk van het aantal combinaties en runs.")
    reporter.console.print("-" * 70)

    base_grid_params = Params(
        gamma_target=0.0,
        stagnation_iter=0,
        max_iter=max_iter,
        tenure_u=tenure_u,
        tenure_v=tenure_v,
        use_mcts=True,
        mcts_budget=0,
        mcts_exploration_const=0.0,
        mcts_max_depth=0,
        lns_repair_depth=0,
        lns_rcl_alpha=1.0,
        max_time_seconds=timeout_seconds,
        seed=0,
        runs=1,
        k=None,
    )

    run_grid_search(
        instances_dir=instances_dir,
        output_file=output_file,
        runs_per_combination=runs_per_combination,
        base_seed=base_seed,
        timeout_seconds=timeout_seconds,
        mcts_budgets=mcts_budgets,
        mcts_exploration_consts=mcts_exploration_consts,
        mcts_max_depths=mcts_max_depths,
        lns_repair_depths=lns_repair_depths,
        stagnation_iters=stagnation_iters,
        lns_rcl_alphas=lns_rcl_alphas,
        reporter=reporter,
        base_params=base_grid_params,
    )

    # --- NIEUW ---
    # Na het voltooien van de grid search, roep de analysefunctie aan.
    analyze_and_print_top_3(output_file)
    # --- EINDE NIEUW ---


if __name__ == "__main__":
    app()