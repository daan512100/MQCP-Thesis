# Bestand: src/tsqc/reporter.py
# DEFINITIEVE, VERBETERDE VERSIE
"""
Deze module bevat de `Reporter`-klasse, die verantwoordelijk is voor alle
terminal-output. Het gebruikt de 'rich' bibliotheek voor visueel aantrekkelijke
en gestructureerde feedback, zoals panelen, tabellen en gekleurde tekst.

Dit centraliseert alle print-logica op één plek, waardoor de CLI-code
(in cli.py) schoner wordt en zich kan focussen op de uitvoeringslogica.
"""

from typing import Optional, Dict

# Rich is een krachtige bibliotheek voor mooie terminal-output.
from rich.console import Console
from rich.panel import Panel
from rich.table import Table
from rich.text import Text
from rich.progress import Progress, SpinnerColumn, BarColumn, TextColumn, TimeElapsedColumn

# =================================================================================
# CORRECTIE: De import is gewijzigd van `from tsqc.api import ...`
# naar `from tsqc import ...` om de publieke API van het package te gebruiken.
# Dit is consistent met de aanpassingen in `cli.py` en verhoogt de robuustheid.
# =================================================================================
from tsqc import SolutionData
from tsqc.benchmark_cases import BenchmarkCase


class Reporter:
    """Beheert alle gestructureerde output naar de terminal."""

    def __init__(self):
        """Initialiseert een 'rich' console-object voor alle output."""
        self.console = Console(highlight=False)

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
                    f"Graph (n={n}, m={m})")
        
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
        # Bepaal de status (haalbaar of niet) op basis van de gamma-drempel.
        # Een kleine epsilon (1e-9) wordt gebruikt om floating-point onnauwkeurigheden te ondervangen.
        is_feasible = sol.density + 1e-9 >= gamma
        status_color = "green" if is_feasible else "yellow"
        status_text = "Feasible" if is_feasible else "Infeasible"

        # Creëer een nette, uitgelijnde output-regel voor de resultaten van de run.
        line = (
            f"  ├─ Run {run_idx:<2} (Seed: {seed:<4}) │ "
            f"Size: {sol.size:<4} │ Edges: {sol.edges:<6} │ "
            f"Density: [bold]{sol.density:.5f}[/bold] │ "
            f"Time: {sol.time:.2f}s │ "
            f"Status: [{status_color}]{status_text}[/{status_color}]"
        )
        self.console.print(line)

    def report_run_error(self, run_idx: int, seed: int, error_msg: str):
        """Rapporteert een fout die is opgetreden tijdens een specifieke run."""
        self.console.print(
            f"  ├─ Run {run_idx} (seed={seed}) [bold red]FAILED[/bold red]: {error_msg}"
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
            summary_text = (
                f"Beste Grootte: [bold green]{best_sol.size}[/bold green] | "
                f"Beste Dichtheid: [bold green]{best_sol.density:.5f}[/bold green]\n"
                f"Totale Tijd: {total_time:.2f}s"
            )
            border_style = "green"
        else:
            summary_text = "[bold red]Geen werkende oplossing gevonden in alle runs.[/bold red]"
            border_style = "red"

        self.console.print(
            Panel(Text(summary_text, justify="center"),
                  title="[bold]Case Samenvatting[/bold]",
                  border_style=border_style,
                  padding=(1, 2)),
            justify="center"
        )

    def report_solve_result(self, sol: SolutionData, params: Dict[str, any]):
        """Presenteert het resultaat van een enkele 'solve' aanroep in een overzichtelijke tabel."""
        table = Table(title="[bold]TSQC Oplosser Resultaat[/bold]", show_header=True, header_style="bold magenta")
        table.add_column("Parameter", style="cyan", no_wrap=True)
        table.add_column("Waarde")

        for key, value in params.items():
            table.add_row(str(key).replace("_", " ").title(), str(value))
        
        table.add_section()
        table.add_row("[bold]Grootte[/bold]", f"[bold green]{sol.size}[/bold green]")
        table.add_row("[bold]Kanten[/bold]", f"[bold green]{sol.edges}[/bold green]")
        table.add_row("[bold]Dichtheid[/bold]", f"[bold green]{sol.density:.5f}[/bold green]")
        table.add_row("[bold]Tijd (s)[/bold]", f"{sol.time:.3f}")
        
        self.console.print(table)
        
    def create_progress_bar(self) -> Progress:
        """Creëert en configureert een 'rich' progressiebalk voor het volgen van de runs."""
        return Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            BarColumn(),
            TextColumn("[progress.percentage]{task.percentage:>3.0f}%"),
            TimeElapsedColumn(),
            console=self.console,
            transient=True  # Laat de progressiebalk verdwijnen na afronding
        )