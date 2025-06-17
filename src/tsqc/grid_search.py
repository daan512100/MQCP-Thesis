# Bestand: tsqc/grid_search.py
"""
Grid search framework voor MCTS-parameters: doorloopt combinaties van C, alpha en iteraties,
voert MCTS-LNS runs uit (max-k) en slaat resultaten op in JSON.
"""
import os
import glob
import json
from itertools import product
from typing import Optional, Dict, List, Union
from pathlib import Path
from tsqc.api import solve_max


def grid_search(
    inst_dir: Union[str, os.PathLike],
    gamma: float,
    grid: Dict[str, List],
    time_limit: Optional[float] = None,
    output_json: str = "grid_results.json",
) -> None:
    """
    Doorloop alle .clq-instanties in inst_dir en grid-combinaties,
    voer MCTS-LNS (max-k) uit met elke parametercombinatie.

    Parameters:
    - inst_dir: pad naar map met .clq-instanties
    - gamma: dichtheidsdrempel voor quasi-cliques
    - grid: dict met lijsten voor keys 'C', 'alpha', 'iters'
    - time_limit: (optioneel) time limit override (niet gebruikt in huidige API)
    - output_json: pad naar output JSON-bestand

    Output JSON structure: lijst van dicts met velden:
      instance, C, alpha, iters, size, edges, density, time, error(optional)
    """
    # Vind alle .clq-bestanden
    pattern = os.path.join(str(inst_dir), "*.clq")
    instances = sorted(glob.glob(pattern))
    results = []

    for inst in instances:
        basename = Path(inst).name
        for C, alpha, iters in product(grid.get("C", []), grid.get("alpha", []), grid.get("iters", [])):
            try:
                sol = solve_max(
                    instance_path=inst,
                    gamma=gamma,
                    seed=0,
                    runs=1,
                    use_mcts=True,
                    mcts_budget=C,
                    mcts_uct=alpha,
                    mcts_depth=iters,
                    lns_repair_depth=10,
                )
                results.append({
                    "instance": basename,
                    "C": C,
                    "alpha": alpha,
                    "iters": iters,
                    "size": sol.size,
                    "edges": sol.edges,
                    "density": sol.density,
                    "time": f"{sol.time:.6f}",
                })
            except Exception as e:
                results.append({
                    "instance": basename,
                    "C": C,
                    "alpha": alpha,
                    "iters": iters,
                    "error": str(e),
                })

    # Schrijf resultaten naar JSON
    with open(output_json, "w") as f:
        json.dump(results, f, indent=2)
    print(f"Grid search complete, results saved to {output_json}")
