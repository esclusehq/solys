import { useState, useEffect, useCallback } from 'react';
import { fetchApi } from '../api/client';
import { supabase } from '../lib/supabase';

export function useProfile() {
    const [loginHistory, setLoginHistory] = useState([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);

    const refetchLoginHistory = useCallback(async () => {
        try {
            setLoading(true);
            const data = await fetchApi('/auth/login-history');
            setLoginHistory(Array.isArray(data) ? data : []);
            setError(null);
        } catch (err) {
            setError(err.message);
            setLoginHistory([]);
        } finally {
            setLoading(false);
        }
    }, []);

    useEffect(() => {
        refetchLoginHistory();
    }, [refetchLoginHistory]);

    const updateProfile = async (data) => {
        try {
            const result = await fetchApi('/auth/profile', {
                method: 'PUT',
                body: JSON.stringify(data),
            });
            return result;
        } catch (err) {
            throw err;
        }
    };

    return {
        loginHistory,
        loading,
        error,
        refetchLoginHistory,
        updateProfile,
    };
}

export async function uploadAvatar(file, userId) {
    const allowedTypes = ['image/jpeg', 'image/png', 'image/webp'];
    if (!allowedTypes.includes(file.type)) {
        throw new Error('Only JPG, PNG, and WebP files are allowed');
    }
    if (file.size > 2 * 1024 * 1024) {
        throw new Error('File size must be under 2MB');
    }

    const ext = file.name.split('.').pop();
    const filePath = `avatars/${userId}/${crypto.randomUUID()}.${ext}`;

    const { error } = await supabase.storage
        .from('avatars')
        .upload(filePath, file, { upsert: false });

    if (error) throw error;

    const { data } = supabase.storage.from('avatars').getPublicUrl(filePath);
    return data.publicUrl;
}
