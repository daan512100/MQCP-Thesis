# src/tsqc/cli.py
# VOLLEDIG HERSCHREVEN
"""
Command Line Interface voor de TSQC Oplosser.
Dit bestand is het centrale toegangspunt voor alle acties. Het gebruikt Typer
voor een robuuste en gebruiksvriendelijke interface en de Reporter-klasse voor
rijke, informatieve output.
Beschikbare commando's:
- `solve`: Lost een enkele instantie op met gespecificeerde parameters.
- `run-benchmarks`: Voert de vooraf gedefinieerde set van benchmarks uit
  (uit benchmark_cases.py) en rapporteert de voortgang en resultaten live.
"""
import time
from pathlib import Path
from typing import Optional

import typer

# Importeer de kerncomponenten van ons Python-pakket.
# Deze imports verwijzen naar de (straks verbeterde) API-laag en de nieuwe reporter.
from tsqc.api import solve_fixed, solve_max, parse_dimacs, SolutionData
from tsqc.reporter import Reporter
from tsqc.benchmark_cases import BENCHMARK_CASES, BenchmarkCase

# Initialiseer de Typer-app voor het definiëren van commando's
# en de Reporter-klasse voor alle output.
app = typer.Typer(
    name="tsqc",
    help="Een robuuste oplosser voor het Maximum Quasi-Clique Probleem, met een Rust-core.",
    add_completion=False,
    no_args_is_help=True,
    rich_markup_mode="markdown", # Activeer rijke opmaak voor help-teksten
)
reporter = Reporter()


@app.command()
def solve(
    instance: Path = typer.Argument(
        ...,  # '...' betekent dat dit argument verplicht is.
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
    use_mcts: bool = typer.Option(False, "--mcts", help="Schakel MCTS-LNS diversificatie in."),
    # NIEUW: Timeout parameter voor de CLI 'solve' opdracht
    timeout_seconds: float = typer.Option(
        0.0, "--timeout", "-t", 
        help="Maximale tijd in seconden voor de run. 0.0 = geen timeout."
    ),
):
    """
    Lost een enkele quasi-clique instantie op (fixed-k of max-k).
    """
    # Bundel de parameters voor een nette rapportage via de reporter.
    params = {"instance": instance.name, "gamma": gamma, "seed": seed, "use_mcts": use_mcts}
    if timeout_seconds > 0:
        params["timeout_seconds"] = timeout_seconds
    
    reporter.console.print(f"Starting solver for [cyan]{instance.name}[/cyan]...")
    
    try:
        if k:
            # Als k is meegegeven, draai in fixed-k modus.
            params["mode"] = "fixed-k"
            params["k"] = k
            result = solve_fixed(
                instance, 
                k=k, 
                gamma=gamma, 
                seed=seed, 
                use_mcts=use_mcts,
                max_time_seconds=timeout_seconds # Geef timeout door
            )
        else:
            # Anders, draai in max-k modus.
            params["mode"] = "max-k"
            result = solve_max(
                instance, 
                gamma=gamma, 
                seed=seed, 
                use_mcts=use_mcts,
                max_time_seconds=timeout_seconds # Geef timeout door
            )
        
        # Laat de reporter het resultaat tonen in een nette tabel.
        reporter.report_solve_result(result, params)
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
    seed: int = typer.Option(99, "-s", "--seed", help="Basis-seed voor de reeks van runs."),
    use_mcts: bool = typer.Option(False, "--mcts", help="Schakel MCTS-LNS diversificatie in voor alle runs."),
    mcts_budget: int = typer.Option(100, help="MCTS: Aantal simulatieruns (budget)."),
    mcts_uct: float = typer.Option(1.414, help="MCTS: UCT exploratieconstante."),
    mcts_depth: int = typer.Option(5, help="MCTS: Maximale diepte van de zoekboom."),
    lns_repair_depth: int = typer.Option(10, help="LNS: Aantal verfijningsstappen in de herstelfase."),
    # NIEUW: Timeout parameter voor de CLI benchmark opdracht
    timeout_seconds: float = typer.Option(
        0.0, "--timeout", "-t", 
        help="Maximale tijd in seconden per run. 0.0 = geen timeout."
    ),
):
    """
    Voert de volledige, vooraf gedefinieerde benchmark-suite uit.
    Dit commando doorloopt alle cases gedefinieerd in `benchmark_cases.py`,
    voert voor elke case meerdere runs uit met unieke seeds, en rapporteert
    de voortgang en resultaten live in de terminal.
    """
    overall_start_time = time.perf_counter()
    total_cases = len(BENCHMARK_CASES)
    reporter.console.print(f"[bold]Benchmark suite gestart: {total_cases} cases, {runs} runs per case.[/bold]")
    if timeout_seconds > 0:
        reporter.console.print(f"[bold yellow]Timeout per run ingesteld op: {timeout_seconds:.1f} seconden.[/bold yellow]")
    reporter.console.print("-" * 70)

    for i, case in enumerate(BENCHMARK_CASES):
        instance_path = instances_dir / case.instance
        
        # --- Validatie van de benchmark case ---
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

        # --- Uitvoering van de runs voor deze case ---
        reporter.report_case_start(case, n, m)
        
        best_solution_for_case: Optional[SolutionData] = None
        case_start_time = time.perf_counter()

        # Gebruik de progressiebalk van de reporter voor live feedback per run.
        progress_ctx = reporter.create_progress_bar()
        with progress_ctx as progress:
            task = progress.add_task(f"[cyan]Runs voor {case.instance}", total=runs)

            for run_idx in range(1, runs + 1):
                # Elke run krijgt een unieke, reproduceerbare seed.
                run_seed = seed + run_idx - 1
                progress.update(task, description=f"[cyan]Run {run_idx}/{runs} (seed={run_seed})")
                
                try:
                    # Roep de API aan voor een ENKELE run. De CLI-loop beheert het totaal.
                    solution = solve_fixed(
                        instance_path=instance_path,
                        k=case.k,
                        gamma=case.gamma,
                        seed=run_seed,
                        use_mcts=use_mcts,
                        runs=1,  # Belangrijk: de Rust-laag doet 1 run, de Python-lus herhaalt.
                        mcts_budget=mcts_budget,
                        mcts_uct=mcts_uct,
                        mcts_depth=mcts_depth,
                        lns_repair_depth=lns_repair_depth,
                        max_time_seconds=timeout_seconds, # Geef timeout door
                    )
                    
                    # Rapporteer het resultaat van deze specifieke run.
                    reporter.report_run_result(run_idx, run_seed, solution, case.gamma)
                   
                    # Update de beste oplossing voor deze case (hoogste dichtheid telt).
                    if best_solution_for_case is None or solution.density > best_solution_for_case.density:
                        best_solution_for_case = solution

                except Exception as e:
                    reporter.report_run_error(run_idx, run_seed, str(e))
                
                progress.advance(task)

        # --- Rapportage van de samenvatting voor deze case ---
        case_total_time = time.perf_counter() - case_start_time
        reporter.report_case_summary(best_solution_for_case, case_total_time)
        reporter.console.print("-" * 70)

    overall_total_time = time.perf_counter() - overall_start_time
    reporter.console.print(f"[bold green]✨ Alle {total_cases} benchmark cases zijn voltooid in {overall_total_time:.2f} seconden. ✨[/bold green]")


if __name__ == "__main__":
    app()