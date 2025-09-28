# Deployment Guide for Kazoeru-kun

This guide covers deploying Kazoeru-kun to production using Fly.io with Supabase (PostgreSQL) and Upstash (Redis).

## Architecture

```
┌─────────────────┐
│   Fly.io App    │
│  (kazoeru-kun)  │
│                 │
│  Backend(Rust)  │
│  Frontend(Yew)  │
└────────┬────────┘
         │
    ┌────┴────┐
    ▼         ▼
┌────────┐ ┌─────────┐
│Supabase│ │ Upstash │
│  (DB)  │ │ (Redis) │
└────────┘ └─────────┘
```

## Prerequisites

1. **Supabase Account**
   - Sign up at https://supabase.com
   - Create a new project (Tokyo region recommended)

2. **Upstash Account**
   - Sign up at https://console.upstash.com
   - Create Redis database (Tokyo region)

3. **Fly.io Account**
   - Sign up at https://fly.io
   - Install Fly CLI: `curl -L https://fly.io/install.sh | sh`
   - Authenticate: `fly auth login`

## Step 1: Configure Supabase

1. Create a new Supabase project
2. Go to Settings → Database
3. Copy the connection string (use Pooling connection on port 6432)
4. Run migrations:
   ```bash
   cd app/backend/migration
   DATABASE_URL="postgresql://postgres:[PASSWORD]@db.[PROJECT_ID].supabase.co:5432/postgres" \
   sea-orm-cli migrate up
   ```

## Step 2: Configure Upstash Redis

1. Create a new Redis database in Upstash
2. Select Tokyo region (ap-northeast-1)
3. Copy the Redis URL from the dashboard
4. Test connection:
   ```bash
   redis-cli -u "redis://default:xxx@xxx.upstash.io:6379" ping
   ```

## Step 3: Configure Environment Variables

1. Copy the environment template:
   ```bash
   cp .env.production.example .env.production
   ```

2. Edit `.env.production` with your values:
   ```env
   SUPABASE_DATABASE_URL=postgresql://postgres:[PASSWORD]@db.[PROJECT_ID].supabase.co:6432/postgres
   UPSTASH_REDIS_URL=redis://default:[PASSWORD]@[ENDPOINT].upstash.io:6379
   ALLOWED_ORIGINS=https://kazoeru-kun.fly.dev
   TARGET_VALUE=10
   SYNC_JOB_INTERVAL_SECS=60
   STATUS_HISTORY_JOB_INTERVAL_SECS=86400
   ```

## Step 4: Deploy to Fly.io

1. **Initialize Fly App** (if not already done):
   ```bash
   fly launch --name kazoeru-kun --region nrt --no-deploy
   ```

2. **Set Secrets**:
   ```bash
   # Database
   fly secrets set SUPABASE_DATABASE_URL="postgresql://..."

   # Redis
   fly secrets set UPSTASH_REDIS_URL="redis://default:..."

   # Application settings
   fly secrets set ALLOWED_ORIGINS="https://kazoeru-kun.fly.dev"
   fly secrets set TARGET_VALUE="10"
   fly secrets set SYNC_JOB_INTERVAL_SECS="60"
   fly secrets set STATUS_HISTORY_JOB_INTERVAL_SECS="86400"
   ```

3. **Deploy**:
   ```bash
   fly deploy
   ```

4. **Verify Deployment**:
   ```bash
   fly status
   fly logs
   ```

5. **Access Application**:
   - Open https://kazoeru-kun.fly.dev

## Local Production Testing

To test the production configuration locally:

1. Set up `.env.production` with your external service credentials

2. Build and run with production docker-compose:
   ```bash
   docker compose -f docker-compose.prod.yml build
   docker compose -f docker-compose.prod.yml up
   ```

3. Access at http://localhost:3001

### Environment Variable Priority

**Local Testing (docker-compose)**:
- Uses `.env.production` file
- Rocket settings in `environment:` section
- Application settings from `env_file:`

**Fly.io Production**:
- Uses `fly secrets` (no .env files)
- Environment variables injected at runtime
- More secure (secrets never in Docker image)

## Monitoring

### Fly.io Monitoring
```bash
# View logs
fly logs

# Check app status
fly status

# View metrics
fly dashboard
```

### Supabase Monitoring
- Dashboard: https://supabase.com/dashboard/project/[PROJECT_ID]
- Check database usage and connections

### Upstash Monitoring
- Dashboard: https://console.upstash.com
- Monitor command usage (500K/month free tier)

## Troubleshooting

### Database Connection Issues
- Verify you're using the pooling connection (port 6432)
- Check if migrations have been run
- Verify credentials in Fly secrets

### Redis Connection Issues
- Check Upstash dashboard for service status
- Verify Redis URL format
- Check monthly command limit

### Deployment Failures
```bash
# View detailed logs
fly logs -n 100

# SSH into container
fly ssh console

# Check secrets
fly secrets list
```

## Cost Management

| Service | Free Tier | Notes |
|---------|-----------|-------|
| Supabase | 500MB storage, 50K MAU | Sufficient for small apps |
| Upstash | 256MB, 500K commands/month | Monitor command usage |
| Fly.io | ~$5/month for small instance | Scales with usage |

**Estimated Monthly Cost**: $0-5

## Backup and Recovery

### Database Backup (Supabase)
- Automatic daily backups on free tier
- Access via Supabase dashboard

### Redis Backup (Upstash)
- Use REST API for data export if needed
- Consider implementing periodic cache warming

## Security Checklist

- [ ] All secrets set via `fly secrets`, not in code
- [ ] CORS configured correctly for production domain
- [ ] Database using pooling connection
- [ ] HTTPS enforced on Fly.io
- [ ] Environment variables properly scoped

## Updating the Application

1. Make changes locally
2. Test with production config
3. Deploy:
   ```bash
   fly deploy
   ```
4. Monitor deployment:
   ```bash
   fly logs --tail
   ```

## Rolling Back

If deployment fails:
```bash
# View releases
fly releases

# Rollback to previous version
fly rollback [VERSION]
```