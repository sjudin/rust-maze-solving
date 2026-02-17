use image::{ImageError, ImageReader, RgbImage};
use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use rustc_hash::FxHashMap;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Coord {
    x: u32,
    y: u32,
}

impl Adjacent for Coord {
    type Neighbors = std::iter::Flatten<std::array::IntoIter<Option<Self>, 4>>;
    fn potential_neighbors(&self) -> Self::Neighbors {
        let x = self.x;
        let y = self.y;

        let arr = [
            y.checked_sub(1).map(|new_y| Coord { x, y: new_y }),
            y.checked_add(1).map(|new_y| Coord { x, y: new_y }),
            x.checked_sub(1).map(|new_x| Coord { x: new_x, y }),
            x.checked_add(1).map(|new_x| Coord { x: new_x, y }),
        ];

        return arr.into_iter().flatten();
    }
}

impl fmt::Display for Coord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl fmt::Display for Vertex<Coord> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "()")
    }
}

pub trait Adjacent: Sized {
    type Neighbors: Iterator<Item = Self>;
    fn potential_neighbors(&self) -> Self::Neighbors;
}

#[derive(Clone)]
pub struct Vertex<T> {
    pos: T,
    neighbors: Vec<(usize, f32)>,
}

impl<T> Vertex<T> {
    pub fn get_neighbors(&self) -> &Vec<(usize, f32)> {
        &self.neighbors
    }
}

pub struct Graph<T> {
    pub start: usize,
    pub end: usize,
    vertices: Vec<Vertex<T>>,
}

impl fmt::Display for Graph<Coord> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, vertex) in self.vertices.iter().enumerate() {
            writeln!(f, "[{i:2}]: {}, {:?}, ", vertex.pos, vertex.neighbors)?;
        }
        Ok(())
    }
}

impl<T> Graph<T> {
    pub fn get_vertices(&self) -> &Vec<Vertex<T>> {
        return &self.vertices;
    }
}

impl Graph<Coord> {
    pub fn from_png<P: AsRef<Path>>(path: P) -> Result<Self, ImageError> {
        let path = path.as_ref();
        let img = ImageReader::open(path)?.decode()?.into_rgb8();
        let mut vertices = create_vertices(&img);
        populate_vertex_neighbors(&mut vertices);
        reduce_vertex_count(&mut vertices);

        let boundary_vertices = find_boundary_vertices(&vertices, img.width(), img.height());
        if boundary_vertices.len() != 2 {
            println!(
                "Could not find definitive start/endpoints for this graph, using two at random"
            )
        }

        Ok(Self {
            start: boundary_vertices[0],
            end: boundary_vertices[1],
            vertices,
        })
    }

    pub fn draw_path<P: AsRef<Path>>(
        &self,
        path_indices: &[usize],
        original_image_path: P,
    ) -> Result<(), ImageError> {
        let mut img = image::open(original_image_path)?.into_rgb8();
        let highlight_color = image::Rgb([255, 0, 0]); // Bright Red

        // Iterate through the path in pairs (A -> B, B -> C)
        for window in path_indices.windows(2) {
            let start_node = &self.vertices[window[0]];
            let end_node = &self.vertices[window[1]];

            // Draw the junction points
            img.put_pixel(start_node.pos.x, start_node.pos.y, highlight_color);
            img.put_pixel(end_node.pos.x, end_node.pos.y, highlight_color);

            // Draw the line between them
            draw_line(
                &mut img,
                start_node.pos.x,
                start_node.pos.y,
                end_node.pos.x,
                end_node.pos.y,
                highlight_color,
            );
        }

        img.save("solved_maze.png")?;
        Ok(())
    }
}

/// Finds the entry/exit points by scanning the image boundaries.
pub fn find_boundary_vertices(vertices: &[Vertex<Coord>], width: u32, height: u32) -> Vec<usize> {
    let pos_map: HashMap<(u32, u32), usize> = vertices
        .iter()
        .enumerate()
        .map(|(i, v)| ((v.pos.x, v.pos.y), i))
        .collect();

    let mut boundary_indices = Vec::new();

    for x in 0..width {
        if let Some(&idx) = pos_map.get(&(x, 0)) {
            boundary_indices.push(idx);
        }
        if let Some(&idx) = pos_map.get(&(x, height - 1)) {
            boundary_indices.push(idx);
        }
    }

    for y in 1..height - 1 {
        if let Some(&idx) = pos_map.get(&(0, y)) {
            boundary_indices.push(idx);
        }
        if let Some(&idx) = pos_map.get(&(width - 1, y)) {
            boundary_indices.push(idx);
        }
    }

    boundary_indices
}

fn draw_line(img: &mut image::RgbImage, x0: u32, y0: u32, x1: u32, y1: u32, color: image::Rgb<u8>) {
    let mut x0 = x0 as i32;
    let mut y0 = y0 as i32;
    let x1 = x1 as i32;
    let y1 = y1 as i32;

    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if x0 >= 0 && x0 < img.width() as i32 && y0 >= 0 && y0 < img.height() as i32 {
            img.put_pixel(x0 as u32, y0 as u32, color);
        }
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn create_vertices(img: &RgbImage) -> Vec<Vertex<Coord>> {
    img.enumerate_pixels()
        .filter(|(_, _, pixel)| pixel.0[0] != 0)
        .map(|(x, y, _)| Vertex {
            pos: Coord { x, y },
            neighbors: Vec::with_capacity(4),
        })
        .collect()
}

fn populate_vertex_neighbors<T>(vertices: &mut Vec<Vertex<T>>)
where
    T: Adjacent + Clone + Eq + std::hash::Hash,
{
    let weight = 1.0; // TODO: Do not hardcode this here

    // FxHashMap is much faster for small keys like coordinates
    let pos_to_idx: FxHashMap<T, usize> = vertices
        .iter()
        .enumerate()
        .map(|(i, v)| (v.pos.clone(), i))
        .collect();

    for vertex in vertices.iter_mut() {
        for potential_neighbor in vertex.pos.potential_neighbors() {
            if let Some(&neighbor_idx) = pos_to_idx.get(&potential_neighbor) {
                vertex.neighbors.push((neighbor_idx, weight));
            }
        }
    }
}

fn reduce_vertex_count<T>(vertices: &mut Vec<Vertex<T>>) {
    for i in 0..vertices.len() {
        // If a vertex only connects two other vertexes then it is redundant
        // We remove it by connecting the two other vertices directly
        // Note that this would not work if we had a 2x2 square
        let vertex = &vertices[i];
        if vertex.neighbors.len() == 2 {
            // Connect the two neighbors together
            let (idx_a, weight_a) = vertex.neighbors[0];
            let (idx_b, weight_b) = vertex.neighbors[1];

            // Update the vertices to point at eachother
            if let Some(edge) = vertices[idx_a]
                .neighbors
                .iter_mut()
                .find(|(idx, _)| *idx == i)
            {
                *edge = (idx_b, weight_a + weight_b);
            }

            if let Some(edge) = vertices[idx_b]
                .neighbors
                .iter_mut()
                .find(|(idx, _)| *idx == i)
            {
                *edge = (idx_a, weight_b + weight_b);
            }

            vertices[i].neighbors.clear();
        }
    }
}
