# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Kazoeru-kun (カウントくん) is a click counter application with a Rust-based microservices architecture. It features:
- **Backend**: Rust with Rocket framework (API server)
- **Frontend**: Rust with Yew framework (WebAssembly)
- **Storage**: Redis for session cache, MySQL for persistent data
- **Infrastructure**: Docker Compose for local development

## Architecture

### Service Architecture
The system consists of 5 Docker services:
1. **backend** (`back`): Rocket API server handling click counting and data persistence
2. **frontend**: Yew WebAssembly application served via Trunk
3. **redis**: Session-based click counter cache with TTL
4. **mysql** (`db`): Persistent storage for aggregated counts
5. **cron**: Periodic sync from Redis to MySQL (1-minute intervals)

### Data Flow
1. User clicks → Frontend calls `POST /session` → Redis increments counter
2. Cron job (every minute) → Calls `PUT /session` → Syncs Redis to MySQL
3. Frontend loads → Calls `GET /count_summary` → Displays total from MySQL

### API Endpoints
- `POST /session`: Increment click count for session (Redis)
- `PUT /session`: Sync Redis counts to MySQL (called by cron)
- `GET /count_summary?start_date=yyyy-mm-dd&end_date=yyyy-mm-dd`: Get aggregated counts

## Development Commands

### Start Development Environment
```bash
# Start all services
docker compose up -d

# Access application
# Frontend: http://localhost:3001
# Backend API: http://localhost:8881
# MySQL: localhost:7776
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
# Access MySQL
docker compose exec db mysql -u root -padmin_password KazoeruKun

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
- `DATABASE_URL`: MySQL connection string
- `ALLOWED_ORIGINS`: CORS allowed origins
- Redis connection is hardcoded to `redis:6379`

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