# Tetronix Backend

Real-time Tetris backend written in **Rust** with **Actix-Web**. Server-authoritative game engine, binary WebSocket protocol, JWT authentication, and token-bucket rate limiting.

Compatible frontend: [Tetronix Mobile App](https://github.com/Petitzenkuzu/Tetronix-MobileApp)

---

## Table of Contents

- [Stack](#stack)
- [Architecture](#architecture)
- [Security](#security)
  - [Authentication — GitHub OAuth + JWT](#authentication--github-oauth--jwt)
  - [Session Cookie Hardening](#session-cookie-hardening)
  - [Rate Limiting — Token Bucket](#rate-limiting--token-bucket)
  - [Anti-Cheat — Server-Authoritative Game Engine](#anti-cheat--server-authoritative-game-engine)
  - [Action Sequencing & Replay Integrity](#action-sequencing--replay-integrity)
  - [WebSocket Input Validation](#websocket-input-validation)
- [WebSocket Protocol](#websocket-protocol)
- [REST API](#rest-api)
- [Game Engine](#game-engine)
- [Data Model](#data-model)
- [Observability](#observability)
- [Configuration](#configuration)
- [Local Setup](#local-setup)
- [Running Tests](#running-tests)
- [Docker](#docker)
- [CI/CD](#cicd)

---

## Stack

| Layer | Technology |
|---|---|
| Language | Rust (stable) |
| HTTP framework | Actix-Web 4 |
| WebSocket | actix-ws |
| Database | PostgreSQL via SQLx (no ORM) |
| Auth | GitHub OAuth 2.0 + HS256 JWT |
| Observability | Prometheus (`actix-web-prom`) + `systemstat` |
| Logging | `tracing`, `env_logger`, `dotenv` |
| Testing | Built-in test suite, `mockito` for OAuth mocking |

---

## Architecture

The project follows a strict layered architecture with trait-based dependency injection throughout — no `dyn Trait` at the service layer, zero-cost monomorphization end-to-end.

```
handlers/         → HTTP layer (routes, WebSocket upgrade)
  ↓
services/         → Business logic (auth, user, game)
  ↓
repository/       → Raw SQLx queries (no ORM)
  ↓
PostgreSQL

game_logic/       → Self-contained game engine (runs on a dedicated OS thread)
middleware/       → Auth (JWT check) + Rate limiter (token bucket)
models/           → Domain types, errors, app state
errors/           → Typed error hierarchy: RepositoryError → ServicesError → AppError
```

`AppState` is a generic struct monomorphized at the type level:

```rust
pub type ConcreteAppState = AppState<
    AuthService<UserRepository>,
    GameService<GameRepository>,
    UserService<UserRepository>,
>;
```

All service and repository traits have mock-friendly interfaces, enabling fast unit tests with a real Postgres test database without spinning up the full HTTP stack.

---

## Security

### Authentication — GitHub OAuth + JWT

Authentication is done exclusively via **GitHub OAuth 2.0**. There are no passwords stored anywhere in the system.

Flow:
1. Client sends the GitHub authorization `code` and `redirect_uri` to `GET /auth/github`
2. Backend exchanges the code for a GitHub access token
3. Backend fetches the authenticated user's GitHub profile
4. On first login, a user record is auto-created
5. A **JWT** is signed with **HMAC-SHA256** (`HS256`) using a `SESSION_SECRET_KEY` and set as an **HTTP-only cookie**
6. Every subsequent request to protected routes is validated by `AuthMiddleware` before reaching any handler

Token lifetime is **7 days**. The JWT payload contains the username as a claim. Verification is done via the `jsonwebtoken` crate. Any invalid or expired token results in a `401 Unauthorized` before the handler is ever invoked.

```
Request → RateLimiterMiddleware → AuthMiddleware → Handler
                                       ↓
                              Reads auth_token cookie
                                       ↓
                              verify_jwt() [HS256]
                                       ↓
                         Insert AuthenticatedUser in extensions
                                       ↓
                         Handler extracts via FromRequest
```

The `AuthenticatedUser` extractor is implemented via Actix-Web's `FromRequest` trait. It reads from the request extensions (set by the middleware), never re-parsing the JWT at the handler level.

### Session Cookie Hardening

Cookies are configured with the following flags:

| Flag | Value | Effect |
|---|---|---|
| `HttpOnly` | `true` | Inaccessible to JavaScript — prevents XSS token theft |
| `SameSite` | `Lax` | Mitigates CSRF in most cross-site scenarios |
| `Secure` | `true` in production | Cookie only sent over HTTPS |
| `Path` | `/` | Scoped to the entire application |

The `Secure` flag is controlled by the `PRODUCTION` environment variable, so local development works over HTTP without friction.

### Rate Limiting — Token Bucket

A custom `RateLimiterTransform` middleware wraps the **entire application** (runs before authentication). It implements the **token bucket algorithm** per client IP address:

| Parameter | Value |
|---|---|
| Capacity | 100 tokens |
| Refill rate | 1 token / second |
| Scope | Per IP address |

State is held in an `Arc<Mutex<HashMap<IpAddr, TokenBucket>>>`. When a bucket is exhausted, the server responds with:

```
HTTP 429 Too Many Requests
Retry-After: 10
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
```

### Anti-Cheat — Server-Authoritative Game Engine

The core anti-cheat design principle: **the client never computes game state**. The client is a pure input terminal.

- The game engine runs **entirely on the server**, in a dedicated OS thread decoupled from the async runtime
- The client only sends action opcodes: `Right`, `Left`, `Rotate`, `HardDrop` — nothing else
- The server computes gravity, piece placement, line clears, scoring, and level progression on every tick
- The server serializes the full game `State` (grid, current piece, next piece, score, level, lines) and broadcasts it to the client on every piece placement
- The client **cannot fabricate** a score or board state — any attempt is simply ignored; the server's state is always authoritative

This makes score manipulation impossible: there is no client-submitted score. The final score is computed exclusively by the server engine and written directly to the database at game end.

### Action Sequencing & Replay Integrity

Every `ClientAction` carries a monotonically increasing `id` (u32):

```
[action_type: u8][id: u32 big-endian]  →  5 bytes total
```

The engine validates the sequence on every tick:

- **Out-of-order actions** are silently dropped
- **Non-consecutive IDs** (gap detected) trigger a `ServerResponse::MissingAction` response, asking the client to retransmit
- The `last_processed_action` field in `State` is always sent back to the client, so both sides maintain a synchronized cursor

Every action processed during a game session is persisted as **JSONB** in the `games` table alongside the final score. This enables:

- **Server-side replay verification**: any game can be fully reconstructed from its action log
- **Forensic auditing**: suspicious high scores can be replayed and validated deterministically

### WebSocket Input Validation

The WebSocket handler enforces a strict binary protocol. Any message that does not conform to the expected 5-byte format is rejected immediately:

- Wrong message length → WebSocket closed with `CloseCode::Policy` (1008)
- Unknown `action_type` byte → handled gracefully (not a crash vector)
- Text frames are not accepted

---

## WebSocket Protocol

**Endpoint:** `GET /game/start` (WebSocket upgrade, requires valid `auth_token` cookie)

### Client → Server (binary, exactly 5 bytes)

| Bytes | Field | Type | Description |
|---|---|---|---|
| 0 | `action_type` | `u8` | `0x00`=Right, `0x01`=Left, `0x02`=Rotate, `0x03`=HardDrop |
| 1–4 | `id` | `u32 BE` | Monotonically increasing sequence number |

### Server → Client (JSON, tagged enum)

All server messages are JSON objects with a `type` field and a `data` field:

```json
{ "type": "State", "data": "{ ... serialized State ... }" }
```

| `type` | Trigger | `data` content |
|---|---|---|
| `Start` | Game session opened | Initial serialized `State` |
| `Ack` | Action processed, no piece placement | `{ "id": <last_processed_id> }` |
| `State` | Piece placed or line cleared | Full serialized `State` |
| `End` | Game over | Final serialized `State` |
| `MissingAction` | Gap in action IDs | Last known good ID |
| `InternalServerError` | Engine error | Error description |

### WebSocket Close Codes

| Code | Meaning |
|---|---|
| `1000` (Normal) | Game ended cleanly |
| `1008` (Policy) | Invalid message format or auth failure |
| `1011` (Error) | Internal server or game engine error |

---

## REST API

All endpoints except `/auth/*` require a valid `auth_token` cookie (enforced by `AuthMiddleware`).

| Method | Path | Description |
|---|---|---|
| `GET` | `/auth/github?code=...&redirect_uri=...` | GitHub OAuth login, sets session cookie |
| `POST` | `/auth/logout` | Invalidates session, clears cookie |
| `GET` | `/user` | Returns the authenticated user's profile |
| `GET` | `/leaderboard` | Top 3 scores across all users |
| `GET` | `/game/stats` | Stats for the authenticated user |
| `GET` | `/game/stats/{game_owner}` | Stats for any user by name |
| `GET` | `/game/replay/{game_owner}` | Full action log for replay |
| `GET` | `/metrics` | Prometheus metrics endpoint |

Any request to an unknown route returns `401 Unauthorized` (not `404`) — no route enumeration.

---

## Game Engine

The engine runs in a dedicated `std::thread` (intentionally outside the async runtime) at a fixed **~60 FPS** tick rate (`sleep(16ms)`). Communication with the WebSocket handler is via two channels:

- `Receiver<ClientAction>` — inbound actions from the async WebSocket handler
- `UnboundedSender<ServerResponse>` — outbound state updates to the async handler

Each tick:
1. Drain all pending `ClientAction`s from the channel
2. Validate sequence IDs; send `MissingAction` if a gap is detected
3. Apply valid actions (move, rotate, hard drop) against the authoritative `State`
4. Run `process_fall()` — apply gravity using the **classic Tetris formula**: `1000 × 0.8^level` ms per row
5. On piece placement: clear lines, update score, check level-up, spawn next piece
6. On game over (spawn position blocked): send `End` response, persist game to DB

**Scoring:**

| Lines cleared | Points |
|---|---|
| 1 | `40 × level` |
| 2 | `100 × level` |
| 3 | `300 × level` |
| 4 (Tetris) | `1200 × level` |
| Hard drop | `rows_dropped × level × 10` |

Level increases every 10 lines. All 7 standard tetrominoes (I, J, L, O, S, T, Z) are supported with full rotation.

---

## Data Model

See [`Schema.sql`](./Schema.sql) for the full definition.

| Table | Key columns |
|---|---|
| `users` | `name` (PK), `best_score`, `created_at` |
| `games` | `game_owner` (PK → FK users), `score`, `level`, `lines`, `game_actions` (JSONB) |

The `game_actions` JSONB column stores the full ordered action log — the foundation of the replay and audit system.

Index `idx_users_best_score` on `(best_score DESC)` powers the leaderboard query.

---

## Observability

The backend exposes a Prometheus-compatible `/metrics` endpoint (auto-wired by `actix-web-prom`), plus custom gauges via `systemstat`:

| Metric | Description |
|---|---|
| `api_http_requests_total` | Request count by method, path, status |
| `api_http_requests_duration_seconds` | Latency histogram |
| `cpu_usage` | Host CPU utilization |
| `memory_usage` | Host memory utilization |

---

## Configuration

```env
PORT=8080
DATABASE_URL=postgres://user:password@host:5432/tetronix
PRODUCTION=false          # true → Secure cookie, stricter settings
GITHUB_CLIENT_ID=...
GITHUB_CLIENT_SECRET=...
SESSION_SECRET_KEY=...    # ≥ 32 bytes, used for HS256 JWT signing

# Required for running the test suite
TEST_DATABASE_URL=postgres://user:password@localhost:5432/test_db
```

**Never commit `.env` files or real credentials to version control.** Rotate any key that has been exposed.

---

## Local Setup

**Prerequisites:** Rust (stable), PostgreSQL

```bash
# 1. Copy and fill in environment variables
cp .env.example .env

# 2. Start the server
cargo run
# Listening on 0.0.0.0:8080
```

---

## Running Tests

The test suite covers unit tests (services, repositories) and integration tests (full HTTP handlers) against a real Postgres instance.

**Using Docker (recommended):**

```bash
# Linux/macOS
./start_test_db.sh
export TEST_DATABASE_URL=postgres://user:password@localhost:5432/test_db
cargo test
```

---

## Docker

Multi-stage build for a minimal production image:

```
Stage 1 (rust:1.92-slim)   → compile release binary with dependency caching
Stage 2 (debian:trixie-slim) → copy binary only, install libssl3 + ca-certificates
```

The final image contains only the binary and its runtime dependencies — no Rust toolchain, no source code.

```bash
# Build
docker build -t tetronix-backend .

# Run (requires a populated .env)
docker compose up tetronix-backend
```

The `docker-compose.yml` defines:
- `tetronix-backend` — the application container
- `test-db` — a Postgres 15 instance with healthcheck

```bash
# Start only the test database
docker compose up -d test-db
```

---

## CI/CD

GitHub Actions workflow (`.github/workflows/prod.yml`) runs on every push to `main`:

1. Spin up a Postgres 15 service container
2. Initialize the database schema (`psql -f Schema.sql`)
3. `cargo build`
4. `cargo test`
5. Build Docker image and push to Docker Hub (`docker/build-push-action`)

Required repository secrets: `DOCKER_USERNAME`, `DOCKER_PASSWORD`.
