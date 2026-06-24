pub enum RobotType {
    Scout,     // Éclaireur [cite: 99]
    Collector, // Collecteur [cite: 104]
}

// Structure simplifiée pour l'affichage initial
pub struct RenderRobot {
    pub x: usize,
    pub y: usize,
    pub r_type: RobotType,
}
