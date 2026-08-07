# TeamFlow Lite API

A small collaboration API built with Rust, Axum, SQLx, and SQLite. The current API focuses on task CRUD and is intended as a clean foundation for future team, comment, and reminder features.

## Requirements

- Rust and Cargo
- SQLite 3 for the command-line examples
- `curl`
- Optional: `jq` for readable JSON output

## Setup

The application reads its database URL from `.env`:

```env
DATABASE_URL=sqlite:teamflow.db
```

Start the API:

```bash
cargo run
```

The server listens at `http://localhost:4000`. SQLx creates the database when needed and runs pending migrations during startup.

Check that it is running:

```bash
curl -i http://localhost:4000/health
```

## API documentation

Swagger UI is available while the server is running:

```text
http://localhost:4000/docs
```

Download the OpenAPI document for a frontend, Postman, Insomnia, or a client generator:

```bash
curl --fail http://localhost:4000/api-docs/openapi.json \
  --output openapi.json
```

The generated document is the API contract. Regenerate it after changing request models, response models, or routes.

## Database and migrations

Migrations live in `migrations/`. By default, pending migrations run automatically when the application starts because `main.rs` calls `sqlx::migrate!()`.

You can also apply migrations manually with the SQLx CLI. This is useful during development and is often preferable as a separate deployment step in production.

Inspect applied migrations:

```bash
sqlite3 teamflow.db \
  "SELECT version, description, success FROM _sqlx_migrations ORDER BY version;"
```

Inspect the database schema:

```bash
sqlite3 teamflow.db ".schema"
```

Open an interactive SQLite shell:

```bash
sqlite3 teamflow.db
```

Useful commands inside that shell:

```sql
.tables
.schema tasks
.headers on
PRAGMA table_info(table_name); --display structure
.mode column
SELECT * FROM tasks;
.quit
```

### Run migrations manually with the SQLx CLI

Install the SQLite-enabled CLI once:

```bash
cargo install sqlx-cli --no-default-features --features sqlite
```

Create the configured database if it does not exist, inspect migration status, and apply pending migrations:

```bash
sqlx database create
sqlx migrate info
sqlx migrate run
```

Create a new migration:

```bash
sqlx migrate add describe_the_change
```

Revert the most recently applied migration during local development:

```bash
sqlx migrate revert
```

Run these commands from the project directory so SQLx reads `DATABASE_URL` from `.env`. Avoid editing a migration after it has been applied; add another migration instead.

The automatic and manual workflows can coexist: running `sqlx migrate run` before `cargo run` means the application simply finds no pending migrations. If deployment should manage migrations exclusively, remove the `sqlx::migrate!().run(&db)` call from `main.rs`.

## Local test data

Task creation requires an existing team because SQLite foreign-key checks are enabled. Add a local team:

```bash
sqlite3 teamflow.db \
  "INSERT OR IGNORE INTO teams (id, name) VALUES ('demo-team', 'Demo Team');"
```

An owner is optional. To test assignment, add a user and membership:

```bash
sqlite3 teamflow.db \
  "INSERT OR IGNORE INTO users (id, name, email) VALUES ('demo-user', 'Demo User', 'demo@example.com');"

sqlite3 teamflow.db \
  "INSERT OR IGNORE INTO team_members (team_id, user_id) VALUES ('demo-team', 'demo-user');"
```

## Test the API with curl

Set reusable shell variables:

```bash
API_URL=http://localhost:4000
TEAM_ID=demo-team
```

### List tasks

```bash
curl --fail-with-body "$API_URL/api/tasks" | jq
```

### Create a task

Without an owner:

```bash
curl --fail-with-body -X POST "$API_URL/api/tasks" \
  -H 'Content-Type: application/json' \
  -d "{
    \"team_id\": \"$TEAM_ID\",
    \"title\": \"Build dashboard\",
    \"description\": \"Create the first dashboard screen\",
    \"owner_id\": null,
    \"due_date\": \"2026-08-14T17:00:00Z\"
  }" | jq
```

With the example owner, use `"owner_id": "demo-user"`.

Copy the `id` from the response:

```bash
TASK_ID=replace-with-task-id
```

### Get one task

```bash
curl --fail-with-body "$API_URL/api/tasks/$TASK_ID" | jq
```

### Update a task

Valid statuses are `todo`, `in_progress`, and `done`.

```bash
curl --fail-with-body -X PUT "$API_URL/api/tasks/$TASK_ID" \
  -H 'Content-Type: application/json' \
  -d '{"status":"in_progress"}' | jq
```

### Delete a task

Successful deletion returns HTTP `204 No Content`:

```bash
curl -i --fail-with-body -X DELETE "$API_URL/api/tasks/$TASK_ID"
```

## Development checks

```bash
cargo fmt --check
cargo check
cargo test
```
