# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Kazoeru-kun (カウントくん) is a click counter application with a Rust-based microservices architecture. It features:
- **Backend**: Rust with Rocket framework (API server with integrated cron jobs)
- **Frontend**: Rust with Yew framework (WebAssembly)
- **Storage**: Redis for session cache, PostgreSQL for persistent data
- **Infrastructure**: Docker Compose for local development

## Architecture

### Service Architecture
The system consists of 4 Docker services:
1. **backend** (`back`): Rocket API server handling click counting, data persistence, and scheduled tasks
2. **frontend**: Yew WebAssembly application served via Trunk
3. **redis**: Session-based click counter cache with TTL
4. **postgres** (`db`): Persistent storage for aggregated counts

### Data Flow
1. User clicks → Frontend calls `POST /session` → Redis increments counter
2. Backend cron job (every minute via tokio::spawn) → Syncs Redis to PostgreSQL
3. Frontend loads → Calls `GET /session_counter` → Displays total from PostgreSQL

### API Endpoints
- `POST /session`: Increment click count for session (Redis)
- `PUT /session`: Sync Redis counts to PostgreSQL (called by internal cron job)
- `GET /session_counter?start_date=yyyy-mm-dd&end_date=yyyy-mm-dd`: Get aggregated counts

## Development Commands

### Start Development Environment
```bash
# Start all services
docker compose up -d

# Access application
# Frontend: http://localhost:3001
# Backend API: http://localhost:8881
# PostgreSQL: localhost:5432
# Redis: localhost:6380
```

### Backend Development (Rust/Rocket)
```bash
# Connect to backend container
docker compose exec back sh

# Run backend server with hot reload
cargo watch -x 'run'

# Run migrations
cd migration && sea-orm-cli migrate up

# Generate entity files from database
cargo make gen-entity
```

### Frontend Development (Rust/Yew)
```bash
# Connect to frontend container
docker compose exec frontend sh

# Run development server (includes Tailwind CSS watch)
npm run dev

# Build for production
npm run prod

# Individual commands
npm run dev:css    # Watch Tailwind CSS changes
npm run dev:serve  # Run Trunk dev server
```

### Database Management
```bash
# Access PostgreSQL
docker compose exec db psql -U postgres -d kazoerukun

# Generate database documentation (automatic on db health check)
docker compose run --rm tbls
```

## Testing

```bash
# Backend tests
docker compose exec back cargo test

# Frontend tests
docker compose exec frontend cargo test
```

## Environment Variables

### Backend (.env)
- `DATABASE_URL`: PostgreSQL connection string
- `ALLOWED_ORIGINS`: CORS allowed origins
- `REDIS_URL`: Redis connection string (default: redis://redis:6379)
- `TARGET_VALUE`: Target value for weekly status calculation (default: 10)

### Frontend (.env)
- `APP_API_ROOT`: Backend API URL (default: http://localhost:8881)

## Key Implementation Details

### Session Management
- Sessions use cookies with expiration timestamps
- Redis keys have TTL matching cookie expiration
- New sessions get generated UUID and expiration

### Database Schema
- `session_counter`: Stores session_id and count
- `status_history`: Status tracking with date ranges (formerly count_summary)
- Migrations in `app/backend/migration/src/`

### CORS Configuration
- Configured via `ALLOWED_ORIGINS` environment variable
- Default allows `http://localhost:3001`

### Build Configuration
- Backend uses cargo with sea-orm for database ORM
- Frontend uses Trunk for WASM bundling and serving
- Tailwind CSS for styling (compiled via npm scripts)

### Cron Jobs (Integrated in Backend)
- **Sync Job**: Runs every 60 seconds to sync Redis counters to PostgreSQL
- **Status History Job**: Runs every 24 hours to update weekly status history
- Both jobs are managed via tokio::spawn with automatic retry logic