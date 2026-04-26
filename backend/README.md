# CDATA Backend

Centriolar Damage Accumulation Theory of Aging backend implementation for LongevityCommon project.

## Overview

This is the production-grade Axum backend for the CDATA subproject, implementing:
- Full CRUD operations for all domain entities
- PostgreSQL database with SQLx
- RESTful API endpoints
- Proper error handling and tracing
- Docker containerization

## Architecture

### Domain Entities

1. **Parameter** - Quantitative parameters from PARAMETERS.md with γ_i = 0 default
2. **Counter** - MCOA counter registry (α_i, β_i, γ_i kinetics)
3. **CdataCounter** - CDATA-specific extension (Hayflick limit, D_crit, rescue half-life)
4. **Tissue** - Tissue types and weights for MCOA
5. **TransplantArm** - HSC transplant arm tracking
6. **SensitivityAnalysis** - Sobol sensitivity storage
7. **McoaComputation** - L_tissue computation results
8. **FclcData** - Privacy budget (ε) and secure aggregation
9. **BiosenseData** - Raw EEG/HRV upload (NO χ_Ze computation)
10. **ScaffoldCounter** - Telomere/MitoROS/EpigeneticDrift/Proteostasis time-series
11. **HapData** - Hepatic+affective joint biomarkers
12. **OntogenesisMilestone** - 0-25 year developmental milestones

### Database

PostgreSQL with:
- UUID primary keys
- Automatic timestamps (created_at, updated_at)
- Proper indices and constraints
- Enum types for status fields
- JSONB for flexible data storage

## Getting Started

### Prerequisites

- Rust 1.75+ (2021 edition)
- PostgreSQL 15+
- Docker (optional)

### Environment Setup

1. Copy `.env.example` to `.env`:
```bash
cp .env.example .env
```

2. Update `.env` with your configuration:
```bash
ENVIRONMENT=development
PORT=3003
DATABASE_URL=postgres://cn:cn@localhost/cdata_db
LOG_LEVEL=debug
```

### Database Setup

1. Create database:
```bash
createdb cdata_db
```

2. Run migrations:
```bash
sqlx database create
sqlx migrate run
```

### Running Locally

```bash
cargo run
```

Server will start at `http://localhost:3003`

### API Endpoints

- `GET /health` - Health check
- `GET /parameters` - List all parameters
- `POST /parameters` - Create new parameter
- `GET /parameters/:id` - Get parameter by ID
- `PUT /parameters/:id` - Update parameter
- `DELETE /parameters/:id` - Delete parameter

Similar endpoints for all other entities.

### Running with Docker

```bash
docker build -t cdata-backend .
docker run -p 3003:3003 --env-file .env cdata-backend
```

## Development

### Testing

```bash
cargo test
```

### Database Migrations

Create new migration:
```bash
sqlx migrate add -r descriptive_name
```

Run migrations:
```bash
sqlx migrate run
```

### Code Style

- Follow Rust 2021 edition conventions
- Use `tracing` for logging
- Proper error handling with `thiserror`
- Input validation with `validator`
- SQLx for type-safe database queries

## Deployment

### Production Considerations

1. Set `ENVIRONMENT=production`
2. Use proper database connection pooling
3. Configure CORS appropriately
4. Enable request/response logging
5. Set up monitoring and alerting

### Health Checks

- `GET /health` - Basic service health
- Database connectivity is validated at startup
- Graceful shutdown on SIGTERM/SIGINT

## License

MIT