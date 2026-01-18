# Environment Variables

## Backend (.env)

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | - |
| `ALLOWED_ORIGINS` | CORS allowed origins | - |
| `REDIS_URL` | Redis connection string | `redis://redis:6379` |
| `TARGET_VALUE` | Target value for weekly status calculation | `10` |
| `SYNC_JOB_INTERVAL_SECS` | Redis to PostgreSQL sync interval (seconds) | `60` |
| `STATUS_HISTORY_JOB_INTERVAL_SECS` | Status history update interval (seconds) | `86400` |

## Frontend (.env)

| Variable | Description | Default |
|----------|-------------|---------|
| `APP_API_ROOT` | Backend API URL | `http://localhost:8881` |

## Local Development Ports

| Service | Port |
|---------|------|
| Frontend | 3001 |
| Backend API | 8881 |
| PostgreSQL | 5432 |
| Redis | 6380 |
