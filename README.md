# Wallet Backend - Rust Learning Project

A Rust backend application for a wallet system, built with Axum web framework and PostgreSQL database using SQLx.

## Features

- **Axum Web Framework**: Modern, async-first web framework built on Tokio
- **PostgreSQL Integration**: Type-safe database queries with SQLx
- **Docker Compose**: Easy database setup with PostgreSQL container
- **Environment Configuration**: Flexible configuration via environment variables
- **Health Checks**: Server and database health monitoring endpoints
- **CORS Support**: Cross-origin resource sharing enabled for web clients

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- [Docker](https://www.docker.com/get-started) and Docker Compose
- PostgreSQL client tools (optional, for direct database access)

## Quick Start

### 1. Clone and Setup

```bash
# Navigate to the project directory
cd wallet

# Copy the environment template
cp .env.example .env

# Edit .env if needed (defaults should work with docker-compose)
```

### 2. Start PostgreSQL Database

```bash
# Start PostgreSQL in a Docker container
docker-compose up -d

# Verify the database is running
docker-compose ps
```

The database will be available at `localhost:5432` with:
- Database: `wallet_db`
- Username: `wallet_user`
- Password: `wallet_password`

### 3. Build and Run the Application

```bash
# Build the project (downloads dependencies)
cargo build

# Run the application (migrations run automatically on startup)
cargo run
```

The server will start on `http://localhost:3000` (or the port specified in `.env`).

**Note:** Database migrations run automatically when the application starts. The migrations create the necessary tables (users, wallets, transactions) in your database.

### 4. Test the API

```bash
# Health check (no database required)
curl http://localhost:3000/health

# Database health check
curl http://localhost:3000/health/db

# Example database query endpoint
curl http://localhost:3000/api/example
```

## Project Structure

```
wallet/
├── src/
│   ├── main.rs          # Application entry point, Axum server setup
│   ├── config.rs        # Configuration management from environment variables
│   └── database.rs      # PostgreSQL connection pool setup and migrations
├── migrations/          # SQL migration files (run automatically on startup)
│   ├── 20240101000001_create_users_table.sql
│   ├── 20240101000002_create_transactions_table.sql
│   └── 20240101000003_create_wallets_table.sql
├── Cargo.toml           # Rust dependencies and project metadata
├── docker-compose.yml   # PostgreSQL service configuration
├── .env.example         # Environment variables template
└── README.md            # This file
```

## Key Rust Concepts Demonstrated

### Async/Await
The application uses Rust's async/await syntax for non-blocking I/O operations:
- Database queries are async
- HTTP request handling is async
- All handlers use `async fn`

### Error Handling
- `anyhow::Result` for application-level error handling
- `?` operator for error propagation
- Proper HTTP status codes in error responses

### Type Safety
- SQLx provides compile-time SQL checking
- Strong typing throughout the application
- Type-safe database queries

### Ownership and Borrowing
- `State` extractor in Axum handlers borrows the database pool
- Connection pools manage resource ownership efficiently

## Environment Variables

Create a `.env` file based on `.env.example`:

```env
DATABASE_URL=postgresql://wallet_user:wallet_password@localhost:5432/wallet_db
PORT=3000
HOST=0.0.0.0
RUST_LOG=debug
```

- `DATABASE_URL`: PostgreSQL connection string
- `PORT`: Server port (default: 3000)
- `HOST`: Server host address (default: 0.0.0.0)
- `RUST_LOG`: Logging level - debug, info, warn, error (default: info)

## API Endpoints

### `GET /health`
Basic health check endpoint. Returns 200 OK if the server is running.

**Response:**
```json
{
  "status": "ok",
  "message": "Wallet API is running"
}
```

### `GET /health/db`
Database connectivity check. Returns 200 OK if database is accessible, 503 otherwise.

**Response:**
```json
{
  "status": "ok",
  "database": "connected"
}
```

### `GET /api/example`
Example endpoint demonstrating database query execution.

**Response:**
```json
{
  "message": "Database query successful",
  "result": 42
}
```

## Database Management

### Database Migrations

Migrations run automatically when you start the application. The migration system:

- Creates tables: `users`, `wallets`, and `transactions`
- Tracks which migrations have been applied
- Only runs new migrations (idempotent)
- Uses transactions for safety (rolls back on failure)

#### Manual Migration Management

If you want to manage migrations manually using SQLx CLI:

```bash
# Install SQLx CLI (one-time setup)
cargo install sqlx-cli --no-default-features --features postgres

# Create a new migration
sqlx migrate add <migration_name>

# Run migrations manually
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

#### View Migration Status

Connect to the database and check the migrations table:

```bash
docker-compose exec postgres psql -U wallet_user -d wallet_db -c "SELECT * FROM _sqlx_migrations ORDER BY version;"
```

### Stop the Database
```bash
docker-compose down
```

### Stop and Remove Data
```bash
docker-compose down -v
```

### View Database Logs
```bash
docker-compose logs postgres
```

### Connect to Database Directly
```bash
docker-compose exec postgres psql -U wallet_user -d wallet_db
```

### View Database Schema

Once migrations have run, you can view the created tables:

```bash
docker-compose exec postgres psql -U wallet_user -d wallet_db -c "\dt"
```

To see table structures:

```bash
docker-compose exec postgres psql -U wallet_user -d wallet_db -c "\d users"
docker-compose exec postgres psql -U wallet_user -d wallet_db -c "\d wallets"
docker-compose exec postgres psql -U wallet_user -d wallet_db -c "\d transactions"
```

## Development Tips

### Running Tests
```bash
cargo test
```

### Formatting Code
```bash
cargo fmt
```

### Linting
```bash
cargo clippy
```

### Building for Release
```bash
cargo build --release
```

## Database Schema

The migrations create three main tables:

### Users Table
- `id` (UUID): Primary key
- `email` (VARCHAR): Unique user email
- `name` (VARCHAR): User's full name
- `password` (VARCHAR): FOR NOW PLAIN TEXT Hashed password (never store plain text!)
- `created_at`, `updated_at` (TIMESTAMPTZ): Timestamps

### Wallets Table
- `id` (UUID): Primary key
- `user_id` (UUID): Foreign key to users table
- `balance` (DECIMAL): Current wallet balance (non-negative)
- `currency` (VARCHAR): Currency code (default: USD)
- `created_at`, `updated_at` (TIMESTAMPTZ): Timestamps

### Transactions Table
- `id` (UUID): Primary key
- `user_id` (UUID): Foreign key to users table
- `transaction_type` (ENUM): deposit, withdrawal, or transfer
- `amount` (DECIMAL): Transaction amount (positive)
- `description` (TEXT): Optional transaction description
- `recipient_user_id` (UUID): For transfers, references recipient user
- `balance_after` (DECIMAL): Account balance after transaction
- `created_at` (TIMESTAMPTZ): Transaction timestamp

## Next Steps

Now that you have a working Rust backend with database connectivity and schema, you can:

1. **Add More Endpoints**: Create REST API endpoints for CRUD operations
   - User registration and authentication
   - Wallet balance queries
   - Transaction creation and history
2. **Implement Authentication**: Add JWT tokens or session-based auth
3. **Add Validation**: Use libraries like `validator` for input validation
4. **Error Handling**: Create custom error types for better error messages
5. **Testing**: Write unit and integration tests
6. **Add Business Logic**: Implement transaction processing, balance updates, etc.

## Learning Resources

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Axum Documentation](https://docs.rs/axum/)
- [SQLx Documentation](https://docs.rs/sqlx/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)

## Troubleshooting

### Database Connection Errors
- Ensure Docker Compose is running: `docker-compose ps`
- Check database logs: `docker-compose logs postgres`
- Verify DATABASE_URL in `.env` matches docker-compose.yml settings

### Port Already in Use
- Change the PORT in `.env` to a different value
- Or stop the service using port 3000

### Compilation Errors
- Ensure you have the latest Rust toolchain: `rustup update`
- Clean and rebuild: `cargo clean && cargo build`

## License

This is a learning project. Feel free to use it as a starting point for your own projects.

