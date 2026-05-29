-- Migration: Setup avatars storage bucket with RLS policies
-- Description: Creates the 'avatars' bucket for user avatar uploads (D-01)

-- Create the avatars bucket (public for read access via CDN)
INSERT INTO storage.buckets (id, name, public, avif_autodetection, file_size_limit, allowed_mime_types)
VALUES (
    'avatars',
    'avatars',
    true,
    false,
    2097152, -- 2MB in bytes (D-03)
    ARRAY['image/jpeg', 'image/png', 'image/webp']::text[]
)
ON CONFLICT (id) DO NOTHING;

-- RLS: Allow authenticated users to upload to their own folder
-- Path format: avatars/{user_id}/{uuid}.ext
CREATE POLICY "Users can upload their own avatar"
ON storage.objects FOR INSERT TO authenticated
WITH CHECK (
    bucket_id = 'avatars' AND
    (storage.foldername(name))[1] = 'avatars' AND
    (storage.foldername(name))[2] = auth.uid()::text
);

-- RLS: Allow authenticated users to update their own avatar
CREATE POLICY "Users can update their own avatar"
ON storage.objects FOR UPDATE TO authenticated
USING (
    bucket_id = 'avatars' AND
    (storage.foldername(name))[1] = 'avatars' AND
    (storage.foldername(name))[2] = auth.uid()::text
);

-- RLS: Anyone can view avatars (public bucket for CDN)
CREATE POLICY "Anyone can view avatars"
ON storage.objects FOR SELECT TO anon, authenticated
USING (bucket_id = 'avatars');

-- RLS: Users can delete their own avatar
CREATE POLICY "Users can delete their own avatar"
ON storage.objects FOR DELETE TO authenticated
USING (
    bucket_id = 'avatars' AND
    (storage.foldername(name))[1] = 'avatars' AND
    (storage.foldername(name))[2] = auth.uid()::text
);
