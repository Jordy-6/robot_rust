use std::collections::{HashMap, HashSet, VecDeque};

use rand::RngExt;

use crate::map::{Map, Tile};

use std::sync::mpsc::Sender;

#[derive(Clone, Debug)]
pub enum RobotMessage {
    ResourceFound(ResourceDiscovery),
    ResourceDepleted(Point),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RobotType {
    Scout,
    Collector,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResourceKind {
    Energy,
    Crystal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ResourceDiscovery {
    pub position: Point,
    pub kind: ResourceKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollectorMode {
    Exploring,
    ToResource(Point),
    ReturningBase,
}

#[derive(Clone, Debug)]
pub struct Robot {
    pub x: usize,
    pub y: usize,
    pub robot_type: RobotType,
    pub carrying_energy: u32,
    pub carrying_crystal: u32,
    pub mode: CollectorMode,
    pub visited: HashSet<Point>,
}

impl Robot {
    pub fn new(x: usize, y: usize, robot_type: RobotType) -> Self {
        let mut visited = HashSet::new();
        visited.insert(Point { x, y });

        Self {
            x,
            y,
            robot_type,
            carrying_energy: 0,
            carrying_crystal: 0,
            mode: CollectorMode::Exploring,
            visited,
        }
    }

    pub fn position(&self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }

    pub fn cargo_total(&self) -> u32 {
        self.carrying_energy + self.carrying_crystal
    }

    pub fn is_at_base(&self, map: &Map) -> bool {
        self.x == map.base.x && self.y == map.base.y
    }

    pub fn update(&mut self, map: &mut Map, known_resources: &mut Vec<ResourceDiscovery>) -> Option<(u32, u32)> {
        self.scan(map, known_resources);

        let delivered = match self.robot_type {
            RobotType::Scout => {
                self.step_scout(map, known_resources);
                None
            }
            RobotType::Collector => self.step_collector(map, known_resources),
        };

        self.scan(map, known_resources);
        delivered
    }

    fn scan(&mut self, map: &Map, known_resources: &mut Vec<ResourceDiscovery>) {
        for dy in -1..=1 {
            for dx in -1..=1 {
                let x = self.x as isize + dx;
                let y = self.y as isize + dy;
                if !map.in_bounds(x, y) {
                    continue;
                }

                let x = x as usize;
                let y = y as usize;
                match map.grid[y][x] {
                    Tile::Energy(_) => Self::remember_resource(
                        known_resources,
                        ResourceDiscovery {
                            position: Point { x, y },
                            kind: ResourceKind::Energy,
                        },
                    ),
                    Tile::Crystal(_) => Self::remember_resource(
                        known_resources,
                        ResourceDiscovery {
                            position: Point { x, y },
                            kind: ResourceKind::Crystal,
                        },
                    ),
                    _ => {}
                }
            }
        }
    }

    fn remember_resource(known_resources: &mut Vec<ResourceDiscovery>, discovery: ResourceDiscovery) {
        if !known_resources.iter().any(|item| item.position == discovery.position) {
            known_resources.push(discovery);
        }
    }

    fn step_scout(&mut self, map: &Map, known_resources: &mut Vec<ResourceDiscovery>) {
        let mut rng = rand::rng();
        let mut candidates = self.walkable_neighbors(map);
        if candidates.is_empty() {
            return;
        }

        candidates.sort_by_key(|point| self.visited.contains(point));
        let preferred: Vec<Point> = candidates
            .iter()
            .copied()
            .filter(|point| !self.visited.contains(point))
            .collect();

        let next = if !preferred.is_empty() {
            preferred[rng.random_range(0..preferred.len())]
        } else {
            candidates[rng.random_range(0..candidates.len())]
        };

        self.move_to(next);
        self.visited.insert(next);
        self.scan(map, known_resources);
    }

    fn step_collector(&mut self, map: &mut Map, known_resources: &mut Vec<ResourceDiscovery>) -> Option<(u32, u32)> {
        if self.cargo_total() >= 12 {
            self.mode = CollectorMode::ReturningBase;
        }

        if self.is_at_base(map) {
            if self.cargo_total() > 0 {
                let delivered = (self.carrying_energy, self.carrying_crystal);
                self.carrying_energy = 0;
                self.carrying_crystal = 0;
                self.mode = CollectorMode::Exploring;
                return Some(delivered);
            }
            self.mode = CollectorMode::Exploring;
        }

        match self.mode {
            CollectorMode::ReturningBase => {
                if let Some(next) = self.next_step_towards(map, map.base) {
                    self.move_to(next);
                }
            }
            CollectorMode::ToResource(target) => {
                if let Some(resource_index) = known_resources.iter().position(|entry| entry.position == target) {
                    if self.position() == target {
                        if let Some(kind) = map.take_resource_unit(target.x, target.y) {
                            match kind {
                                "energy" => self.carrying_energy += 1,
                                "crystal" => self.carrying_crystal += 1,
                                _ => {}
                            }
                        } else {
                            known_resources.remove(resource_index);
                            self.mode = CollectorMode::Exploring;
                        }
                    } else if let Some(next) = self.next_step_towards(map, target) {
                        self.move_to(next);
                    } else {
                        self.mode = CollectorMode::Exploring;
                    }
                } else {
                    self.mode = CollectorMode::Exploring;
                }
            }
            CollectorMode::Exploring => {
                if let Some(target) = self.choose_best_known_resource(map, known_resources) {
                    self.mode = CollectorMode::ToResource(target);
                    if let Some(next) = self.next_step_towards(map, target) {
                        self.move_to(next);
                    }
                } else if self.is_at_base(map) {
                    self.patrol_near_base(map);
                } else if let Some(next) = self.next_step_towards(map, map.base) {
                    self.move_to(next);
                }
            }
        }

        None
    }

    fn patrol_near_base(&mut self, map: &Map) {
        let mut rng = rand::rng();
        let mut options = self.walkable_neighbors(map);
        if options.is_empty() {
            return;
        }

        options.sort_by_key(|point| self.visited.contains(point));
        let next = options[rng.random_range(0..options.len())];
        self.move_to(next);
        self.visited.insert(next);
    }

    fn choose_best_known_resource(&self, map: &Map, known_resources: &[ResourceDiscovery]) -> Option<Point> {
        known_resources
            .iter()
            .filter(|resource| !matches!(map.grid[resource.position.y][resource.position.x], Tile::Empty))
            .min_by_key(|resource| self.manhattan(resource.position))
            .map(|resource| resource.position)
    }

    fn manhattan(&self, target: Point) -> usize {
        self.x.abs_diff(target.x) + self.y.abs_diff(target.y)
    }

    fn walkable_neighbors(&self, map: &Map) -> Vec<Point> {
        let mut neighbors = Vec::new();
        for (dx, dy) in [(0isize, -1isize), (1, 0), (0, 1), (-1, 0)] {
            let x = self.x as isize + dx;
            let y = self.y as isize + dy;
            if map.in_bounds(x, y) {
                let point = Point {
                    x: x as usize,
                    y: y as usize,
                };
                if map.is_walkable(point.x, point.y) {
                    neighbors.push(point);
                }
            }
        }
        neighbors
    }

    fn next_step_towards(&self, map: &Map, goal: Point) -> Option<Point> {
        if self.position() == goal {
            return None;
        }

        let start = self.position();
        let mut queue = VecDeque::new();
        let mut came_from = HashMap::<Point, Point>::new();
        queue.push_back(start);
        came_from.insert(start, start);

        while let Some(current) = queue.pop_front() {
            if current == goal {
                break;
            }

            for neighbor in neighbors_of(current, map) {
                if came_from.contains_key(&neighbor) {
                    continue;
                }
                came_from.insert(neighbor, current);
                queue.push_back(neighbor);
            }
        }

        if !came_from.contains_key(&goal) {
            return None;
        }

        let mut step = goal;
        while let Some(previous) = came_from.get(&step).copied() {
            if previous == start {
                return Some(step);
            }
            if previous == step {
                break;
            }
            step = previous;
        }

        None
    }

    fn move_to(&mut self, next: Point) {
        self.x = next.x;
        self.y = next.y;
    }
}

fn neighbors_of(point: Point, map: &Map) -> Vec<Point> {
    let mut neighbors = Vec::new();
    for (dx, dy) in [(0isize, -1isize), (1, 0), (0, 1), (-1, 0)] {
        let x = point.x as isize + dx;
        let y = point.y as isize + dy;
        if map.in_bounds(x, y) {
            let candidate = Point {
                x: x as usize,
                y: y as usize,
            };
            if map.is_walkable(candidate.x, candidate.y) {
                neighbors.push(candidate);
            }
        }
    }
    neighbors
}

#[derive(Debug)]
pub struct World {
    pub map: Map,
    pub robots: Vec<Robot>,
    pub known_resources: Vec<ResourceDiscovery>,
    pub total_energy: u32,
    pub total_crystals: u32,
    tick: u64,
}

impl World {
    pub fn new(width: usize, height: usize) -> Self {
        let map = Map::new(width, height);
        let base = map.base;

        let robots = vec![
            Robot::new(base.x, base.y, RobotType::Scout),
            Robot::new(base.x, base.y, RobotType::Scout),
            Robot::new(base.x, base.y, RobotType::Collector),
            Robot::new(base.x, base.y, RobotType::Collector),
            Robot::new(base.x, base.y, RobotType::Collector),
        ];

        Self {
            map,
            robots,
            known_resources: Vec::new(),
            total_energy: 0,
            total_crystals: 0,
            tick: 0,
        }
    }

    pub fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        let map = &mut self.map;
        let known_resources = &mut self.known_resources;

        for robot in &mut self.robots {
            if let Some((energy, crystals)) = robot.update(map, known_resources) {
                self.total_energy = self.total_energy.saturating_add(energy);
                self.total_crystals = self.total_crystals.saturating_add(crystals);
            }
        }

        self.known_resources.retain(|resource| {
            !matches!(self.map.grid[resource.position.y][resource.position.x], Tile::Empty)
        });
    }

    pub fn robot_at(&self, x: usize, y: usize) -> Option<&Robot> {
        self.robots.iter().find(|robot| robot.x == x && robot.y == y)
    }

    pub fn summary(&self) -> String {
        format!(
            "Energy collected: {}   Crystals collected: {}   Discovered resources: {}",
            self.total_energy,
            self.total_crystals,
            self.known_resources.len(),
        )
    }
}
