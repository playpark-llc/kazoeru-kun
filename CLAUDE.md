# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Kazoeru-kun (カゾエルくん) is a click counter application with a Rust-based microservices architecture.

- **Backend**: Rust with Rocket framework
- **Frontend**: Rust with Yew framework (WebAssembly)
- **Storage**: Redis (session cache) + PostgreSQL (persistent data)

## Documentation

| Document | Contents |
|----------|----------|
| [doc/architecture.md](./doc/architecture.md) | Service architecture, API endpoints, data flow, cron jobs |
| [doc/development.md](./doc/development.md) | Development commands, testing, build configuration |
| [doc/environment.md](./doc/environment.md) | Environment variables |
| [doc/production.md](./doc/production.md) | Production infrastructure, deployment |
| [doc/KazoeruKun/README.md](./doc/KazoeruKun/README.md) | Database schema definition |

## Quick Start

```bash
docker compose up -d
# Frontend: http://localhost:3001
# Backend: http://localhost:8881
```

## Rules

### Code Style
- Follow existing patterns in the codebase
- Use sea-orm for database operations
- Use Rocket framework conventions for API endpoints

### Git Workflow
- Main branch: `dev`
- Create feature branches for all work
- PR to `main` triggers preview deploy, merge triggers production deploy

### API Design
- All API endpoints defined in `app/backend/src/routes/`
- Use appropriate HTTP methods (GET for read, POST for create, PUT for update)
- Return proper status codes and JSON responses

### Testing
- Run `cargo test` before committing backend changes
- Ensure Docker services are running for integration tests
