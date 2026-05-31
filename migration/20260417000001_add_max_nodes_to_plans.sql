-- Update plans to add max_nodes limit
-- Phase 17: Multi-node support per user

-- Update existing plans to include max_nodes in limits
UPDATE plans SET limits = jsonb_set(limits, '{max_nodes}', '1') WHERE name = 'free';
UPDATE plans SET limits = jsonb_set(limits, '{max_nodes}', '1') WHERE name = 'starter';
UPDATE plans SET limits = jsonb_set(limits, '{max_nodes}', '3') WHERE name = 'pro';
UPDATE plans SET limits = jsonb_set(limits, '{max_nodes}', '-1') WHERE name = 'enterprise';

-- Verify the update
SELECT name, limits->>'max_nodes' as max_nodes FROM plans;