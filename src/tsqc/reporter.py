# src/tsqc/reporter.py
# NIEUW BESTAND (met aanpassingen voor grid search rapportage)
"""
Deze module bevat de `Reporter`-klasse, die verantwoordelijk is voor alle
terminal-output. Het gebruikt de 'rich' bibliotheek voor visueel aantrekkelijke
en gestructureerde feedback, zoals panelen, tabellen en gekleurde tekst.
Dit centraliseert alle print-logica op één plek, waardoor de CLI-code
(in cli.py) schoner wordt en zich kan focussen op de uitvoeringslogica.
"""

from typing import Optional, Dict, Any, NamedTuple

# Rich is een krachtige bibliotheek voor mooie terminal-output.
from rich.console import Console
from rich.panel import Panel
from rich.table import Table
from rich.text import Text
from rich.progress import Progress, SpinnerColumn, BarColumn, TextColumn, TimeElapsedColumn

# Importeer de datastructuren.
from tsqc.api import SolutionData 
from tsqc.benchmark_cases import BenchmarkCase # BenchmarkCase is hier nodig voor type hints


class MCTSGridParams(NamedTuple):
    """
    NamedTuple om een set MCTS parameters voor een grid search combinatie vast te houden.
    Verplaatst van grid_search.py om circulaire imports te voorkomen.
    Wordt nu voornamelijk intern door de Reporter gebruikt voor type-hints,
    of voor het geval dat een Dict niet flexibel genoeg is, maar we zullen Dict gebruiken.
    """
    mcts_budget: int
    mcts_exploration_const: float
    mcts_max_depth: int
    lns_repair_depth: int
    # stagnation_iter: int # We gebruiken een Dict[str, Any] voor flexibiliteit, dus deze is niet nodig hier.


class Reporter:
    """Beheert alle gestructureerde output naar de terminal."""

    def __init__(self):
        """Initialiseert een 'rich' console-object voor alle output."""
        self.console = Console(highlight=False)

    def report_combination_start(self, combination_idx: int, total_combinations: int, params: Dict[str, Any]): # AANGEPAST: params is nu Dict[str, Any]
        """
        Rapporteert de start van een nieuwe parametercombinatie in de grid search.
        Args:
            combination_idx (int): De index van de huidige combinatie.
            total_combinations (int): Het totale aantal combinaties.
            params (Dict[str, Any]): Een dictionary met de MCTS en andere getunede parameters voor deze combinatie.
        """
        title = f"[bold magenta]Combinatie {combination_idx}/{total_combinations}[/bold magenta]"
        subtitle_parts = [
            f"Budget={params['mcts_budget']}",
            f"Exploration={params['mcts_exploration_const']:.3f}",
            f"MaxDepth={params['mcts_max_depth']}",
            f"RepairDepth={params['lns_repair_depth']}"
        ]
        # Voeg stagnation_iter alleen toe als deze aanwezig is in de params dict
        if 'stagnation_iter' in params:
            subtitle_parts.append(f"Stagnation={params['stagnation_iter']}")

        subtitle = " | ".join(subtitle_parts)

        self.console.print(
            Panel(Text(subtitle, justify="center"), title=title, border_style="magenta", expand=False),
            justify="center"
        )
        self.console.print("-" * 70)


    def report_case_start(self, case: BenchmarkCase, n: int, m: int):
        """
        Rapporteert de start van een nieuwe benchmark-case met een duidelijke header.
        Args:
            case: Het BenchmarkCase-object dat wordt uitgevoerd.
            n: Het aantal knopen (vertices) in de graaf.
            m: Het aantal kanten (edges) in de graaf.
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
        Presenteert het resultaat van een enkele, voltooide run.
        Args:
            run_idx: Het indexnummer van de run (bv. 1, 2, 3...).
            seed: De random seed die voor deze specifieke run is gebruikt.
            sol: Het SolutionData-object met de resultaten van de solver.
            gamma: De gamma-drempel die gold, om de haalbaarheid correct te beoordelen.
        """
        is_feasible = sol.density + 1e-9 >= gamma
        
        status_text = ""
        status_color = "yellow"

        if sol.is_timed_out:
            status_text = "Timed Out"
            status_color = "orange3"
        elif is_feasible:
            status_text = "Feasible"
            status_color = "green"
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
        Args:
            best_sol: De beste SolutionData die in alle runs voor deze case is gevonden.
            Kan 'None' zijn als alle runs faalden.
            total_time: De totale tijd die alle runs voor deze case hebben gekost.
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
            TextColumn("[progress.completed]/[progress.total]"), # Toont nu ook absoluut aantal
            TextColumn("[progress.percentage]{task.percentage:>3.0f}%"),
            TimeElapsedColumn(),
            console=self.console,
            transient=True  # Laat de progressiebalk verdwijnen na afronding
        )