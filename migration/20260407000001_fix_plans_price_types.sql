-- Migration: Fix plans table price column types
-- Description: Change DECIMAL to FLOAT8 for price columns to match Rust f64 type

ALTER TABLE plans ALTER COLUMN price_monthly TYPE FLOAT8;
ALTER TABLE plans ALTER COLUMN price_yearly TYPE FLOAT8;
