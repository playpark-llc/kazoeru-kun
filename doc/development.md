# Development Guide

## Start Development Environment

```bash
# Start all services
docker compose up -d

# Access application
# Frontend: http://localhost:3001
# Backend API: http://localhost:8881
# PostgreSQL: localhost:5432 (user: postgres, db: kazoerukun)
# Redis: localhost:6380
```

## Backend Development (Rust/Rocket)

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

## Frontend Development (Rust/Yew)

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

## Database Management

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

## Build Configuration

- Backend uses cargo with sea-orm for database ORM
- Frontend uses Trunk for WASM bundling and serving
- Tailwind CSS for styling (compiled via npm scripts)
