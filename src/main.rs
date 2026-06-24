mod map;
mod robot;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::{io, time::Duration};

use map::{Map, Tile};
use robot::{RenderRobot, RobotType};

fn main() -> Result<(), io::Error> {
    // Configuration du terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialisation de l'état du jeu
    let map = Map::new(60, 20); // Taille de la carte

    // On place deux robots factices pour tester l'affichage
    let robots = vec![
        RenderRobot {
            x: 30,
            y: 10,
            r_type: RobotType::Scout,
        },
        RenderRobot {
            x: 31,
            y: 10,
            r_type: RobotType::Collector,
        },
    ];

    // Boucle principale de l'application
    loop {
        terminal.draw(|f| ui(f, &map, &robots))?;

        // Gérer les entrées utilisateur : toute pression de touche quitte
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // On quitte peu importe la touche (sauf si c'est un simple relâchement)
                if key.kind == event::KeyEventKind::Press {
                    break;
                }
            }
        }
    }

    // Restauration du terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, map: &Map, robots: &[RenderRobot]) {
    let size = f.size();
    let mut lines = Vec::new();

    // Rendu de la carte et des robots
    for y in 0..map.height {
        let mut spans = Vec::new();
        for x in 0..map.width {
            // 1. Vérifier si un robot est sur la case
            if let Some(robot) = robots.iter().find(|r| r.x == x && r.y == y) {
                match robot.r_type {
                    RobotType::Scout => {
                        spans.push(Span::styled("x", Style::default().fg(Color::Red)))
                    } // [cite: 136]
                    RobotType::Collector => {
                        spans.push(Span::styled("o", Style::default().fg(Color::Magenta)))
                    } // [cite: 137]
                }
            } else {
                // 2. Sinon, afficher l'élément de la carte avec les couleurs requises
                let (symbol, color) = match map.grid[y][x] {
                    Tile::Empty => (".", Color::DarkGray),
                    Tile::Obstacle => ("0", Color::LightCyan), // [cite: 132]
                    Tile::Energy(_) => ("E", Color::Green),    // [cite: 133]
                    Tile::Crystal(_) => ("C", Color::LightMagenta), // [cite: 134]
                    Tile::Base => ("#", Color::LightGreen),    // [cite: 135]
                };
                spans.push(Span::styled(symbol, Style::default().fg(color)));
            }
        }
        lines.push(Line::from(spans));
    }

    // Création du bloc d'interface
    let map_paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(" Simulation de Collecte de Ressources ")
            .borders(Borders::ALL),
    ); // [cite: 83]

    // Affichage au centre de l'écran (simplifié)
    let render_area = Rect::new(0, 0, map.width as u16 + 2, map.height as u16 + 2);

    // On s'assure de ne pas dessiner hors de l'écran
    if render_area.width <= size.width && render_area.height <= size.height {
        f.render_widget(map_paragraph, render_area);
    }
}
