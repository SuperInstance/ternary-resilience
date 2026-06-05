#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResilientNetwork {
    /// Number of nodes
    pub n: usize,
    /// Adjacency matrix: weights[i][j] = -1 (adversarial), 0 (absent), 1 (strong)
    pub weights: Vec<Vec<i8>>,
}

impl ResilientNetwork {
    pub fn new(n: usize) -> Self {
        Self {
            n,
            weights: vec![vec![0i8; n]; n],
        }
    }

    pub fn set_edge(&mut self, a: usize, b: usize, w: i8) {
        self.weights[a][b] = w;
        self.weights[b][a] = w;
    }

    /// Get neighbors connected by positive edges
    fn positive_neighbors(&self, node: usize) -> Vec<usize> {
        let mut neighbors = Vec::new();
        for j in 0..self.n {
            if self.weights[node][j] > 0 {
                neighbors.push(j);
            }
        }
        neighbors
    }

    /// BFS/DFS from a starting node following only positive edges
    fn reachable(&self, start: usize) -> Vec<bool> {
        let mut visited = vec![false; self.n];
        let mut stack = vec![start];
        visited[start] = true;
        while let Some(node) = stack.pop() {
            for &nb in &self.positive_neighbors(node) {
                if !visited[nb] {
                    visited[nb] = true;
                    stack.push(nb);
                }
            }
        }
        visited
    }
}

/// Is the graph connected (following only positive edges)?
pub fn connectivity(network: &ResilientNetwork) -> bool {
    if network.n == 0 {
        return true;
    }
    let visited = network.reachable(0);
    visited.iter().all(|&v| v)
}

/// Minimum number of positive edges whose removal disconnects the graph.
/// For small graphs, tries removing each edge and checking connectivity.
pub fn min_cut(network: &ResilientNetwork) -> usize {
    if network.n <= 1 {
        return 0;
    }
    if !connectivity(network) {
        return 0;
    }

    // Collect all positive edges (i < j)
    let mut edges: Vec<(usize, usize)> = Vec::new();
    for i in 0..network.n {
        for j in (i + 1)..network.n {
            if network.weights[i][j] > 0 {
                edges.push((i, j));
            }
        }
    }

    if edges.is_empty() {
        return 0;
    }

    // Try removing 1 edge at a time (simplified min-cut)
    for k in 1..=edges.len() {
        if _try_remove_k(network, &edges, k, 0, &mut vec![false; edges.len()]) {
            return k;
        }
    }
    edges.len()
}

fn _try_remove_k(
    network: &ResilientNetwork,
    edges: &[(usize, usize)],
    k: usize,
    start: usize,
    removed: &mut Vec<bool>,
) -> bool {
    if k == 0 {
        // Check if graph is disconnected
        let mut test = network.clone();
        for (idx, &r) in removed.iter().enumerate() {
            if r {
                let (a, b) = edges[idx];
                test.weights[a][b] = 0;
                test.weights[b][a] = 0;
            }
        }
        return !connectivity(&test);
    }

    for i in start..=(edges.len() - k) {
        removed[i] = true;
        if _try_remove_k(network, edges, k - 1, i + 1, removed) {
            removed[i] = false;
            return true;
        }
        removed[i] = false;
    }
    false
}

/// Count the number of node-independent (vertex-disjoint) paths to a given node.
/// Uses a simplified approach: BFS-based, removing intermediate nodes.
pub fn redundancy(network: &ResilientNetwork, target: usize) -> usize {
    if network.n <= 1 {
        return 0;
    }

    let mut count = 0;
    let mut current = network.clone();

    loop {
        let visited = current.reachable(0);
        if !visited[target] {
            break;
        }
        count += 1;

        // Find a path and remove intermediate nodes
        let path = _find_path(&current, 0, target);
        if path.len() <= 2 {
            // Direct edge — remove it
            current.weights[0][target] = 0;
            current.weights[target][0] = 0;
        } else {
            // Remove intermediate nodes (set all their edges to 0)
            for &node in &path[1..path.len() - 1] {
                for j in 0..current.n {
                    current.weights[node][j] = 0;
                    current.weights[j][node] = 0;
                }
            }
        }
    }
    count
}

