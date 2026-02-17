use std::env;
use std::time::Instant;

mod graph;
mod pathfinding;
use pathfinding::PathfindingAlgorithm;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tot_runtime = Instant::now();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: maze-solving <path-to-maze-png>");
        std::process::exit(1);
    }

    let filename = &args[1];
    let graph_create_now = Instant::now();
    let g = graph::Graph::from_png(filename)?;
    println!(
        "Graph creation took {}ms",
        graph_create_now.elapsed().as_millis()
    );

    let solvers = &[
        PathfindingAlgorithm::BreadthFirst,
        PathfindingAlgorithm::DepthFirst,
        PathfindingAlgorithm::Dijkstra,
    ];

    for solver in solvers {
        let graph_solve = Instant::now();
        let result = pathfinding::solve_graph(&g, solver).unwrap();
        println!(
            "Graph solved using {solver:?} took {}ms with cost {}",
            graph_solve.elapsed().as_millis(),
            pathfinding::calculate_cost(&g, &result)
        );

        if let PathfindingAlgorithm::Dijkstra = solver {
            g.draw_path(&result, filename)?;
        }
    }

    println!("Total runtime was {}ms", tot_runtime.elapsed().as_millis());
    Ok(())
}
