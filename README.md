# ternary-resilience

Network resilience analysis with ternary edge weights. Connectivity, min-cut, redundancy, cascade failures, and backup paths.

How fragile is your network? If you remove one node, does everything stay connected? If two nodes fail simultaneously, does the network fragment? This crate answers these questions by modeling networks as graphs with ternary edge weights: **+1 (strong)**, **0 (absent)**, **-1 (adversarial)**. The adversarial edges are the key innovation—they represent hostile connections that exist but harm connectivity, giving you a richer model than simple connected/disconnected.

Built for `no_std` environments with only `alloc`. Runs on microcontrollers, embedded systems, and anywhere you need to reason about network topology without pulling in a graph library.

## Why this exists

Network resilience analysis typically treats edges as present or absent. But real networks have hostile connections: adversarial links that actively undermine connectivity. In a multi-agent system, a compromised node doesn't just disconnect—it can actively route traffic through itself to disrupt communication.

The ternary edge model captures three states:
- **+1**: A healthy, reliable link (trust, cooperation, strong signal)
- **0**: No link (absent, untested, neutral)
- **-1**: An adversarial link (mistrust, defection, active interference)

Positive edges are followed for connectivity. Adversarial edges are *present* but *not followed*—they exist in the topology without contributing to reachability. This distinction matters for cascade failure modeling.

## The key insight

A network's resilience is a ternary property, not a spectrum. Given the robustness score algorithm, your network falls into one of three buckets:

| Score | Meaning | Interpretation |
|-------|---------|----------------|
| +1 | Robust | Well-connected with strong edges |
| 0 | Marginal | Some connectivity, but fragile |
| -1 | Fragile | Disconnected or dominated by adversarial edges |

This isn't a simplification—it's the right granularity for decision-making. When you're deciding whether to harden a network, you don't need a floating-point score. You need to know: *is this network safe, at risk, or broken?*

## Quick start

```rust
use ternary_resilience::*;

// Build a triangle network with strong edges
let mut net = ResilientNetwork::new(3);
net.set_edge(0, 1, 1);
net.set_edge(1, 2, 1);
net.set_edge(0, 2, 1);

// Check basic properties
assert!(connectivity(&net));           // fully connected
assert_eq!(min_cut(&net), 2);         // need to remove 2 edges to disconnect
assert_eq!(robustness_score(&net), 1); // robust

// Find backup paths between nodes 0 and 2
let paths = backup_paths(&net, 0, 2);
assert!(paths.len() >= 2);  // at least 2 independent paths
```

## API reference

### ResilientNetwork

```rust
let mut net = ResilientNetwork::new(n_nodes);

// Set edge weight: -1 (adversarial), 0 (absent), 1 (strong)
net.set_edge(a, b, weight);

// Access state
net.n;               // node count
net.weights;         // adjacency matrix (Vec<Vec<i8>>)
```

### Analysis functions

```rust
// Is the graph connected (following only positive edges)?
connectivity(&net)                    // → bool

// Minimum edges to remove to disconnect
min_cut(&net)                         // → usize

// Number of vertex-disjoint paths to a target
redundancy(&net, target_node)         // → usize

// Simulate cascading failure from a failed node
cascade_failure(&net, failed_node, max_steps)  // → Vec<Vec<bool>>

// Find all paths between two nodes (positive edges only)
backup_paths(&net, a, b)              // → Vec<Vec<usize>>

// Aggregate resilience score
robustness_score(&net)                // → i8 {-1, 0, 1}

// Upgrade edges within a budget
harden(&net, budget)                  // → ResilientNetwork
```

## Cascade failure simulation

The most powerful tool in the crate. When a node fails, its neighbors feel the impact. If more than half of a node's neighbors are dead, it fails too—creating a cascade.

```rust
let mut net = ResilientNetwork::new(6);
// Linear chain: 0-1-2-3-4-5
for i in 0..5 { net.set_edge(i, i+1, 1); }

// Fail node 2 — watch the cascade
let history = cascade_failure(&net, 2, 10);

// history[0] = initial state (node 2 dead)
// history[1] = after one cascade step
// ...continues until no more nodes fail or steps exhausted

for (step, alive) in history.iter().enumerate() {
    let alive_count = alive.iter().filter(|&&a| a).count();
    println!("Step {}: {}/{} nodes alive", step, alive_count, alive.len());
}
```

Each step of the cascade checks every alive node: if more than half its neighbors (weighted by non-zero edges) are dead, it fails. This creates realistic failure propagation where nodes with few connections are more vulnerable.

## Backup paths and redundancy

```rust
// Redundancy: how many independent paths reach the target?
let mut net = ResilientNetwork::new(4);
net.set_edge(0, 1, 1);
net.set_edge(1, 3, 1);
net.set_edge(0, 2, 1);
net.set_edge(2, 3, 1);

let paths = redundancy(&net, 3);  // 2 vertex-disjoint paths
assert!(paths >= 2);

// All paths (not just vertex-disjoint)
let all_paths = backup_paths(&net, 0, 3);
// [[0, 1, 3], [0, 2, 3]]
```

## Hardening a network

Given a budget, `harden()` upgrades edges: adversarial → absent → strong, prioritizing adversarial edges first.

```rust
let mut net = ResilientNetwork::new(3);
net.set_edge(0, 1, -1);  // adversarial
net.set_edge(1, 2, 0);   // absent

// With budget 3, upgrade all edges
let hardened = harden(&net, 3);
assert_eq!(hardened.weights[0][1], 0);   // -1 → 0 (first upgrade)
assert_eq!(hardened.weights[1][2], 1);   // 0 → 1 (second upgrade)
```

## Architecture

The network is stored as an adjacency matrix (`Vec<Vec<i8>>`). This is O(n²) space but O(1) edge lookup, which is ideal for the small-to-medium networks (10-1000 nodes) this crate targets.

All graph traversals use BFS or DFS following only positive-weight edges. The min-cut implementation tries removing k edges in increasing order until disconnection—correct but exponential in the worst case. For networks under 50 nodes with moderate density, this is fast enough.

The crate is `#![no_std]` with only `alloc` as a dependency—suitable for embedded environments.

## Ecosystem connections

- **ternary-gauge** — gauge the robustness score over time as the network evolves
- **ternary-membrane** — diffusion across membranes is a form of network flow; resilience tells you if the flow network is robust
- **ternary-version** — version the network topology to track resilience changes over time

## Open questions

- **Efficient min-cut**: The current implementation is exponential for large networks. Stoer-Wagner or Karger's algorithm would give polynomial-time min-cut.
- **Weighted cascade**: The cascade model treats all non-zero edges equally. A weighted cascade (where strong edges propagate failure faster) would model real infrastructure better.
- **Adversarial edges in cascade**: Currently, adversarial edges contribute to the "dead neighbor" count. This is debatable—an adversarial link might *prevent* cascade propagation.

## Stats

| Metric | Value |
|--------|-------|
| Tests | 17 |
| Lines of code | 466 |
| Public API surface | 10 items |
| License | Apache-2.0 |
| Unsafe | 0 |
| `no_std` | Yes |

## Installation

```toml
[dependencies]
ternary-resilience = "0.1.0"
```

## License

Apache-2.0