fn _find_path(network: &ResilientNetwork, start: usize, end: usize) -> Vec<usize> {
    let mut parent = vec![None; network.n];
    let mut visited = vec![false; network.n];
    let mut queue = vec![start];
    visited[start] = true;

    while let Some(node) = queue.pop() {
        if node == end {
            // Reconstruct path
            let mut path = vec![end];
            let mut cur = end;
            while let Some(p) = parent[cur] {
                path.push(p);
                cur = p;
            }
            path.reverse();
            return path;
        }
        for &nb in &network.positive_neighbors(node) {
            if !visited[nb] {
                visited[nb] = true;
                parent[nb] = Some(node);
                queue.push(nb);
            }
        }
    }
    vec![]
}

/// Simulate cascading failure: when a node fails, its neighbors may fail too.
/// Returns a vector of which nodes are alive (true) after each step.
pub fn cascade_failure(network: &ResilientNetwork, failed_node: usize, steps: usize) -> Vec<Vec<bool>> {
    let mut alive = vec![true; network.n];
    alive[failed_node] = false;
    let mut history = vec![alive.clone()];

    for _ in 0..steps {
        let mut new_alive = alive.clone();
        let mut changed = false;
        for node in 0..network.n {
            if !alive[node] {
                continue;
            }
            // Count how many neighbors are dead
            let mut dead_neighbors = 0usize;
            let mut total_neighbors = 0usize;
            for j in 0..network.n {
                if network.weights[node][j] != 0 {
                    total_neighbors += 1;
                    if !alive[j] {
                        dead_neighbors += 1;
                    }
                }
            }
            // If more than half of neighbors are dead, this node fails too
            if total_neighbors > 0 && dead_neighbors * 2 > total_neighbors {
                new_alive[node] = false;
                changed = true;
            }
        }
        alive = new_alive;
        history.push(alive.clone());
        if !changed {
            break;
        }
    }
    history
}

/// Find all paths from a to b using only positive-weight edges.
pub fn backup_paths(network: &ResilientNetwork, a: usize, b: usize) -> Vec<Vec<usize>> {
    let mut results = Vec::new();
    let mut path = vec![a];
    let mut visited = vec![false; network.n];
    visited[a] = true;
    _dfs_paths(network, a, b, &mut visited, &mut path, &mut results);
    results
}

fn _dfs_paths(
    network: &ResilientNetwork,
    current: usize,
    target: usize,
    visited: &mut [bool],
    path: &mut Vec<usize>,
    results: &mut Vec<Vec<usize>>,
) {
    if current == target {
        results.push(path.clone());
        return;
    }
    for j in 0..network.n {
        if network.weights[current][j] > 0 && !visited[j] {
            visited[j] = true;
            path.push(j);
            _dfs_paths(network, j, target, visited, path, results);
            path.pop();
            visited[j] = false;
        }
    }
}

/// Aggregate resilience metric: -1 (fragile) to 1 (robust).
/// Based on connectivity, redundancy, and edge quality.
pub fn robustness_score(network: &ResilientNetwork) -> i8 {
    if network.n == 0 {
        return 1;
    }

    let mut total_weight: i16 = 0;
    let mut edge_count: i16 = 0;
    for i in 0..network.n {
        for j in (i + 1)..network.n {
            if network.weights[i][j] != 0 {
                total_weight += network.weights[i][j] as i16;
                edge_count += 1;
            }
        }
    }

    if edge_count == 0 {
        return -1;
    }

    let avg_weight = total_weight * 100 / edge_count;

    let conn_bonus: i16 = if connectivity(network) { 50 } else { -50 };

    let score = (avg_weight + conn_bonus).clamp(-100, 100);
    // Map to -1, 0, 1
    if score > 33 {
        1
    } else if score < -33 {
        -1
    } else {
        0
    }
}

