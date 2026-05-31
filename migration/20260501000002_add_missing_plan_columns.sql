-- Fix: Add missing billing columns to plans table
-- The Rust Plan struct has these fields but they were not in the original CREATE TABLE

ALTER TABLE plans ADD COLUMN IF NOT EXISTS lemon_squeezy_variant_id_monthly VARCHAR(255);
ALTER TABLE plans ADD COLUMN IF NOT EXISTS lemon_squeezy_variant_id_yearly VARCHAR(255);

-- stripe columns were in the original migration but may be missing in some environments
ALTER TABLE plans ADD COLUMN IF NOT EXISTS stripe_price_id_monthly VARCHAR(255);
ALTER TABLE plans ADD COLUMN IF NOT EXISTS stripe_price_id_yearly VARCHAR(255);
