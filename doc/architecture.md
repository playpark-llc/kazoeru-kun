# Architecture

## Service Architecture

The system consists of 4 Docker services:

| Service | Description | Port |
|---------|-------------|------|
| backend (`back`) | Rocket API server with integrated cron jobs | 8881 |
| frontend (`front`) | Yew WebAssembly application served via Trunk | 3001 |
| redis | Session-based click counter cache with TTL | 6380 |
| postgres (`db`) | Persistent storage (PostgreSQL 16 Alpine) | 5432 |

## Data Flow

1. User clicks → Frontend calls `POST /session` → Redis increments counter
2. Backend internal cron job (every 60 seconds via tokio::spawn) → Syncs Redis to PostgreSQL
3. Frontend loads → Calls `GET /session_counter` → Displays total from PostgreSQL

## API Endpoints

### Session

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/session` | Increment click count for session (Redis) |
| PUT | `/session` | Sync Redis counts to PostgreSQL (internal cron job) |
| GET | `/session_counter?start_date=yyyy-mm-dd&end_date=yyyy-mm-dd` | Get aggregated counts |

### Status History

| Method | Endpoint | Description |
|--------|----------|-------------|
| PUT | `/status_history` | Update weekly status history (internal cron job) |
| GET | `/status_history?limit=<n>&offset=<n>&start_date=yyyy-mm-dd&end_date=yyyy-mm-dd` | Get status history |

### Health Check

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | System health check (PostgreSQL and Redis connectivity) |
| GET | `/health/liveness` | Kubernetes liveness probe |
| GET | `/health/readiness` | Kubernetes readiness probe |

## Session Management

- Sessions use cookies with expiration timestamps
- Redis keys have TTL matching cookie expiration
- New sessions get generated UUID and expiration

## Database Schema

- `session_counter`: Stores session_id and count
- `status_history`: Status tracking with date ranges
- Migrations in `app/backend/migration/src/`
- See [Database Definition](./KazoeruKun/README.md) for details

## CORS Configuration

- Configured via `ALLOWED_ORIGINS` environment variable
- Default allows `http://localhost:3001`

## Cron Jobs (Integrated in Backend)

### Sync Job
- Syncs Redis counters to PostgreSQL
- Interval: `SYNC_JOB_INTERVAL_SECS` env variable (default: 60 seconds)
- Implementation: `app/backend/src/cron/jobs/sync.rs`

### Status History Job
- Updates weekly status history
- Interval: `STATUS_HISTORY_JOB_INTERVAL_SECS` env variable (default: 86400 seconds/24 hours)
- Implementation: `app/backend/src/cron/jobs/status_history.rs`

Both jobs are managed via tokio::spawn with automatic retry logic.
Configuration in `app/backend/src/cron/config.rs`.
Minimum interval validation: 1 second (prevents excessive load).
