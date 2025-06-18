# src/tsqc/reporter.py
# NIEUW BESTAND
"""
Deze module bevat de `Reporter`-klasse, die verantwoordelijk is voor alle
terminal-output. Het gebruikt de 'rich' bibliotheek voor visueel aantrekkelijke
en gestructureerde feedback, zoals panelen, tabellen en gekleurde tekst.
Dit centraliseert alle print-logica op één plek, waardoor de CLI-code
(in cli.py) schoner wordt en zich kan focussen op de uitvoeringslogica.
"""

from typing import Optional, Dict

# Rich is een krachtige bibliotheek voor mooie terminal-output.
# Deze moet later worden toegevoegd aan de dependencies in pyproject.toml.
from rich.console import Console
from rich.panel import Panel
from rich.table import Table
from rich.text import Text
from rich.progress import Progress, SpinnerColumn, BarColumn, TextColumn, TimeElapsedColumn

# Importeer de datastructuren.
# Deze import gaat er vanuit dat we later de __init__.py en api.py aanpassen.
from tsqc.api import SolutionData
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
        
        # Gebruik Text(subtitle, justify="center", markup=True) als subtitle markup bevat,
        # maar hier bevat het geen markup, dus dit is prima.
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
        
        status_text = ""
        status_color = "yellow" # Default for timed out or infeasible

        if sol.is_timed_out:
            status_text = "Timed Out"
            status_color = "orange3"
        elif is_feasible:
            status_text = "Feasible"
            status_color = "green"
        else:
            status_text = "Infeasible"
            status_color = "yellow"

        # Gebruik een f-string met directe markup, want console.print() verwerkt dit correct.
        line = (
            f"  ├─ Run {run_idx:<2} (Seed: {seed:<4}) │ "
            f"Size: {sol.size:<4} │ Edges: {sol.edges:<6} │ "
            f"Density: [bold]{sol.density:.5f}[/bold] │ "
            f"Time: {sol.time:.2f}s │ "
            f"Status: [{status_color}]{status_text}[/{status_color}]"
        )
        self.console.print(line, markup=True) # Zorg ervoor dat markup wordt geparset

    def report_run_error(self, run_idx: int, seed: int, error_msg: str):
        """Rapporteert een fout die is opgetreden tijdens een specifieke run."""
        self.console.print(
            f"  ├─ Run {run_idx} (seed={seed}) [bold red]FAILED[/bold red]: {error_msg}",
            markup=True # Zorg ervoor dat markup wordt geparset
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
            # CORRECTIE: Maak één Text object en gebruik daarin de markup.
            # `Text.assemble` is een goede optie voor het samenstellen van opgemaakte tekst.
            summary_text = Text.assemble(
                "Beste Grootte: ", Text(str(best_sol.size), style="bold green"),
                " | Beste Dichtheid: ", Text(f"{best_sol.density:.5f}", style="bold green"),
                f"\nTotale Tijd: {total_time:.2f}s"
            )
            border_style = "green"
        else:
            # Hier direct een Text object met markup True als de hele string markup bevat.
            summary_text = Text("[bold red]Geen werkende oplossing gevonden in alle runs.[/bold red]", markup=True)
            border_style = "red"

        self.console.print(
            Panel(summary_text, # Geef het reeds opgemaakte Text object direct door
                  title="[bold]Case Samenvatting[/bold]", # Titel bevat ook markup, dit werkt prima zonder extra markup=True
                  border_style=border_style,
                  padding=(1, 2)),
            justify="center"
        )
        # De extra lege regel na de samenvatting is al eerder verwijderd in cli.py of hier.
        # self.console.print() 

    def report_solve_result(self, sol: SolutionData, params: Dict[str, any]):
        """Presenteert het resultaat van een enkele 'solve' aanroep in een overzichtelijke tabel."""
        table = Table(title="[bold]TSQC Oplosser Resultaat[/bold]", show_header=True, header_style="bold magenta",
                      caption_justify="center") # Optioneel: centreer de titel

        table.add_column("Parameter", style="cyan", no_wrap=True)
        table.add_column("Waarde")

        for key, value in params.items():
            # Gebruik Text objecten of zorg dat de string geen markup heeft als Text().markup=True niet wordt gebruikt
            table.add_row(Text(str(key).replace("_", " ").title()), Text(str(value)))
        
        table.add_section()
        # Gebruik Text objecten voor opmaak
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
            TextColumn("[progress.percentage]{task.percentage:>3.0f}%"),
            TimeElapsedColumn(),
            console=self.console,
            transient=True  # Laat de progressiebalk verdwijnen na afronding
        )