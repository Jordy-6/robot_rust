# Robot Rust — Simulation de Collecte de Ressources

Simulation temps réel en terminal (Ratatui) de robots autonomes qui explorent une carte générée procéduralement, découvrent des ressources et les ramènent à une base centrale.

## Aperçu

- Carte générée avec du bruit de Perlin (seed aléatoire à chaque lancement)
- 2 types de robots : **Scouts** (explorateurs) et **Collectors** (collecteurs)
- Pathfinding BFS avec contournement dynamique des autres robots
- Détection de collisions : deux robots ne peuvent pas occuper la même case
- Partage asynchrone des ressources ET des obstacles découverts entre robots
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

| Symbole | Élément   | Couleur       |
| ------- | --------- | ------------- |
| `O`     | Obstacle  | Cyan clair    |
| `E`     | Énergie   | Vert          |
| `C`     | Cristal   | Magenta clair |
| `#`     | Base      | Vert clair    |
| `x`     | Scout     | Rouge         |
| `o`     | Collector | Magenta       |
| `.`     | Case vide | Gris foncé    |

## Architecture du code

```
src/
├── main.rs   → Setup terminal, boucle de rendu, gestion des threads
├── map.rs    → Génération de carte (Perlin noise), gestion des tuiles
└── robot.rs  → Logique des robots (Scout/Collector), World, pathfinding
```

### Concepts clés

- **`Map`** : grille 2D de `Tile` (Empty, Obstacle, Energy, Crystal, Base). Le seed Perlin est tiré au hasard à chaque `Map::new`, donc chaque partie est différente
- **`Robot`** : position, type, mode (Exploring / ToResource / ReturningBase), cargo. Les robots spawn à des positions distinctes autour de la base pour éviter tout chevauchement initial
- **`World`** : orchestre la carte + les robots + les statistiques globales. Tient à jour `known_resources` et `known_obstacles` (agrégés depuis les messages)
- **Concurrence** : `Arc<Mutex<World>>` partagé entre le thread de simulation (tick toutes les 120ms) et le thread principal (rendu + input)
- **Communication inter-robots** : chaque robot envoie des `RobotMessage` (`ResourceFound`, `ResourceDepleted`, `ObstacleFound`) via un `mpsc::channel` plutôt que de modifier directement un état partagé ; `World` lit les messages reçus à chaque tick et met à jour les connaissances partagées
- **Collisions** : `World::tick` construit un `HashSet<Point>` des positions occupées avant la boucle des robots. Chaque déplacement passe par `try_move_to` (refuse si la case est déjà prise, met à jour le set). Le BFS filtre aussi les cases occupées pour trouver un chemin de contournement, tout en laissant la case cible atteignable (sinon la base occupée par un robot bloquerait tous les autres)

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

- [x] Génération de carte avec Perlin noise (seed aléatoire par run)
- [x] Scouts : exploration + découverte de ressources
- [x] Collectors : pathfinding BFS + collecte + retour base
- [x] Rendu Ratatui en couleur
- [x] Architecture concurrente (Arc/Mutex + threads)
- [x] Communication asynchrone via channels
- [x] Partage des obstacles découverts entre robots (`ObstacleFound`)
- [x] Gestion des collisions entre robots (snapshot d'occupation + BFS contournant)
- [ ] Documentation des fonctions (rustdoc)
- [ ] Tests unitaires
- [ ] Statistiques avancées (ressources/seconde, état détaillé par robot)
