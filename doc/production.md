# Production Environment

## Infrastructure

| Component | Service | Region | Notes |
|-----------|---------|--------|-------|
| Frontend | Cloudflare Pages | Edge (Global) | WebAssembly app |
| Backend | Fly.io | nrt (Tokyo) | Rocket API server |
| PostgreSQL | Neon | - | Serverless PostgreSQL |
| Redis | Upstash | - | Serverless Redis |

## Deployment

### Automatic Deployment (GitHub Actions)

| Trigger | Action |
|---------|--------|
| PR to main | Frontend preview deploy (Cloudflare Pages) |
| Merge to main | Backend (Fly.io) + Frontend (Cloudflare Pages) production deploy |

### Configuration Files

| Service | Config File |
|---------|-------------|
| Fly.io | `app/backend/fly.toml` |
| Cloudflare Pages | `app/frontend/yew-app/wrangler.toml` |
| GitHub Actions | `.github/workflows/deploy.yml` |

## Fly.io Backend Configuration

- App: `kazoeru-kun`
- Region: `nrt` (Tokyo)
- VM: 1 shared CPU, 1GB memory
- Auto-scaling: 0-N machines (auto stop/start)
- Health check: `/health/liveness` every 15s

## Required Secrets (GitHub Actions)

| Secret | Description |
|--------|-------------|
| `FLY_API_TOKEN` | Fly.io deployment token |
| `CLOUDFLARE_API_TOKEN` | Cloudflare API token |
| `CLOUDFLARE_ACCOUNT_ID` | Cloudflare account ID |
| `APP_API_ROOT` | Production backend API URL |

## Production Environment Variables (Fly.io)

Set via `fly secrets set`:
- `DATABASE_URL` - Neon PostgreSQL connection string
- `REDIS_URL` - Upstash Redis connection string
- `ALLOWED_ORIGINS` - Production frontend URL
- `TARGET_VALUE` - Weekly status target value
