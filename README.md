## Tetronix Backend (Rust + Actix-Web) [EN]

Real-time Tetris-style backend, designed to be robust, testable, and production-ready, can be used as-is with the public mobile frontend on my GitHub.

– Compatible app: https://github.com/Petitzenkuzu/Tetronix-MobileApp

### Technical Stack
- Language: Rust
- HTTP framework: Actix-Web
- WebSocket: actix-ws
- Database: PostgreSQL via SQLx (no heavy ORM)
- Observability: Prometheus (actix-web-prom) + system metrics (systemstat)
- Logging/Config: env_logger, dotenv

### Key Features
- GitHub OAuth authentication with cookie-based session
- Binary WebSocket game loop (anti-cheat, ACKs, explicit close codes)
- Rate limiting with token bucket algorithm (configurable per endpoint)
- Persistence: Users, Sessions, Games (actions stored as JSONB), leaderboard
- REST API for profile, stats, and replays
- Observability: `/metrics` endpoint (Prometheus)
- Tests: unit + integration with SQLx fixtures

### REST Endpoints
- `GET /auth/github?code=...&redirect_uri=...` → authenticates via GitHub and sets the session cookie
- `POST /auth/logout` → logs out and clears the cookie
- `GET /user` → returns the user bound to the session
- `GET /leaderboard` → top 3 scores
- `GET /game/stats` → stats for the current session user
- `GET /game/stats/{game_owner}` → stats for another user
- `GET /game/replay/{game_owner}` → replay data (actions JSONB)

### Game WebSocket
- URL: `GET /game/start` (WebSocket upgrade)
- Binary message (10 bytes):
  - `action_type: u8`
  - `piece: u8`
  - `timestamp: i64` (big-endian)
- `ActionType` (hex): Start 0x00, Rotate 0x01, Right 0x02, Left 0x03, Fall 0x04, HardDrop 0x05, ChangePiece 0x06, End 0x07, Ping 0xFF
- `PieceType` (hex): Cyan 0x00, Blue 0x01, Yellow 0x02, Orange 0x03, Purple 0x04, Green 0x05, Red 0x06, Void 0x07
- Server responses: binary ACK `0x00` after processing
- Close codes (`CloseCode`): Normal (1000) game ended, Policy (1008) auth/policy, Error (1011) server/game errors

### Data Model (PostgreSQL)
See `Schema.sql` for table definitions (`Users`, `Sessions`, `Games`) and indexes (e.g., `idx_users_best_score`).

### Configuration (environment)
```env
PORT=8080
DATABASE_URL=postgres://user:password@localhost:5432/tetronix
PRODUCTION=false/true
GITHUB_CLIENT_ID=...
GITHUB_CLIENT_SECRET=...
SESSION_SECRET_KEY=some-64-bytes-secret
# Optional (tests):
TEST_DATABASE_URL=postgres://user:password@localhost:5432/tetronix_test
```

### Local Setup
1) Create the database and apply `Schema.sql`
2) Export environment variables
3) Run the server
```bash
cargo run
# Server listens on 0.0.0.0:PORT (8080 by default)
```

### Observability (Prometheus/Grafana) – optional
- The backend exposes `/metrics` (Prometheus text format)
- Example Docker setup (Prometheus + Grafana) can be used to visualize:
  - `api_http_requests_total`, `api_http_requests_duration_seconds`
  - `cpu_usage`, `memory_usage`

### Tests
1) Run a dedicated Postgres and set `TEST_DATABASE_URL`
2) Execute:
```bash
# for free-tier DB otherwise just run cargo test
cargo test -- --test-threads=1
```

### Using with the mobile app
- This backend works directly with the public Tetronix mobile app.
- Frontend link: `[Tetronix](https://github.com/Petitzenkuzu/Tetronix-MobileApp)`
- Typical flow:
  1. GitHub auth on the client → session cookie
  2. Open WebSocket on `/game/start`
  3. Send binary actions (10 bytes), receive ACKs
  4. End of game → persist score + actions → leaderboard/replay

---

## Tetronix Backend (Rust + Actix-Web) [FR]

Backend de jeu temps réel type Tetris, conçu pour être robuste, testable et prêt pour la prod. Il peut être utilisé tel quel avec l'application mobile publié sur mon GitHub.

