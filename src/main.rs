extern crate rand;

use std::cmp::min;
use std::cmp::max;

/// A graph contains 64 nodes which represent squares on the chess board.
///
/// Each node is connected to between 2 and 8 others via edges which represent valid moves made by
/// a knight (1 square in one direction followed by 2 squares in another or vice versa).
///
/// Ants traverse this graph in an attempt to find a valid knight's tour. Pheromone is layed along
/// each edge so that subsequent ants can learn from those who came before.
struct Graph {
    nodes: Vec<Node>,
}

impl Graph {
    fn new(initial_pheromone: f32) -> Self {
        let mut nodes = Vec::with_capacity(64);

        for i in 0..64 {
            let mut node = Node::new(i);

            // Find the minimum square of nodes which contain all of the possible moves from
            // the current node. A knight can only move 2 squares in any direction so there is
            // no point searching for moves beyond that boundary.
            let min_x = max(0, node.x - 2);
            let max_x = min(7, node.x + 2);
            let min_y = max(0, node.y - 2);
            let max_y = min(7, node.y + 2);

            for x in min_x..max_x + 1 {
                for y in min_y..max_y + 1 {
                    // Use pythagoras (a^2 + b^2 = c^2) to determine if this is a valid knigth's move.
                    // A knight's move is two sides of a right-angled triangle where a = 1 and b = 2.
                    // This means that c must be 1^2 + 2^2 = 1 + 4 = 5 to form a valid move.
                    if 5 == ((node.x - x).pow(2) + (node.y - y).pow(2)) {
                        let edge = Edge::new(initial_pheromone, y * 8 + x);
                        node.edges.push(edge);
                    }
                }
            }

            nodes.push(node);
        }

        Graph {nodes: nodes}
    }

    fn node(&self, index: &i8) -> &Node {
        &self.nodes[*index as usize]
    }

    fn node_mut(&mut self, index: &i8) -> &mut Node {
        &mut self.nodes[*index as usize]
    }

    /// Evapourates previously laid pheromones at a specified rate.
    ///
    /// This is useful so that fruitless paths are eventually forgotten by the ants which
    /// strengthens the use of fruitful paths.
    fn evaporate_pheromones(&mut self, rate: &f32) {
        for node in &mut self.nodes {
            for edge in &mut node.edges {
                edge.pheromone *= 1.0 - rate
            }
        }
    }
}

/// A single node in the graph representing a square on the chess board.
struct Node {
    x: i8,
    y: i8,
    edges: Vec<Edge>,
}

impl Node {
    fn new(index: i8) -> Self {
        Node {x: index % 8, y: index / 8, edges: Vec::with_capacity(8)}
    }

    fn edge(&self, index: &i8) -> &Edge {
        &self.edges[*index as usize]
    }

    fn edge_mut(&mut self, index: &i8) -> &mut Edge {
        &mut self.edges[*index as usize]
    }
}

/// A single connection betweeon one node and another.
///
/// Each edge represents a valid knight's move from the owning node to a target node.
struct Edge {
    pheromone: f32,
    target: i8,
}

impl Edge {
    fn new(pheromone: f32, target: i8) -> Self {
        Edge {pheromone: pheromone, target: target}
    }
}

/// Ants traverse the graph in an attempt to find knight's tours.
struct Ant {
    start: i8,
    current: i8,
    tabu: Vec<i8>,
    moves: Vec<i8>,
}

impl Ant {
    fn new(start: i8) -> Self {
        let mut tabu = Vec::with_capacity(64);

        tabu.push(start);

        Ant {start: start, current: start, tabu: tabu, moves: Vec::with_capacity(64)}
    }

    fn tour(&mut self, graph: &Graph) -> bool {
        let pheromone_strength_exponent: f32 = 1.0;

        loop {

            let current_node = graph.node(&self.current);
            let mut pks = Vec::with_capacity(8);
            let mut pk_sum: f32 = 0.0;

            // Check each edge to see if it is an available mode (we have not followed it before).
            // If it is, calculate its pheromone strenth with which to weight the probability of
            // following this edge versus others.
            for (k, edge) in current_node.edges.iter().enumerate() {
                if !self.tabu.contains(&edge.target) {
                    let pheromone_strength = edge.pheromone.powf(pheromone_strength_exponent);
                    pk_sum += pheromone_strength;
                    pks.push((k as i8, pheromone_strength));
                }
            }

            // If there are no pks then there are no more edges to try.
            if pks.is_empty() {
                break;
            }

            // Calculate the probability of choosing each edge k based on the pheromone level Pk.
            let ps = pks.iter().map(|&pk| (pk.0, pk.1 / pk_sum)).collect::<Vec<_>>();

            let mut x = rand::random::<f32>();
            let mut k = 0;

            // FIXME: Why can I not use "for (mv, p) in &ps" here?
            for p in &ps {
                x -= p.1;
                if x <= 0.0 {
                    k = p.0;
                    break;
                }
            }

            let next = current_node.edge(&k).target;

            // Move to the new node.
            self.current = next;

            // Prevent visiting the current node again.
            self.tabu.push(self.current);

            // Record the move.
            self.moves.push(k);
        }

        self.moves.len() == 63
    }

    fn lay_pheromone(&self, graph: &mut Graph) {
        let pheromone_update_rate: f32 = 1.0;
        let num_moves = self.moves.len();
        let mut current = self.start;

        for (i, k) in self.moves.iter().enumerate() {

            let delta_pheromone = pheromone_update_rate * ((num_moves - i) as f32 / (63 - i) as f32);
            let edge = graph.node_mut(&current).edge_mut(k);

            edge.pheromone += delta_pheromone;
            current = edge.target;
        }
    }
}

struct TourFinder {
    graph: Graph,
    complete: u32,
    incomplete: u32,
    p_evap_rate: f32
}

impl TourFinder {
    fn new(p_initial_level: f32, p_evap_rate: f32) -> Self {
        TourFinder {
            graph: Graph::new(p_initial_level),
            complete: 0,
            incomplete: 0,
            p_evap_rate: p_evap_rate
        }
    }

    fn run(&mut self, cycles: u32) {
        for _ in 0..cycles {
            self.cycle()
        }
    }

    fn cycle(&mut self) {
        // Place an ant on each node.
        let mut ants = Vec::with_capacity(64);

        for i in 0..64 {
            ants.push(Ant::new(i));
        }

        // Have each ant attempt a tour.
        // TODO: Do this concurrently - pheromones are not laid until all ants have finished.
        for ant in &mut ants {
            if ant.tour(&self.graph) {
                self.complete += 1;
            } else {
                self.incomplete += 1;
            }
        }

        // Now all ants have finished an attempt, have them lay pheromones.
        for ant in &ants {
            ant.lay_pheromone(&mut self.graph);
        }

        // Evapourate pheromones so that weak routes are forgotten over time.
        self.graph.evaporate_pheromones(&self.p_evap_rate);
    }
}


fn main() {
    let mut tour_finder = TourFinder::new(0.000001, 0.25);

    tour_finder.run(10000);

    println!("Complete: {}, Incomplete: {}", tour_finder.complete, tour_finder.incomplete);
}
