mod map;
mod robot;
mod world;

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::{
    io,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use map::Tile;
use robot::RobotType;
use world::World;

fn main() -> Result<(), io::Error> {
    // Configuration du terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let world = Arc::new(Mutex::new(World::new(70, 30)));
    let running = Arc::new(AtomicBool::new(true));

    let sim_world = Arc::clone(&world);
    let sim_running = Arc::clone(&running);
    let simulation = thread::spawn(move || {
        while sim_running.load(Ordering::Relaxed) {
            if let Ok(mut world) = sim_world.lock() {
                world.tick();
            }
            thread::sleep(Duration::from_millis(120));
        }
    });

    loop {
        terminal.draw(|f| {
            if let Ok(world) = world.lock() {
                ui(f, &world);
            }
        })?;

        // Gérer les entrées utilisateur : toute pression de touche quitte
        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && key.kind == event::KeyEventKind::Press
        {
            running.store(false, Ordering::Relaxed);
            break;
        }
    }

    let _ = simulation.join();

    // Restauration du terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, world: &World) {
    let size = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(size);

    let stats = Paragraph::new(world.summary()).block(
        Block::default()
            .title(" Resource Collection Simulation ")
            .borders(Borders::ALL),
    );
    f.render_widget(stats, chunks[0]);

    let mut lines = Vec::new();

    // Rendu de la carte et des robots
    for y in 0..world.map.height {
        let mut spans = Vec::new();
        for x in 0..world.map.width {
            // 1. Vérifier si un robot est sur la case
            if let Some(robot) = world.robot_at(x, y) {
                match robot.robot_type {
                    RobotType::Scout => {
                        spans.push(Span::styled("x", Style::default().fg(Color::Red)))
                    }
                    RobotType::Collector => {
                        spans.push(Span::styled("o", Style::default().fg(Color::Magenta)))
                    }
                }
            } else {
                // 2. Sinon, afficher l'élément de la carte avec les couleurs requises
                let (symbol, color) = match world.map.grid[y][x] {
                    Tile::Empty => (".", Color::DarkGray),
                    Tile::Obstacle => ("O", Color::LightCyan),
                    Tile::Energy(_) => ("E", Color::Green),
                    Tile::Crystal(_) => ("C", Color::LightMagenta),
                    Tile::Base => ("#", Color::LightGreen),
                };
                spans.push(Span::styled(symbol, Style::default().fg(color)));
            }
        }
        lines.push(Line::from(spans));
    }

    // Création du bloc d'interface
    let map_paragraph =
        Paragraph::new(lines).block(Block::default().title(" Carte ").borders(Borders::ALL));

    // Affichage au centre de l'écran (simplifié)
    let render_area = Rect::new(
        0,
        3,
        world.map.width as u16 + 2,
        world.map.height as u16 + 2,
    );

    // On s'assure de ne pas dessiner hors de l'écran
    if render_area.width <= size.width && render_area.bottom() <= size.height {
        f.render_widget(map_paragraph, render_area);
    }
}