/// Upgrade edges to improve robustness within budget (each upgrade costs 1).
/// Upgrades adversarial (-1) → absent (0) → strong (1).
pub fn harden(network: &ResilientNetwork, budget: usize) -> ResilientNetwork {
    let mut result = network.clone();
    let mut remaining = budget;

    // Strategy: upgrade adversarial edges first, then absent edges
    // Priority: adversarial edges on the path of connected components
    for i in 0..result.n {
        if remaining == 0 {
            break;
        }
        for j in (i + 1)..result.n {
            if remaining == 0 {
                break;
            }
            if result.weights[i][j] < 1 {
                result.weights[i][j] += 1;
                result.weights[j][i] += 1;
                remaining -= 1;
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_network_connected() {
        let net = ResilientNetwork::new(0);
        assert!(connectivity(&net));
    }

    #[test]
    fn test_single_node_connected() {
        let net = ResilientNetwork::new(1);
        assert!(connectivity(&net));
    }

    #[test]
    fn test_two_nodes_positive_connected() {
        let mut net = ResilientNetwork::new(2);
        net.set_edge(0, 1, 1);
        assert!(connectivity(&net));
    }

    #[test]
    fn test_two_nodes_disconnected() {
        let net = ResilientNetwork::new(2);
        assert!(!connectivity(&net));
    }

    #[test]
    fn test_triangle_connected() {
        let mut net = ResilientNetwork::new(3);
        net.set_edge(0, 1, 1);
        net.set_edge(1, 2, 1);
        net.set_edge(0, 2, 1);
        assert!(connectivity(&net));
    }

    #[test]
    fn test_adversarial_edge_not_followed() {
        let mut net = ResilientNetwork::new(2);
        net.set_edge(0, 1, -1);
        assert!(!connectivity(&net));
    }

    #[test]
    fn test_min_cut_disconnected() {
        let net = ResilientNetwork::new(2);
        assert_eq!(min_cut(&net), 0);
    }

    #[test]
    fn test_min_cut_single_bridge() {
        let mut net = ResilientNetwork::new(4);
        net.set_edge(0, 1, 1);
        net.set_edge(1, 2, 1);
        net.set_edge(2, 3, 1);
        assert_eq!(min_cut(&net), 1);
    }

    #[test]
    fn test_redundancy_basic() {
        let mut net = ResilientNetwork::new(3);
        net.set_edge(0, 1, 1);
        net.set_edge(1, 2, 1);
        net.set_edge(0, 2, 1);
        assert!(redundancy(&net, 2) >= 1);
    }

    #[test]
    fn test_redundancy_no_path() {
        let net = ResilientNetwork::new(3);
        assert_eq!(redundancy(&net, 2), 0);
    }

    #[test]
    fn test_cascade_single_failure() {
        let mut net = ResilientNetwork::new(3);
        net.set_edge(0, 1, 1);
        net.set_edge(1, 2, 1);
        let history = cascade_failure(&net, 1, 5);
        assert!(!history[0][1]); // node 1 dead initially
        assert!(history.len() > 1);
    }

    #[test]
    fn test_cascade_isolated_node() {
        let mut net = ResilientNetwork::new(4);
        net.set_edge(0, 1, 1);
        net.set_edge(2, 3, 1);
        let history = cascade_failure(&net, 0, 5);
        // Node 1 only has one neighbor (node 0) which is dead → 1/1 dead → cascades
        assert!(!history[history.len() - 1][1]);
    }

    #[test]
    fn test_backup_paths_basic() {
        let mut net = ResilientNetwork::new(3);
        net.set_edge(0, 1, 1);
        net.set_edge(1, 2, 1);
        net.set_edge(0, 2, 1);
        let paths = backup_paths(&net, 0, 2);
        assert!(paths.len() >= 2);
    }

    #[test]
    fn test_backup_paths_no_path() {
        let net = ResilientNetwork::new(3);
        let paths = backup_paths(&net, 0, 2);
        assert!(paths.is_empty());
    }

    #[test]
    fn test_robustness_strong() {
        let mut net = ResilientNetwork::new(3);
        net.set_edge(0, 1, 1);
        net.set_edge(1, 2, 1);
        net.set_edge(0, 2, 1);
        assert_eq!(robustness_score(&net), 1);
    }

    #[test]
    fn test_robustness_empty() {
        let net = ResilientNetwork::new(3);
        assert_eq!(robustness_score(&net), -1);
    }

    #[test]
    fn test_harden_upgrades_edges() {
        let mut net = ResilientNetwork::new(2);
        net.set_edge(0, 1, -1);
        let hardened = harden(&net, 2);
        assert!(hardened.weights[0][1] > -1);
    }
}
