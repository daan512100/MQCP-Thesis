# src/tsqc/reporter.py

from typing import Optional, Dict, Any

from rich.console import Console
from rich.panel import Panel
from rich.table import Table
from rich.text import Text
from rich.progress import Progress, SpinnerColumn, BarColumn, TextColumn, TimeElapsedColumn

from .api import SolutionData 
from .benchmark_cases import BenchmarkCase


class Reporter:
    """Beheert alle gestructureerde output naar de terminal."""

    def __init__(self):
        """Initialiseert een 'rich' console-object voor alle output."""
        self.console = Console(highlight=False)

    def report_combination_start(self, combination_idx: int, total_combinations: int, params: Dict[str, Any]):
        """
        Rapporteert de start van een nieuwe parametercombinatie in de grid search.
        """
        title = f"[bold magenta]Combinatie {combination_idx}/{total_combinations}[/bold magenta]"
        subtitle_parts = [
            f"Budget={params['mcts_budget']}",
            f"Exploration={params['mcts_exploration_const']:.3f}",
            f"MaxDepth={params['mcts_max_depth']}",
            f"RepairDepth={params['lns_repair_depth']}"
        ]
        
        if 'stagnation_iter' in params:
            subtitle_parts.append(f"Stagnation={params['stagnation_iter']}")

        # --- NIEUW ---
        # Voeg de nieuwe alpha-parameter toe aan de header voor de duidelijkheid.
        if 'lns_rcl_alpha' in params:
            subtitle_parts.append(f"RCL-α={params['lns_rcl_alpha']:.2f}")
        # --- EINDE NIEUW ---

        subtitle = " | ".join(subtitle_parts)

        self.console.print(
            Panel(Text(subtitle, justify="center"), title=title, border_style="magenta", expand=False),
            justify="center"
        )
        self.console.print("-" * 70)


    def report_case_start(self, case: BenchmarkCase, n: int, m: int):
        """
        Rapporteert de start van een nieuwe benchmark-case met een duidelijke header.
        """
        title = f"[bold cyan]Case: {case.instance}[/bold cyan]"
        subtitle = (f"γ={case.gamma:.3f} | Target k={case.k} | "
                    f"Graaf (n={n}, m={m})")
        
        panel_content = Text(subtitle, justify="center")
        self.console.print(
            Panel(panel_content, title=title, border_style="cyan", expand=False),
            justify="center"
        )

    def report_run_result(
        self,
        run_idx: int,
        seed: int,
        sol: SolutionData,
        gamma: float
    ):
        """
        Presenteert het resultaat van een enkele, voltooide run met verbeterde statuslogica.
        """
        is_feasible = sol.density + 1e-9 >= gamma
        
        status_text = ""
        status_color = "yellow"

        if is_feasible:
            status_text = "Feasible"
            status_color = "green"
            if sol.is_timed_out:
                status_text += " (Timed Out)"
        elif sol.is_timed_out:
            status_text = "Timed Out"
            status_color = "orange3"
        else:
            status_text = "Infeasible"
            status_color = "yellow"

        line = (
            f"  ├─ Run {run_idx:<2} (Seed: {seed:<4}) │ "
            f"Size: {sol.size:<4} │ Edges: {sol.edges:<6} │ "
            f"Density: [bold]{sol.density:.5f}[/bold] │ "
            f"Time: {sol.time:.2f}s │ "
            f"Status: [{status_color}]{status_text}[/{status_color}]"
        )
        self.console.print(line, markup=True)

    def report_run_error(self, run_idx: int, seed: int, error_msg: str):
        """Rapporteert een fout die is opgetreden tijdens een specifieke run."""
        self.console.print(
            f"  ├─ Run {run_idx} (seed={seed}) [bold red]FAILED[/bold red]: {error_msg}",
            markup=True
        )

    def report_case_summary(
        self,
        best_sol: Optional[SolutionData],
        total_time: float
    ):
        """
        Rapporteert een samenvatting met het beste resultaat voor de afgeronde case.
        """
        if best_sol:
            summary_text = Text.assemble(
                "Beste Grootte: ", Text(str(best_sol.size), style="bold green"),
                " | Beste Dichtheid: ", Text(f"{best_sol.density:.5f}", style="bold green"),
                f"\nTotale Tijd: {total_time:.2f}s"
            )
            border_style = "green"
        else:
            summary_text = Text("[bold red]Geen werkende oplossing gevonden in alle runs.[/bold red]", markup=True)
            border_style = "red"

        self.console.print(
            Panel(summary_text,
                  title="[bold]Case Samenvatting[/bold]",
                  border_style=border_style,
                  padding=(1, 2)),
            justify="center"
        )
        
    def report_solve_result(self, sol: SolutionData, params: Dict[str, Any]):
        """Presenteert het resultaat van een enkele 'solve' aanroep in een overzichtelijke tabel."""
        table = Table(title="[bold]TSQC Oplosser Resultaat[/bold]", show_header=True, header_style="bold magenta",
                      caption_justify="center")

        table.add_column("Parameter", style="cyan", no_wrap=True)
        table.add_column("Waarde")

        for key, value in params.items():
            table.add_row(Text(str(key).replace("_", " ").title()), Text(str(value)))
        
        table.add_section()
        table.add_row(Text("Grootte", style="bold"), Text(str(sol.size), style="bold green"))
        table.add_row(Text("Kanten", style="bold"), Text(str(sol.edges), style="bold green"))
        table.add_row(Text("Dichtheid", style="bold"), Text(f"{sol.density:.5f}", style="bold green"))
        table.add_row(Text("Tijd (s)", style="bold"), Text(f"{sol.time:.3f}"))
        table.add_row(Text("Timed Out", style="bold"), Text("Ja" if sol.is_timed_out else "Nee"))
        
        self.console.print(table)
        
    def create_progress_bar(self) -> Progress:
        """Creëert en configureert een 'rich' progressiebalk voor het volgen van de runs."""
        return Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            BarColumn(),
            TextColumn("[progress.completed]/[progress.total]"),
            TextColumn("[progress.percentage]{task.percentage:>3.0f}%"),
            TimeElapsedColumn(),
            console=self.console,
            transient=True
        )