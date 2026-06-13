# ternary-resilience

Network resilience analysis with ternary edge weights {−1=Adversarial, 0=Absent, +1=Strong} — connectivity, minimum cut, redundancy, cascading failures, and hardening.

## Background

Network resilience studies how a system maintains function despite failures, attacks, and degradation. Classical percolation theory (Broadbent & Hammersley, 1957) analyzes connectivity under random edge removal. Albert & Barabási (2000) showed that scale-free networks are resilient to random failures but vulnerable to targeted attacks on high-degree nodes. These analyses typically assume uniform edge weights—edges are either present or absent.

The `ternary-resilience` crate extends resilience analysis to networks with **three edge types**:
- **+1 (Strong)**: A reliable, cooperative connection that contributes to connectivity.
- **0 (Absent)**: No connection.
- **−1 (Adversarial)**: A hostile or unreliable connection that should be *avoided*, not relied upon.

This ternary model captures real-world scenarios where connections can be actively harmful—compromised network links, malicious peers in a P2P network, or adversarial relationships in social graphs. Crucially, adversarial edges are **not followed** during connectivity analysis, pathfinding, or community detection. They represent connections that exist but should not be trusted.

The crate implements five core resilience metrics: connectivity (BFS on positive edges), minimum cut (smallest set of positive edges whose removal disconnects the graph), node-independent path count (redundancy), cascading failure simulation, and an aggregate robustness score. A `harden()` function upgrades edges within a budget, providing a concrete hardening strategy.

## How It Works

### Architecture

`ResilientNetwork` stores an `n × n` symmetric adjacency matrix of `i8` weights. Edges are undirected (setting edge (a,b) also sets (b,a)). Positive neighbors are extracted by filtering for weight > 0.

### Connectivity

BFS from node 0 following only positive-weight edges (weight > 0). Adversarial edges (weight = −1) are treated as non-existent for traversal purposes. The network is connected iff all nodes are reachable from node 0 via positive edges.

### Minimum Cut

Brute-force enumeration: try removing 1 positive edge, then 2, then 3, until the graph disconnects. Uses a recursive helper that tries all k-combinations of edges. This is O(2^|E|) in the worst case but practical for small graphs. Returns the minimum number of positive edges whose removal disconnects the graph.

### Redundancy (Node-Independent Paths)

Counts vertex-disjoint paths from node 0 to a target node using iterative BFS with intermediate node removal:
1. Find a path from 0 to target via BFS on positive edges.
2. Remove intermediate nodes (not 0 or target) from the graph.
3. Repeat until no more paths exist.
The count is the number of independent paths—a measure of how many distinct failure scenarios the connection can survive.

### Cascading Failure Simulation

Starting from an initial failed node, simulate spreading failure:
- At each step, a node fails if more than half of its neighbors (by total edge count, including adversarial) are dead.
- This models contagion in infrastructure networks where overloaded neighboring nodes cascade.
- Returns the full history of alive/dead states at each step.

### Backup Paths

DFS enumeration of all simple paths between two nodes using only positive edges. Provides a complete set of alternative routes for contingency planning.

### Robustness Score

An aggregate metric combining average edge weight and connectivity:
- Average weight = Σ weights / |edges| (scaled by 100)
- Connectivity bonus: +50 if connected, −50 if not
- Mapped to ternary output: score > 33 → +1 (robust), score < −33 → −1 (fragile), otherwise → 0 (moderate)

### Hardening

Upgrade edges within a budget: iterate through all edge pairs in order, upgrading each by one step (−1→0→+1) until budget is exhausted. Strategy: prioritize upgrading adversarial edges to neutral, then neutral to strong.

## Experimental Results

All 17 unit tests pass:

