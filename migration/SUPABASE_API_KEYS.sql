-- API Keys Table (Production-Ready)
CREATE TABLE IF NOT EXISTS api_keys (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  name TEXT NOT NULL DEFAULT 'API Key',
  key_hash TEXT NOT NULL,
  key_prefix TEXT NOT NULL,  -- Untuk lookup cepat (contoh: escluse_sk_abcd)
  scopes TEXT[] DEFAULT ARRAY['read'],  -- Optional permissions
  last_used_at TIMESTAMPTZ,
  expires_at TIMESTAMPTZ,  -- Optional expiration
  created_at TIMESTAMPTZ DEFAULT NOW(),
  revoked_at TIMESTAMPTZ  -- Untuk revocation
);

-- Enable RLS
ALTER TABLE api_keys ENABLE ROW LEVEL SECURITY;

-- RLS Policies
CREATE POLICY "Users can view own api_keys" ON api_keys
  FOR SELECT USING (auth.uid() = user_id);

CREATE POLICY "Users can insert own api_keys" ON api_keys
  FOR INSERT WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can delete own api_keys" ON api_keys
  FOR DELETE USING (auth.uid() = user_id);

CREATE POLICY "Users can update own api_keys" ON api_keys
  FOR UPDATE USING (auth.uid() = user_id);

-- Indexes for performance
CREATE INDEX idx_api_keys_user_id ON api_keys(user_id);
CREATE INDEX idx_api_keys_prefix ON api_keys(key_prefix) WHERE revoked_at IS NULL;

-- Function to create API key
CREATE OR REPLACE FUNCTION create_api_key(
  p_name TEXT DEFAULT 'API Key',
  p_scopes TEXT[] DEFAULT ARRAY['read'],
  p_expires_at TIMESTAMPTZ DEFAULT NULL
)
RETURNS TABLE (id UUID, full_key TEXT)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
  v_user_id UUID := auth.uid();
  v_key_id UUID;
  v_full_key TEXT;
  v_key_hash TEXT;
  v_prefix TEXT;
BEGIN
  -- Generate random key: escluse_sk_ + 32 random chars
  v_full_key := 'escluse_sk_' || encode(gen_random_bytes(24), 'base64url');
  
  -- Get prefix (first 12 chars)
  v_prefix := substr(v_full_key, 1, 12);
  
  -- Hash the full key
  v_key_hash := encode(digest(v_full_key, 'sha256'), 'hex');
  
  -- Insert into database
  INSERT INTO api_keys (user_id, name, key_hash, key_prefix, scopes, expires_at)
  VALUES (v_user_id, p_name, v_key_hash, v_prefix, p_scopes, p_expires_at)
  RETURNING id INTO v_key_id;
  
  RETURN QUERY SELECT v_key_id, v_full_key;
END;
$$ LANGUAGE plpgsql;

-- Function to revoke API key
CREATE OR REPLACE FUNCTION revoke_api_key(p_key_id UUID)
RETURNS VOID
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
  UPDATE api_keys
  SET revoked_at = NOW()
  WHERE id = p_key_id AND user_id = auth.uid();
END;
$$ LANGUAGE plpgsql;

-- Function to list user API keys (without hash)
CREATE OR REPLACE FUNCTION list_api_keys()
RETURNS TABLE (
  id UUID,
  name TEXT,
  key_prefix TEXT,
  scopes TEXT[],
  last_used_at TIMESTAMPTZ,
  expires_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ,
  is_active BOOLEAN
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
  RETURN QUERY
  SELECT 
    ak.id,
    ak.name,
    ak.key_prefix,
    ak.scopes,
    ak.last_used_at,
    ak.expires_at,
    ak.created_at,
    CASE 
      WHEN ak.revoked_at IS NULL 
        AND (ak.expires_at IS NULL OR ak.expires_at > NOW())
      THEN TRUE 
      ELSE FALSE 
    END AS is_active
  FROM api_keys ak
  WHERE ak.user_id = auth.uid()
  ORDER BY ak.created_at DESC;
END;
$$ LANGUAGE plpgsql;

-- Function to verify API key (for backend use)
CREATE OR REPLACE FUNCTION verify_api_key(p_key TEXT)
RETURNS TABLE (user_id UUID, scopes TEXT[])
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
  v_prefix TEXT;
  v_record RECORD;
BEGIN
  -- Get prefix from key
  v_prefix := substr(p_key, 1, 12);
  
  -- Find matching key
  SELECT ak.user_id, ak.key_hash, ak.scopes, ak.revoked_at, ak.expires_at
  INTO v_record
  FROM api_keys ak
  WHERE ak.key_prefix = v_prefix
    AND ak.revoked_at IS NULL
  LIMIT 1;
  
  -- Verify if key matches and not expired
  IF v_record.user_id IS NOT NULL THEN
    IF encode(digest(p_key, 'sha256'), 'hex') = v_record.key_hash THEN
      IF v_record.expires_at IS NULL OR v_record.expires_at > NOW() THEN
        RETURN QUERY SELECT v_record.user_id, v_record.scopes;
      END IF;
    END IF;
  END IF;
  
  RETURN;
END;
$$ LANGUAGE plpgsql;
