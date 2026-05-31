-- Drop existing functions first
DROP FUNCTION IF EXISTS create_api_key(TEXT, TEXT[], TIMESTAMPTZ);
DROP FUNCTION IF EXISTS revoke_api_key(UUID);
DROP FUNCTION IF EXISTS list_api_keys();

-- Create create_api_key function
CREATE FUNCTION create_api_key(
  p_name TEXT DEFAULT 'API Key',
  p_scopes TEXT[] DEFAULT ARRAY['read'],
  p_expires_at TIMESTAMPTZ DEFAULT NULL
)
RETURNS TABLE (id UUID, full_key TEXT)
LANGUAGE plpgsql
SECURITY DEFINER
AS '
DECLARE
  v_user_id UUID := auth.uid();
  v_key_id UUID;
  v_full_key TEXT;
  v_key_hash TEXT;
  v_prefix TEXT;
BEGIN
  v_full_key := ''escluse_sk_'' || encode(gen_random_bytes(24), ''base64url'');
  v_prefix := substr(v_full_key, 1, 12);
  v_key_hash := encode(digest(v_full_key, ''sha256''), ''hex'');
  
  INSERT INTO api_keys (user_id, name, key_hash, key_prefix, scopes, expires_at)
  VALUES (v_user_id, p_name, v_key_hash, v_prefix, p_scopes, p_expires_at)
  RETURNING id INTO v_key_id;
  
  RETURN QUERY SELECT v_key_id, v_full_key;
END;
';

-- Create revoke_api_key function
CREATE FUNCTION revoke_api_key(p_key_id UUID)
RETURNS VOID
LANGUAGE plpgsql
SECURITY DEFINER
AS '
BEGIN
  UPDATE api_keys SET revoked_at = NOW() WHERE id = p_key_id AND user_id = auth.uid();
END;
';

-- Create list_api_keys function
CREATE FUNCTION list_api_keys()
RETURNS TABLE (id UUID, name TEXT, key_prefix TEXT, scopes TEXT[], last_used_at TIMESTAMPTZ, expires_at TIMESTAMPTZ, created_at TIMESTAMPTZ, is_active BOOLEAN)
LANGUAGE plpgsql
SECURITY DEFINER
AS '
BEGIN
  RETURN QUERY
  SELECT ak.id, ak.name, ak.key_prefix, ak.scopes, ak.last_used_at, ak.expires_at, ak.created_at,
    CASE WHEN ak.revoked_at IS NULL AND (ak.expires_at IS NULL OR ak.expires_at > NOW()) THEN TRUE ELSE FALSE END
  FROM api_keys ak WHERE ak.user_id = auth.uid() ORDER BY ak.created_at DESC;
END;
';
