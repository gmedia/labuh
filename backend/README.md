# Labuh Backend

Backend service for Labuh - a lightweight PaaS platform. Built with Rust using the **Hexagonal Architecture** (Ports and Adapters) pattern to ensure a highly decoupled, testable, and maintainable codebase.

## 🏗️ Architecture Overview

Labuh follows the Hexagonal Architecture pattern, which separates the core business logic from external concerns like databases, APIs, and the container runtime.

### 🧩 Components

```mermaid
graph TD
    subgraph "API Layer (Adapters)"
        REST[Axum REST API]
    end

    subgraph "Application Layer"
        UC[UseCases]
    end

    subgraph "Domain Layer (Core)"
        DP[Ports / Traits]
        DM[Domain Models]
    end

    subgraph "Infrastructure Layer (Adapters)"
        DB[(SQLite / SQLx)]
        RT[Docker Runtime]
    end

    REST --> UC
    UC --> DP
    UC --> DM
    DP <|-- DB
    DP <|-- RT
```

### 📂 Directory Structure

- `src/api/`: **Primary Adapters**. Contains the Axum REST API handlers and routes.
- `src/usecase/`: **Application Services**. Orchestrates domain logic and interacts with domain ports.
- `src/domain/`: **Core Logic**.
  - `models/`: Plain Data Objects (PDOs) representing the business entities.
  - `ports/`: Traits defining the interface for repositories and external services (e.g., `StackRepository`, `RuntimePort`).
- `src/infrastructure/`: **Secondary Adapters**. Concrete implementations of domain ports.
  - `sqlite/`: Database persistence using SQLx.
  - `runtime/`: Docker runtime interaction using `bollard`.
- `src/services/`: **Legacy Services**. (Phasing out) Bridge services preserved for background task stability.

## 🛠️ Tech Stack

- **Framework**: [Axum](https://github.com/tokio-rs/axum)
- **Database**: [SQLite](https://sqlite.org/) with [SQLx](https://github.com/launchbadge/sqlx)
- **Runtime**: [Tokio](https://tokio.rs/)
- **API Client**: [Bollard](https://github.com/fussybeaver/bollard) (Docker API)
- **Authentication**: JWT via [jsonwebtoken](https://github.com/Keats/jsonwebtoken)

## 🚀 Getting Started

### Prerequisites

- Rust (latest stable)
- Docker (running locally)

### Configuration

Copy the example environment file and adjust the values:

```bash
cp .env.example .env
```

Key variables:

- `DATABASE_URL`: Path to the SQLite database (e.g., `sqlite:labuh.db`).
- `JWT_SECRET`: Secret key for JWT signing.
- `SERVER_ADDR`: Binding address (default: `0.0.0.0:3000`).

### Running Locally

```bash
# Run migrations and start the server
cargo run
```

### Build for Production

```bash
cargo build --release
```

## 🧪 Testing & Linting

```bash
# Code formatting
cargo fmt

# Static analysis
cargo clippy

# Run tests
cargo test
```
