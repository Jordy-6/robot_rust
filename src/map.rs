use noise::{NoiseFn, Perlin};

#[derive(Clone, Copy, PartialEq)]
pub enum Tile {
    Empty,
    Obstacle,
    Energy(u32),
    Crystal(u32),
    Base,
}

pub struct Map {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Vec<Tile>>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        let perlin = Perlin::new(42);
        let mut grid = vec![vec![Tile::Empty; width]; height];

        // Génération des obstacles via le bruit de Perlin
        for y in 0..height {
            for x in 0..width {
                let noise_value = perlin.get([x as f64 * 0.1, y as f64 * 0.1]);
                if noise_value > 0.2 {
                    grid[y][x] = Tile::Obstacle;
                }
            }
        }

        // Placement de la base centrale
        let base_x = width / 2;
        let base_y = height / 2;
        grid[base_y][base_x] = Tile::Base;

        // Placement des ressources (Énergie et Cristaux)
        // L'API de rand 0.10 est beaucoup plus directe
        for _ in 0..20 {
            let rx = rand::random_range(0..width);
            let ry = rand::random_range(0..height);

            if grid[ry][rx] == Tile::Empty {
                let quantity = rand::random_range(50..=200);
                if rand::random_bool(0.5) {
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
        }
    }
}
