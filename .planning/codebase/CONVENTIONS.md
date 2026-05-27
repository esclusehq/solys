# Coding Conventions

**Analysis Date:** 2026-04-08

## Naming Patterns

### Files

**JavaScript/JSX (app):**
- PascalCase for components: `LoginPage.jsx`, `ServerManagerPage.jsx`, `Sidebar.jsx`
- CamelCase for utilities/stores: `api.js`, `authStore.js`, `useWebSocket.js`
- Kebab-case for directories: `pages/auth`, `components/IDE`, `pages/servers`

**Rust (api):**
- Snake_case for modules and files: `server_handlers.rs`, `sqlx_repository.rs`, `mod.rs`
- PascalCase for structs and types: `ServerHandlers`, `CreateServerRequest`, `ApiResponse`
- CamelCase for functions and methods: `list_servers`, `create_server`, `get_token`

### Functions

**JavaScript:**
- camelCase for all functions: `login`, `refreshAccessToken`, `handleLogout`

**Rust:**
- Snake_case for functions: `fn list_servers`, `fn create_server`

### Variables

**JavaScript:**
- camelCase: `isAuthenticated`, `accessToken`, `sidebarOpen`
- Constants (uppercase with underscore): `API_URL`, `DEFAULT_PORT`

**Rust:**
- Snake_case: `auth_user`, `server_port`, `pool.clone()`

### Types

**JavaScript:**
- No explicit type annotations (plain JavaScript with JSX)
- JSDoc occasionally used for documentation

**Rust:**
- PascalCase for structs and enums: `Node`, `Server`, `CreateServerRequest`
- Tuple structs for value objects: `struct ApiResponse<T>(T)`

## Code Style

### Formatting

**JavaScript:**
- Prettier not configured - no explicit formatting rules detected
- Uses ES6+ features: arrow functions, destructuring, async/await, template literals

**Rust:**
- Uses `cargo fmt` (Rust's standard formatter)
- Standard Rust formatting conventions:
  ```rust
  async fn list_servers(
      State(state): State<ApiState>,
      auth_user: AuthUser,
  ) -> Result<impl IntoResponse, String> {
  ```

### Linting

**JavaScript:**
- No ESLint configuration detected (no `.eslintrc*` files)
- No explicit linting rules

**Rust:**
- Uses Clippy for additional linting (via `cargo clippy`)
- Standard Rust warnings enabled

### Tailwind CSS (app)

- Uses Tailwind CSS v4 with `@import "tailwindcss"` syntax
- Custom theme defined in `index.css` with CSS variables:
  ```css
  @theme {
      --color-deep-space: #080b15;
      --color-nebula: #0d0f1a;
      --color-cosmic-cyan: #0ddff2;
  }
  ```
- Custom classes: `.glass-panel`, `.glow-cyan`, `.stars-bg`, `.status-dot`

## Import Organization

### JavaScript (app/src)

```javascript
// 1. React and React Router imports
import React from 'react'
import ReactDOM from 'react-dom/client'
import { BrowserRouter, Routes, Route } from 'react-router-dom'

// 2. Third-party library imports
import { create } from 'zustand'
import { persist } from 'zustand/middleware'

// 3. Internal imports - relative paths
import { useAuthStore } from '../store/authStore'
import { useUIStore } from '../store/uiStore'
import LoginPage from '../pages/auth/LoginPage'

// 4. Internal imports - no extension
import { api, serversApi } from './lib/api'
import * as authApi from '../api/auth'
```

### Rust (api/src)

```rust
// 1. Standard library imports
use std::net::SocketAddr;

// 2. External crate imports (alphabetical)
use axum::{extract::State, Json, Router};
use serde::Deserialize;
use uuid::Uuid;

// 3. Internal crate imports
use crate::domain::auth::middleware::AuthUser;
use crate::domain::server::model::{CreateServerRequest, Server};
use crate::infrastructure::solys_client::client::SolysServerStats;
```

## Error Handling

### JavaScript

- Uses try/catch blocks for async operations
- Throws errors with messages for API failures
- Zustand store handles error state in `error` property
- Example from `authStore.js`:
  ```javascript
  try {
    const user = await authApi.getMe()
    set({ user, isAuthenticated: true })
  } catch (err) {
    set({ user: null, isAuthenticated: false })
    return false
  }
  ```

### Rust

- Uses `Result<T, String>` or `anyhow::Result<()>` for error handling
- Propagation with `?` operator
- Error messages converted to strings for HTTP responses
- Example from `server_handlers.rs`:
  ```rust
  async fn list_servers(...)
      -> Result<impl IntoResponse, String> {
      let servers = repo.find_by_user_id(auth_user.tenant_id)
          .await
          .map_err(|e| e.to_string())?;
      Ok(Json(ApiResponse::success(servers)))
  }
  ```

## Logging

### JavaScript

- Uses `console.log`, `console.error` sparingly
- Error logging in store/actions:
  ```javascript
  console.error('SignOut error:', e)
  console.error('Token refresh failed:', err)
  ```

### Rust

- Uses `tracing` crate for structured logging
- Examples from codebase:
  ```rust
  tracing::info!("Listening on {}", addr);
  tracing::info!("create_server called: user_id={}", auth_user.user_id);
  tracing::debug!("Processing request: {:?}", request);
  ```

## Comments

### JavaScript

- Minimal comments in source code
- Occasionally uses JSDoc-style comments in API files
- No consistent documentation pattern

### Rust

- Minimal inline comments
- Tests include descriptive comments:
  ```rust
  #[test]
  fn test_node_creation() {
      // Test node with valid parameters
  }
  ```

## Function Design

### JavaScript

- Component functions: Functional components with hooks
- Store actions: Async functions that update state
- API methods: Class-based with async/await

### Rust

- Handler functions: Async with extractors
- Repository functions: Async with SQLx
- Clear separation between handlers, services, and repositories

## Module Design

### JavaScript (app)

- Component-based architecture
- Store pattern with Zustand for state management
- Hooks for reusable logic: `useServers.js`, `useNodes.js`, `useAlerts.js`
- API clients organized by domain: `serversApi`, `nodesApi`, `billingApi`

### Rust (api)

- Layered architecture: Presentation → Application → Domain → Infrastructure
- Module organization:
  - `presentation/handlers/` - HTTP handlers
  - `application/services/` - Business logic services
  - `domain/` - Entities, repositories, services
  - `infrastructure/` - Database, external services, executors

### Exports

**JavaScript:**
- Named exports for components and utilities
- Default exports for page components

**Rust:**
- Re-exports via `mod.rs` files
- Public modules marked with `pub`

---

*Convention analysis: 2026-04-08*