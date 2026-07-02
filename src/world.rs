use std::collections::HashSet;
use std::sync::mpsc::{self, Receiver, Sender};

use crate::map::{Map, Tile};
use crate::robot::{Point, ResourceDiscovery, Robot, RobotMessage, RobotType};

#[derive(Debug)]

pub struct World {
    pub map: Map,
    pub robots: Vec<Robot>,
    pub known_resources: Vec<ResourceDiscovery>,
    pub known_obstacles: Vec<Point>,
    pub total_energy: u32,
    pub total_crystals: u32,
    tick: u64,
    sender: Sender<RobotMessage>,
    receiver: Receiver<RobotMessage>,
}

impl World {
    pub fn new(width: usize, height: usize) -> Self {
        let map = Map::new(width, height);
        let base = map.base;
        let (sender, receiver) = mpsc::channel();

        let spawn_offsets = [
            (0isize, 0isize),
            (1, 0),
            (-1, 0),
            (0, 1),
            (0, -1),
        ];
        let types = [
            RobotType::Scout,
            RobotType::Scout,
            RobotType::Collector,
            RobotType::Collector,
            RobotType::Collector,
        ];

        let mut robots = Vec::with_capacity(types.len());
        for (offset, robot_type) in spawn_offsets.iter().zip(types.iter()) {
            let x = (base.x as isize + offset.0).clamp(0, width as isize - 1) as usize;
            let y = (base.y as isize + offset.1).clamp(0, height as isize - 1) as usize;
            robots.push(Robot::new(x, y, *robot_type));
        }

        Self {
            map,
            robots,
            known_resources: Vec::new(),
            known_obstacles: Vec::new(),
            total_energy: 0,
            total_crystals: 0,
            tick: 0,
            sender,
            receiver,
        }
    }

    pub fn summary(&self) -> String {
        format!(
            " Énergie: {}  |  Cristaux: {}  |  Ressources connues: {}  |  Obstacles connus: {}  |  Tick: {}",
            self.total_energy,
            self.total_crystals,
            self.known_resources.len(),
            self.known_obstacles.len(),
            self.tick
        )
    }

    pub fn robot_at(&self, x: usize, y: usize) -> Option<&Robot> {
        self.robots.iter().find(|r| r.x == x && r.y == y)
    }

    /// Une étape de simulation : chaque robot agit, puis World lit
    /// sa boîte aux lettres (receiver) et met à jour l'état centralisé.
    pub fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);

        // Copie en lecture seule pour éviter un conflit d'emprunt avec
        // l'itération mutable sur self.robots juste en dessous.
        let known = self.known_resources.clone();
        
        let mut occupied: HashSet<Point> = self.robots.iter().map(|r| r.position()).collect();

        for robot in &mut self.robots {
            if let Some((energy, crystals)) =
                robot.update(&mut self.map, &known, &self.sender, &mut occupied)
            {
                self.total_energy = self.total_energy.saturating_add(energy);
                self.total_crystals = self.total_crystals.saturating_add(crystals);
            }
        }

        // Traite tous les messages reçus ce tick (non bloquant)
        while let Ok(msg) = self.receiver.try_recv() {
            match msg {
                RobotMessage::ResourceFound(discovery) => {
                    if !self
                        .known_resources
                        .iter()
                        .any(|r| r.position == discovery.position)
                    {
                        self.known_resources.push(discovery);
                    }
                }
                RobotMessage::ResourceDepleted(point) => {
                    self.known_resources.retain(|r| r.position != point);
                }
                RobotMessage::ObstacleFound(point) => {
                    if !self.known_obstacles.contains(&point) {
                        self.known_obstacles.push(point);
                    }
                }
            }
        }

        // Nettoyage de sécurité : retire toute ressource connue dont
        // la case est redevenue vide sur la carte (cas limite épuisement)
        self.known_resources.retain(|resource| {
            !matches!(
                self.map.grid[resource.position.y][resource.position.x],
                Tile::Empty
            )
        });
    }
}