| Test | Result | Observation |
|------|--------|-------------|
| `test_empty_network_connected` | ✅ | 0-node network is trivially connected |
| `test_single_node_connected` | ✅ | 1-node network is connected |
| `test_two_nodes_positive_connected` | ✅ | +1 edge connects 2 nodes |
| `test_two_nodes_disconnected` | ✅ | No edges: disconnected |
| `test_triangle_connected` | ✅ | Triangle with all +1 edges: connected |
| `test_adversarial_edge_not_followed` | ✅ | −1 edge between 2 nodes: treated as disconnected |
| `test_min_cut_disconnected` | ✅ | Already disconnected: min_cut = 0 |
| `test_min_cut_single_bridge` | ✅ | Chain 0→1→2→3: min_cut = 1 (bridge at 1-2) |
| `test_redundancy_basic` | ✅ | Triangle (3 nodes, 3 edges): ≥1 independent path |
| `test_redundancy_no_path` | ✅ | No edges: 0 independent paths |
| `test_cascade_single_failure` | ✅ | Chain 0→1→2, node 1 fails: cascades to neighbors |
| `test_cascade_isolated_node` | ✅ | Disconnected graph: cascade limited to connected component |
| `test_backup_paths_basic` | ✅ | Triangle: 2+ backup paths between 0 and 2 |
| `test_backup_paths_no_path` | ✅ | No edges: 0 backup paths |
| `test_robustness_strong` | ✅ | Full triangle: robustness = +1 (robust) |
| `test_robustness_empty` | ✅ | No edges: robustness = −1 (fragile) |
| `test_harden_upgrades_edges` | ✅ | −1 edge upgraded with budget=2: weight increases |

The `test_min_cut_single_bridge` result confirms that a 4-node chain has a single point of failure. The `test_cascade_isolated_node` test shows that cascading failures don't cross disconnected components—adversarial edges or absent edges act as firebreaks.

## Impact of Ternary {-1, 0, +1}

The ternary edge model enables **triage-based resilience assessment**:

- **+1 edges** are assets that contribute to connectivity and robustness. More +1 edges → higher robustness score.
- **0 edges** represent untapped potential—connections that could be established but aren't. Harden() upgrades these to +1.
- **−1 edges** are liabilities that actively harm the network. They don't contribute to connectivity (in fact, they reduce the robustness score via negative average weight) but their presence indicates adversarial pressure.

This three-valued model is richer than "connected/disconnected" and more actionable than weighted graphs with continuous values.

## Use Cases

1. **Infrastructure Network Hardening**: Model a power grid or transportation network with +1 for reliable links, −1 for known-vulnerable links, and 0 for potential links. Compute min-cut to find critical failure points, then apply `harden()` within a maintenance budget.

2. **Adversarial Network Analysis**: In cybersecurity, model network topology with −1 for compromised links and +1 for secured links. Connectivity analysis reveals which nodes are isolated from the trusted network; backup paths provide alternative routes avoiding compromised links.

3. **Supply Chain Resilience**: Model supplier relationships: +1 for reliable suppliers, −1 for risky suppliers. Cascading failure simulation shows how a single supplier failure can propagate through the supply chain.

4. **Social Network Contagion Modeling**: Simulate information or disease spread with ternary edges (+1 = close contact, −1 = antagonistic relationship that reduces spread). Cascading failure models viral spread thresholds.

5. **GPU Cluster Fault Tolerance**: Model inter-GPU communication links. After detecting a faulty NVLink (set to −1), compute backup paths for data redistribution and measure how many independent paths remain to each GPU.

## Open Questions

1. **Scalable Minimum Cut**: The current brute-force approach is exponential. For production networks with hundreds or thousands of nodes, Stoer-Wagner's algorithm (O(V³)) or Karger's randomized algorithm would be needed. How should these handle the ternary edge distinction?

2. **Probabilistic Edge States**: Real networks have edges that are "sometimes reliable." Should the model extend to probabilistic ternary states (e.g., +1 with 90% confidence, −1 with 70% confidence)?

3. **Dynamic Resilience**: The current model is static. In practice, edges change state over time (links recover, new adversaries appear). How should the robustness score and hardening strategy adapt to a time-varying ternary graph?

## Connection to Oxide Stack

Within the five-layer Oxide ternary architecture:

- **Layer 1 (Ternary Genome)**: Edge weights {-1, 0, +1} are genome bases, encoding network topology as genetic information. Evolutionary pressure selects for robust network structures.
- **Layer 2 (Cellular Computation)**: Each node is a computational cell; the adjacency matrix defines the cell communication graph. Ternary edges classify communication channels as cooperative (+1), absent (0), or adversarial (−1).
- **Layer 3 (Organism Behavior)**: An organism navigating the network uses ternary edge information to choose paths—preferring +1, avoiding −1. The robustness score drives risk-aware behavior.
- **Layer 4 (Population Dynamics)**: Population-level resilience is the aggregate of individual network positions. Organisms at low-redundancy nodes are more vulnerable; populations concentrated in single communities face correlated failure risk.
- **Layer 5 (Ecosystem)**: The ecosystem's resilience profile—robustness score, min-cut, cascade behavior—determines how the system responds to external shocks. Harden() at this level represents ecosystem-level investment in redundancy and diversity.