– application compatible: https://github.com/Petitzenkuzu/Tetronix-MobileApp

### Stack technique
- **Langage**: Rust
- **Framework HTTP**: Actix-Web
- **WebSocket**: actix-ws
- **DB**: PostgreSQL via SQLx (sans ORM lourd)
- **Observabilité**: Prometheus (actix-web-prom) + métriques CPU/Mémoire (systemstat)
- **Logs/Config**: env_logger, dotenv

### Principales fonctionnalités
- **Auth GitHub OAuth** avec gestion de session par cookie
- **WebSocket de jeu** binaire (anti-cheat, ACK, close codes explicites)
- **Rate limiting** avec algorithme token bucket (configurable par endpoint)
- **Persistance**: `Users`, `Sessions`, `Games`
- **API REST** pour profil, stats, replays
- **Observabilité**: endpoint `/metrics` (Prometheus)
- **Tests**: unitaires et d'intégration avec fixtures SQLx

### Endpoints REST
- `GET /auth/github?code=...&redirect_uri=...`
  - Authentifie via GitHub, set le cookie de session
- `POST /auth/logout`
  - Invalide la session, supprime le cookie
- `GET /user`
  - Retourne l’utilisateur lié à la session
- `GET /leaderboard`
  - Top 3 meilleurs scores
- `GET /game/stats`
  - Stats du joueur lié à la session
- `GET /game/stats/{game_owner}`
  - Stats d’un autre joueur
- `GET /game/replay/{game_owner}`
  - Données de replay (actions JSONB)

### WebSocket de jeu
- URL: `GET /game/start` (upgrade WebSocket)
- Message binaire (10 octets):
  - `action_type: u8`
  - `piece: u8`
  - `timestamp: i64` (big-endian)
- `ActionType` (hex): Start 0x00, Rotate 0x01, Right 0x02, Left 0x03, Fall 0x04, HardDrop 0x05, ChangePiece 0x06, End 0x07, Ping 0xFF
- `PieceType` (hex): Cyan 0x00, Blue 0x01, Yellow 0x02, Orange 0x03, Purple 0x04, Green 0x05, Red 0x06, Void 0x07
- Réponses serveur: ACK `0x00` binaire après traitement
- Fermetures (`CloseCode`): Normal (1000) fin de partie, Policy (1008) auth/policy, Error (1011) erreurs serveur/jeu

### Modèle de données (PostgreSQL)
- Voir Schema.sql

### Configuration (variables d’environnement)
```env
PORT=8080
DATABASE_URL=postgres://user:password@localhost:5432/tetronix
PRODUCTION=false/true
GITHUB_CLIENT_ID=...
GITHUB_CLIENT_SECRET=...
SESSION_SECRET_KEY=some-64-bytes-secret
# Optionnel (tests):
TEST_DATABASE_URL=postgres://user:password@localhost:5432/tetronix_test
```

### Démarrage local
1) Créer la base et appliquer `Schema.sql`
2) Exporter les variables d’env
3) Lancer le serveur
```bash
cargo run
# Serveur écoute sur 0.0.0.0:PORT (par défaut 8080)
```

### Observabilité (Prometheus/Grafana) – optionnel
- Le backend expose `/metrics` (Prometheus text format)
- Exemple de stack Docker (Prometheus + Grafana) disponible dans ce dépôt
- Une fois Prometheus configuré pour scrapper `http://<host>:8080/metrics`, vous pouvez visualiser:
  - `api_http_requests_total`, `api_http_requests_duration_seconds`
  - `cpu_usage`, `memory_usage`

### Tests
1) Lancer un Postgres de test et définir `TEST_DATABASE_URL`
2) Exécuter:
```bash
# si database free tier mettre threads=1 sinon juste cargo test
cargo test -- --test-threads=1
```

### Utiliser avec l'application mobile
- Ce backend est conçu pour fonctionner directement avec l'application mobile de Tetronix.
- Lien du frontend: insérez ici le lien GitHub de votre frontend (ex: `[Tetronix](https://github.com/Petitzenkuzu/Tetronix-MobileApp)`).
- Le flux standard:
  1. Auth GitHub côté frontend → cookie de session
  2. Ouverture du WebSocket sur `/game/start`
  3. Envoi des actions (binaire 10 octets), réception des ACK
  4. Fin de partie → persistance score + actions → leaderboard/replay


