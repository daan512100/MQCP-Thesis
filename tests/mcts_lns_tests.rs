// tests/mcts_lns_tests.rs
//! Unit tests voor MCTS-LNS implementatie: validatie van UCT-formule,
//! selectie, expansie, rollout en backpropagation.

extern crate tsqc;
use tsqc::graph::Graph;
use tsqc::params::Params;
use tsqc::solution::Solution;
use tsqc::mcts::{MctsTree, MctsNode};
use rand::{SeedableRng, rngs::StdRng};

#[test]
fn test_uct_score_infinite() {
    let graph = Graph::with_vertices(1);
    let params = Params::default();
    let initial_solution = Solution::new(&graph);
    let tree = MctsTree::new(&initial_solution, &graph, &params);
    // Zero bezoeken → oneindige UCT-score
    assert!(tree.uct_score(0, 1).is_infinite());
}

#[test]
fn test_uct_score_finite() {
    let graph = Graph::with_vertices(1);
    let mut params = Params::default();
    // Stel exploration constant op 2.0
    params.enable_mcts(1, 2.0, 1, 1);
    let initial_solution = Solution::new(&graph);
    let mut tree = MctsTree::new(&initial_solution, &graph, &params);
    // Simuleer eerdere bezoeken en beloning
    tree.nodes[0].visits = 2;
    tree.nodes[0].total_reward = 4.0;
    let score = tree.uct_score(0, 10);
    let exploitation = 4.0 / 2.0;
    let exploration = 2.0 * ((10f64.ln() / 2.0).sqrt());
    let expected = exploitation + exploration;
    assert!((score - expected).abs() < 1e-6);
}

#[test]
fn test_select_initial() {
    let graph = Graph::with_vertices(3);
    let params = Params::default();
    let initial_solution = Solution::new(&graph);
    let tree = MctsTree::new(&initial_solution, &graph, &params);
    let (idx, path) = tree.select();
    assert_eq!(idx, 0);
    assert!(path.is_empty());
}

#[test]
fn test_expand_no_visit() {
    let graph = Graph::with_vertices(2);
    let params = Params::default();
    let initial_solution = Solution::new(&graph);
    let mut tree = MctsTree::new(&initial_solution, &graph, &params);
    let next = tree.expand(0, &[], &mut StdRng::seed_from_u64(0));
    assert_eq!(next, 0);
}

#[test]
fn test_rollout_returns_size() {
    let mut graph = Graph::with_vertices(2);
    graph.add_edge(0, 1);
    let params = Params::default();
    let initial_solution = Solution::new(&graph);
    let mut tree = MctsTree::new(&initial_solution, &graph, &params);
    let node = &tree.nodes[0];
    let size = tree.rollout(node, &mut StdRng::seed_from_u64(0));
    assert_eq!(size, initial_solution.size() as f64);
}

#[test]
fn test_backpropagate() {
    let graph = Graph::with_vertices(1);
    let params = Params::default();
    let initial_solution = Solution::new(&graph);
    let mut tree = MctsTree::new(&initial_solution, &graph, &params);
    // Voeg een extra node voor backpropagation-test
    let child = MctsNode { parent: Some(0), children: Vec::new(), visits: 0, total_reward: 0.0, vertex_removed: None, depth: 0 };
    tree.nodes.push(child);
    let idx = tree.nodes.len() - 1;
    tree.backpropagate(idx, 3.5);
    assert_eq!(tree.nodes[idx].visits, 1);
    assert_eq!(tree.nodes[idx].total_reward, 3.5);
    assert_eq!(tree.nodes[0].visits, 1);
    assert_eq!(tree.nodes[0].total_reward, 3.5);
}
