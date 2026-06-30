# Robot Rust — Simulation de Collecte de Ressources

Simulation temps réel en terminal (Ratatui) de robots autonomes qui explorent une carte générée procéduralement, découvrent des ressources et les ramènent à une base centrale.

## Aperçu

- Carte générée avec du bruit de Perlin (obstacles)
- 2 types de robots : **Scouts** (explorateurs) et **Collectors** (collecteurs)
- Pathfinding BFS pour la navigation
- Rendu terminal en couleur via Ratatui
- Architecture concurrente (thread de simulation séparé du thread de rendu)

## Prérequis

- Rust et Cargo installés ([rustup.rs](https://rustup.rs))

## Lancer le projet

```bash
git clone https://github.com/Jordy-6/robot_rust.git
cd robot_rust
cargo run
```

Quitter la simulation : n'importe quelle touche du clavier.

Build optimisé (recommandé si la simulation rame) :
```bash
cargo run --release
```

## Légende visuelle

| Symbole | Élément       | Couleur       |
|---------|---------------|---------------|
| `O`     | Obstacle      | Cyan clair    |
| `E`     | Énergie       | Vert          |
| `C`     | Cristal       | Magenta clair |
| `#`     | Base          | Vert clair    |
| `x`     | Scout         | Rouge         |
| `o`     | Collector     | Magenta       |
| `.`     | Case vide     | Gris foncé    |

## Architecture du code

```
src/
├── main.rs   → Setup terminal, boucle de rendu, gestion des threads
├── map.rs    → Génération de carte (Perlin noise), gestion des tuiles
└── robot.rs  → Logique des robots (Scout/Collector), World, pathfinding
```

### Concepts clés

- **`Map`** : grille 2D de `Tile` (Empty, Obstacle, Energy, Crystal, Base)
- **`Robot`** : position, type, mode (Exploring / ToResource / ReturningBase), cargo
- **`World`** : orchestre la carte + les robots + les statistiques globales
- **Concurrence** : `Arc<Mutex<World>>` partagé entre le thread de simulation (tick toutes les 120ms) et le thread principal (rendu + input)

## Développement — workflow Git

Une branche par fonctionnalité, merge sur `main` une fois la fonctionnalité testée :

```bash
git checkout main
git pull origin main
git checkout -b feature/nom-de-la-feature
# ... travail ...
git push origin feature/nom-de-la-feature
# Pull Request sur GitHub vers main
```

### Commandes utiles avant de commit

```bash
cargo build          # vérifie que ça compile
cargo clippy          # linter, repère les mauvaises pratiques
cargo fmt              # formate le code automatiquement
cargo test             # lance les tests unitaires
```

## État actuel / Roadmap

- [x] Génération de carte avec Perlin noise
- [x] Scouts : exploration + découverte de ressources
- [x] Collectors : pathfinding BFS + collecte + retour base
- [x] Rendu Ratatui en couleur
- [x] Architecture concurrente (Arc/Mutex + threads)
- [ ] Communication asynchrone via channels (actuellement mémoire partagée)
- [ ] Gestion des collisions entre robots
- [ ] Scouts mémorisent les obstacles découverts
- [ ] Documentation des fonctions (rustdoc)
- [ ] Tests unitaires
- [ ] Statistiques avancées (ressources/seconde, état détaillé par robot)
