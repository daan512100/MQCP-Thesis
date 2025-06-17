#!/usr/bin/env python3
"""
Command-line interface (CLI) for the TSQC solver.

Biedt een gestructureerde interface voor het uitvoeren van fixed-k en max-k
zoekopdrachten, met optionele MCTS-LNS diversificatie.
"""
import argparse
import sys
import time
import os
from pathlib import Path
from tsqc import solve_k_py, solve_max_py, parse_dimacs_py

def main():
    """Hoofdfunctie voor het parsen van argumenten en het aanroepen van de oplosser."""
    parser = argparse.ArgumentParser(
         description="Een productieklare oplosser voor het Maximum Quasi-Clique Probleem.",
        formatter_class=argparse.RawTextHelpFormatter
    )

    # --- Groep voor vereiste argumenten ---
    req_group = parser.add_argument_group("Vereiste Argumenten")
    req_group.add_argument(
        "-i", "--instance", type=Path, required=True,
        help="Pad naar het DIMACS.clq-instantiebestand."
    )
    req_group.add_argument(
        "-g", "--gamma", type=float, required=True,
        help="Doeldichtheid gamma (0, 1] voor de quasi-clique."
    )

    # --- Groep voor oplossermodus ---
    mode_group = parser.add_argument_group("Oplossermodus")
    mode_group.add_argument(
        "--mode", choices=["fixed", "max"], default="max",
        help="Kies de zoekmodus: 'fixed' voor een vaste k, 'max' voor maximale k.\n(default: max)"
    )
    mode_group.add_argument(
        "-k", type=int,
        help="Doelgrootte k voor de 'fixed' modus."
    )

    # --- Groep voor uitvoeringscontrole ---
    exec_group = parser.add_argument_group("Uitvoeringscontrole")
    exec_group.add_argument(
        "-r", "--runs", type=int, default=1,
        help="Aantal onafhankelijke runs met verschillende seeds.\n(default: 1)"
    )
    exec_group.add_argument(
        "-s", "--seed", type=int, default=42,
        help="Basis-seed voor de random number generator. (default: 42)"
    )
    exec_group.add_argument(
        "--threads", type=int, default=None,
        help="Aantal threads voor Rayon (parallelle MCTS). (default: alle beschikbare cores)"
    )

    # --- Groep voor MCTS-LNS Diversificatie ---
    mcts_group = parser.add_argument_group(
        "MCTS-LNS Diversificatieparameters",
        "Deze opties zijn alleen actief als --use-mcts is opgegeven."
    )
    mcts_group.add_argument(
        "--use-mcts", action="store_true",
        help="Schakel MCTS-LNS in als diversificatiestrategie."
    )
    mcts_group.add_argument("--mcts-budget", type=int, default=100, help="Simulatiebudget voor MCTS. (default: 100)")
    mcts_group.add_argument("--mcts-uct", type=float, default=1.414, help="UCT-exploratieconstante. (default: 1.414)")
    mcts_group.add_argument("--mcts-depth", type=int, default=5, help="Maximale boomdiepte voor MCTS. (default: 5)")
    
    # CORRECTIE 1: De naam van het command-line argument is aangepast naar '--lns-repair-depth'.
    mcts_group.add_argument("--lns-repair-depth", type=int, default=10, help="Aantal hersteliteraties voor LNS. (default: 10)")

    args = parser.parse_args()

    # --- Validatie van argumenten ---
    if args.mode == "fixed" and args.k is None:
        parser.error("-k is vereist wanneer --mode=fixed is geselecteerd.")
    if not args.instance.is_file():
        parser.error(f"Instantiebestand niet gevonden: {args.instance}")

    # --- Configuratie van parallellisme ---
    if args.threads:
        os.environ["RAYON_NUM_THREADS"] = str(args.threads)

    # --- Uitvoering van de oplosser ---
    try:
        n, m = parse_dimacs_py(str(args.instance))
        print(f"--- Instantie: {args.instance.name} (n={n}, m={m}, gamma={args.gamma}) ---")
    except Exception as e:
        print(f"Fout bij het parsen van de DIMACS-header: {e}", file=sys.stderr)
        sys.exit(1)

    print(f"{'Run':>3s} | {'Seed':>10s} | {'Size':>5s} | {'Edges':>8s} | {'Density':>9s} | {'Time (s)':>10s}")
    print("-" * 65)

    total_start_time = time.perf_counter()
    
    for run_idx in range(1, args.runs + 1):
        run_seed = args.seed + run_idx - 1
        start_time = time.perf_counter()

        try:
            # CORRECTIE 2: De naam van het keyword argument is hier ook aangepast.
            if args.mode == 'max':
                size, edges, density = solve_max_py(
                    instance_path=str(args.instance), 
                    gamma=args.gamma, 
                    seed=run_seed, 
                    runs=1,
                    use_mcts=args.use_mcts, 
                    mcts_budget=args.mcts_budget, 
                    mcts_uct=args.mcts_uct, 
                    mcts_depth=args.mcts_depth, 
                    lns_repair_depth=args.lns_repair_depth # Aangepast van lns_repair
                )
            else: # mode == 'fixed'
                size, edges, density = solve_k_py(
                    instance_path=str(args.instance),
                    k=args.k,
                    gamma=args.gamma, 
                    seed=run_seed, 
                    runs=1,
                    use_mcts=args.use_mcts, 
                    mcts_budget=args.mcts_budget, 
                    mcts_uct=args.mcts_uct, 
                    mcts_depth=args.mcts_depth, 
                    lns_repair_depth=args.lns_repair_depth # Aangepast van lns_repair
                )
            
            elapsed = time.perf_counter() - start_time
            print(f"{run_idx:3d} | {run_seed:10d} | {size:5d} | {edges:8d} | {density:9.5f} | {elapsed:10.3f}")

        except Exception as e:
            elapsed = time.perf_counter() - start_time
            print(f"Fout tijdens run {run_idx} (seed={run_seed}): {e}", file=sys.stderr)
            print(f"{run_idx:3d} | {run_seed:10d} | {'FAIL':>5s} | {'-':>8s} | {'-':>9s} | {elapsed:10.3f}")

    total_elapsed = time.perf_counter() - total_start_time
    print("-" * 65)
    print(f"Totale tijd voor {args.runs} runs: {total_elapsed:.3f}s")

if __name__ == "__main__":
    main()