# src/tsqc/analyzer.py

from pathlib import Path
import pandas as pd
from rich.console import Console
from rich.table import Table
from rich.panel import Panel

def analyze_and_print_top_3(csv_path: Path):
    """
    Analyseert een grid search resultaten-CSV, berekent een score voor elke
    parametercombinatie en print een Top 3.
    """
    console = Console()
    console.print() # Lege regel voor scheiding
    console.print(Panel("[bold cyan]Grid Search Analyse: Top 3 Beste Combinaties[/bold cyan]", expand=False, padding=(0, 2)))

    try:
        df = pd.read_csv(csv_path)
        if df.empty:
            console.print("[yellow]Resultatenbestand is leeg. Kan geen analyse uitvoeren.[/yellow]")
            return
    except FileNotFoundError:
        console.print(f"[red]Fout: Resultatenbestand '{csv_path}' niet gevonden.[/red]")
        return
    except Exception as e:
        console.print(f"[red]Fout bij het lezen van CSV-bestand: {e}[/red]")
        return

    # --- Stap 1: Bepaal de "Moeilijkheidsgraad" (Zeldzaamheidsscore) per benchmark ---
    
    # Een combinatie wordt als 'oplosser' voor een instance beschouwd als het minstens één keer slaagde.
    # We gebruiken de max_solution_density omdat de avg_density misleidend kan zijn bij timeouts.
    df['is_solver'] = df['max_solution_density'] >= df['gamma']
    
    solvers_per_instance = df[df['is_solver']].groupby('instance_name')['combination_id'].nunique()
    total_combinations = df['combination_id'].nunique()

    # De zeldzaamheidsscore is hoger voor problemen die door minder combinaties worden opgelost.
    # We gebruiken total_combinations in de teller voor een betere spreiding van de scores.
    rarity_scores = total_combinations / solvers_per_instance
    
    # Geef een extra hoge bonus aan benchmarks die *nooit* zijn opgelost.
    # Dit is de ultieme uitdaging, dus een oplossing hiervoor moet het zwaarst wegen.
    rarity_scores = rarity_scores.fillna(total_combinations * 2) 
    
    df['rarity_bonus'] = df['instance_name'].map(rarity_scores)

    # --- Stap 2: Bereken de score voor elke combinatie ---

    # De prestatiescore per benchmark hangt af van betrouwbaarheid en kwaliteit.
    df['reliability'] = (df['total_runs'] - df['timeout_count']) / df['total_runs']
    df['performance_score'] = df['reliability'] * (1 + df['avg_solution_density']) # Beloon ook betere dichtheid
    df['weighted_score'] = df['performance_score'] * df['rarity_bonus']

    # Identificeer de parameterkolommen
    param_cols = [
        'combination_id', 'mcts_budget', 'mcts_exploration_const', 'mcts_max_depth',
        'lns_repair_depth', 'stagnation_iter', 'lns_rcl_alpha'
    ]
    
    # Totaalscore per combinatie = som van de gewogen scores over alle benchmarks
    final_scores = df.groupby(param_cols)['weighted_score'].sum().sort_values(ascending=False)

    top_3 = final_scores.head(3)

    if top_3.empty:
        console.print("[yellow]Geen succesvolle combinaties gevonden om een ranking te maken.[/yellow]")
        return

    # --- Stap 3: Presenteer de Top 3 in een nette tabel ---
    table = Table(title="🏆 Top 3 Parameterinstellingen 🏆", show_header=False, show_edge=False, box=None)
    table.add_column("Rank", style="cyan", justify="center", width=7)
    table.add_column("Parameters")
    table.add_column("Score", style="magenta", justify="right")

    for rank, (params, score) in enumerate(top_3.items(), 1):
        param_dict = dict(zip(param_cols[1:], params[1:])) # Sla combination_id over
        
        param_str_parts = []
        for k, v in param_dict.items():
            key_name = k.replace('_', ' ').replace('mcts', 'MCTS').replace('lns', 'LNS').replace('rcl', 'RCL').replace('iter', 'Iter').title()
            if key_name == "Lns Rcl Alpha": key_name = "RCL-α"
            param_str_parts.append(f"[dim]{key_name}:[/dim] [green]{v}[/green]")

        param_str = " | ".join(param_str_parts)
        
        rank_emoji = {1: "🥇 [bold]1[/bold]", 2: "🥈 2", 3: "🥉 3"}.get(rank, f"{rank}.")
        
        table.add_row(rank_emoji, param_str, f"{score:.3f}")

    console.print(table)