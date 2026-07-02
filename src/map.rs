use noise::{NoiseFn, Perlin};
use rand::RngExt;

use crate::robot::Point;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tile {
    Empty,
    Obstacle,
    Energy(u32),
    Crystal(u32),
    Base,
}

#[derive(Debug)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Vec<Tile>>,
    pub base: Point,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        let mut rng = rand::rng();
        let perlin = Perlin::new(rng.random_range(0..u32::MAX));
        let mut grid = vec![vec![Tile::Empty; width]; height];
        let base = Point {
            x: width / 2,
            y: height / 2,
        };

        for (y, row) in grid.iter_mut().enumerate() {
            for (x, tile) in row.iter_mut().enumerate() {
                let noise_value = perlin.get([x as f64 * 0.1, y as f64 * 0.1]);
                if noise_value > 0.25 {
                    *tile = Tile::Obstacle;
                }
            }
        }

        // Placement de la base centrale
        for dy in -1..=1 {
            for dx in -1..=1 {
                let x = base.x as isize + dx;
                let y = base.y as isize + dy;
                if x >= 0 && y >= 0 && (x as usize) < width && (y as usize) < height {
                    grid[y as usize][x as usize] = Tile::Empty;
                }
            }
        }
        grid[base.y][base.x] = Tile::Base;

        // Placement des ressources (Énergie et Cristaux)
        for _ in 0..24 {
            let rx = rng.random_range(0..width);
            let ry = rng.random_range(0..height);

            if grid[ry][rx] == Tile::Empty && !(rx == base.x && ry == base.y) {
                let quantity = rng.random_range(50..=200);
                if rng.random_bool(0.5) {
                    grid[ry][rx] = Tile::Energy(quantity);
                } else {
                    grid[ry][rx] = Tile::Crystal(quantity);
                }
            }
        }

        Map {
            width,
            height,
            grid,
            base,
        }
    }

    pub fn in_bounds(&self, x: isize, y: isize) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height
    }

    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        !matches!(self.grid[y][x], Tile::Obstacle)
    }

    pub fn take_resource_unit(&mut self, x: usize, y: usize) -> Option<&'static str> {
        match self.grid[y][x] {
            Tile::Energy(amount) if amount > 0 => {
                let remaining = amount.saturating_sub(1);
                self.grid[y][x] = if remaining == 0 {
                    Tile::Empty
                } else {
                    Tile::Energy(remaining)
                };
                Some("energy")
            }
            Tile::Crystal(amount) if amount > 0 => {
                let remaining = amount.saturating_sub(1);
                self.grid[y][x] = if remaining == 0 {
                    Tile::Empty
                } else {
                    Tile::Crystal(remaining)
                };
                Some("crystal")
            }
            _ => None,
        }
    }
}
