use crate::graph::Graph;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};

#[derive(Debug)]
pub enum PathfindingAlgorithm {
    DepthFirst,
    BreadthFirst,
    Djikstra,
}

pub fn solve_graph<T>(graph: &Graph<T>, algo: &PathfindingAlgorithm) -> Option<Vec<usize>> {
    match algo {
        PathfindingAlgorithm::DepthFirst => dfs_iterative(graph),
        PathfindingAlgorithm::BreadthFirst => bfs(graph),
        PathfindingAlgorithm::Djikstra => dijkstra(graph),
    }
}

fn reconstruct_path(parent_map: &[Option<usize>], target: usize) -> Vec<usize> {
    let mut path = vec![target];
    let mut current = target;

    while let Some(parent) = parent_map[current] {
        path.push(parent);
        current = parent;
    }

    path.reverse();
    path
}

pub fn calculate_cost<T>(graph: &Graph<T>, solution: &[usize]) -> f32 {
    let mut tot_cost = 0.0;
    for i in 0..solution.len().saturating_sub(1) {
        let current = solution[i];
        let next = solution[i + 1];
        if let Some((_, weight)) = graph.get_vertices()[current]
            .get_neighbors()
            .iter()
            .find(|(idx, _)| *idx == next)
        {
            tot_cost += weight;
        }
    }
    tot_cost
}

fn dfs_iterative<T>(graph: &Graph<T>) -> Option<Vec<usize>> {
    let mut stack = vec![graph.start];
    
    let mut visited = vec![false; graph.get_vertices().len()];
    
    let mut parent_map = vec![None; graph.get_vertices().len()];

    while let Some(current) = stack.pop() {
        if current == graph.end {
            return Some(reconstruct_path(&parent_map, graph.end));
        }

        if !visited[current] {
            visited[current] = true;

            for (neighbor, _) in graph.get_vertices()[current].get_neighbors() {
                if !visited[*neighbor] {
                    parent_map[*neighbor] = Some(current);
                    stack.push(*neighbor);
                }
            }
        }
    }
    None
}

fn bfs<T>(graph: &Graph<T>) -> Option<Vec<usize>> {
    let mut queue = VecDeque::new();
    queue.push_back(graph.start);

    let mut visited = vec![false; graph.get_vertices().len()];
    let mut parent_map = vec![None; graph.get_vertices().len()];

    visited[graph.start] = true;

    while let Some(current) = queue.pop_front() {
        if current == graph.end {
            return Some(reconstruct_path(&parent_map, graph.end));
        }

        for (neighbor_idx, _) in graph.get_vertices()[current].get_neighbors() {
            if !visited[*neighbor_idx] {
                visited[*neighbor_idx] = true;
                parent_map[*neighbor_idx] = Some(current);
                queue.push_back(*neighbor_idx);
            }
        }
    }
    None
}

pub fn dijkstra<T>(graph: &Graph<T>) -> Option<Vec<usize>> {
    let mut dists = vec![f32::MAX; graph.get_vertices().len()];
    let mut parent_map = vec![None; graph.get_vertices().len()];
    let mut heap = BinaryHeap::new();

    dists[graph.start] = 0.0;
    heap.push(State {
        cost: 0.0,
        position: graph.start,
    });

    while let Some(State { cost, position }) = heap.pop() {
        if position == graph.end {
            return Some(reconstruct_path(&parent_map, graph.end));
        }

        if cost > dists[position] {
            continue;
        }

        for (neighbor_idx, weight) in graph.get_vertices()[position].get_neighbors() {
            let next_dist = cost + weight;
            if next_dist < dists[*neighbor_idx] {
                dists[*neighbor_idx] = next_dist;
                parent_map[*neighbor_idx] = Some(position);
                heap.push(State {
                    cost: next_dist,
                    position: *neighbor_idx,
                });
            }
        }
    }
    None
}

#[derive(Copy, Clone, PartialEq)]
struct State {
    cost: f32,
    position: usize,
}

impl Eq for State {}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
