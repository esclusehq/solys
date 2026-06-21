---
status: resolved
trigger: "Setelah logout via dropdown TopBar -> Profile -> Settings -> Login ulang dengan rhnbztnl@gmail.com, akun akan login sebagai rhnbztnl dengan benar. Tapi jika dibiarkan idle ~30-60 menit, akun berubah sendiri ke ryhanbz217@gmail.com tanpa aksi apapun."
created: 2026-06-16
updated: 2026-06-16
---

## Symptoms

- **Expected**: Setelah login sebagai rhnbztnl@gmail.com, akun tetap rhnbztnl sampai user melakukan logout manual.
- **Actual**: Setelah ~30-60 menit idle, akun otomatis berganti ke ryhanbz217@gmail.com.
- **Error messages**: Tidak ada error di console browser.
- **Timeline**: Tidak diketahui kapan mulai terjadi.
- **Reproduction**: 
  1. Buka aplikasi
  2. Logout via dropdown TopBar -> Profile -> Settings
  3. Login dengan rhnbztnl@gmail.com
  4. Biarkan tab terbuka tanpa interaksi selama ~30-60 menit
  5. Akun berubah ke ryhanbz217@gmail.com

## Current Focus

- **root_cause_identified**: true
- **root_cause_summary**: |
    **Root Cause: Dual auth state dengan fallback yang tidak aman.**

    Mekanisme `checkAuth()` di `authStore.js` memiliki fallback ke Supabase session ketika backend auth gagal. Kombinasi dua masalah menyebabkan account switch:

    1. **`logout()` tidak memanggil `supabase.auth.signOut()`** — Supabase session (PKCE dengan `persistSession: true`) tetap tersimpan di localStorage setelah user logout.

    2. **Saat idle ~30-60 menit**: Access token backend expires (15 menit). Polling interval memicu 401 → `refreshAccessToken()` dipanggil. Jika refresh gagal atau timing tertentu, auth state jadi inconsistent.

    3. **checkAuth fallback mechanism**: Ketika `getMe()` gagal (token expired), `checkAuth()` fallback ke `supabase.auth.getSession()`. Jika Supabase session mengandung email user SEBELUMNYA (ryhanbz217) yang tidak pernah di-signOut, `oauthLogin('google', 'ryhanbz217@gmail.com')` dipanggil, dan auth store terisi dengan user ryhanbz217.

    **Contributing factor:** Backend `/auth/refresh` handler menggunakan `AuthUser` dari middleware yang mengekstrak user dari token/cookie. Middleware juga menerima REFRESH TOKEN sebagai sumber autentikasi (sama validnya dengan access token). Tidak ada pembedaan antara access dan refresh token di level middleware.

- **test**: Verifikasi bahwa Supabase session untuk ryhanbz217 masih ada di localStorage setelah logout dan login ulang sebagai rhnbztnl. Verifikasi bahwa `checkAuth()` fallback mechanism dapat menyebabkan account switch.
- **next_action**: Fix logout untuk memanggil `supabase.auth.signOut()`, dan perbaiki `checkAuth()` fallback agar hanya menggunakan Supabase session jika benar-benar diperlukan dan dengan validasi tambahan.
- **fix_applied**: 2026-06-16 — `authStore.js` line 33: tambah guard `session.user.email === get().user?.email` di fallback checkAuth. line 110: tambah `await supabase.auth.signOut()` di logout.

## Evidence

- timestamp: 2026-06-16T{current_time}
  investigator: session-manager
  finding: |
    **Frontend Auth Store (`app/src/store/authStore.js`):**
    - Uses Zustand `persist` middleware → saves `user`, `accessToken`, `refreshToken`, `isAuthenticated` to localStorage under key `escluse-auth`
    - `logout()` calls `authApi.logout()` (POST /auth/logout → clears cookies) + clears store state
    - **CRITICAL: `logout()` does NOT call `supabase.auth.signOut()`** → Supabase session persists in localStorage after logout
    - `refreshAccessToken()` calls `/auth/refresh` → on success, updates tokens and calls `getMe()` to refresh user object
    - `checkAuth()` has a **fallback mechanism**: jika backend `getMe()` gagal (token expired), coba recovery via `supabase.auth.getSession()` → `authApi.oauthLogin(provider, session.user.email)`

- timestamp: 2026-06-16T{current_time}
  investigator: session-manager
  finding: |
    **Backend Auth Flow (`api/src/presentation/handlers/auth_handlers.rs`):**
    - Access token expiry: **15 menit** (`JwtService` default)
    - Refresh token expiry: **7 hari**
    - `POST /auth/refresh` memerlukan `AuthUser` (middleware extracts dari Authorization header atau cookies)
    - `POST /auth/oauth` dan `POST /auth/logout` TIDAK memerlukan autentikasi
    - Tokens dikembalikan BOTH di response body (JSON) dan sebagai cookies (`Set-Cookie`)

- timestamp: 2026-06-16T{current_time}
  investigator: session-manager
  finding: |
    **Backend Middleware (`api/src/domain/auth/middleware.rs`):**
    - Ekstraksi user: 1) `Authorization: Bearer` header → 2) `access_token` cookie → 3) `refresh_token` cookie
    - Tidak ada pembedaan antara access token dan refresh token — keduanya bisa digunakan untuk autentikasi
    - `refresh_token` cookie di-cookie parser dengan `starts_with("refresh_token=")` — bisa rentan jika ada multiple cookies

- timestamp: 2026-06-16T{current_time}
  investigator: session-manager
  finding: |
    **Supabase Client (`app/src/lib/supabase.js`):**
    - Config: `autoRefreshToken: true`, `persistSession: true`, `flowType: 'pkce'`
    - Supabase secara otomatis merefresh session-nya sendiri (access token gotrue ~1 hour)
    - Menyimpan session di localStorage key `sb-<project_ref>-auth-token`
    - Fungsi `onAuthStateChange` diexport tapi **tidak pernah dipanggil** di app
    - Fungsi `signOut()` diexport tapi **tidak dipanggil di authStore.logout()**

- timestamp: 2026-06-16T{current_time}
  investigator: session-manager
  finding: |
    **ProtectRoute (`app/src/components/ProtectedRoute.jsx`):**
    - Memanggil `checkAuth()` pada mount effect → hanya sekali
    - Jika `isAuthenticated` false → redirect ke /login
    - Seluruh app di-wrap dalam ProtectedRoute, jadi komponen tidak remount saat navigasi SPA

- timestamp: 2026-06-16T{current_time}
  investigator: session-manager
  finding: |
    **Polling Intervals:**
    - `useBackups.js`: `setInterval(refresh, 10000)` — 10 detik
    - `useConnectivity.js`: `setInterval(refresh, pollIntervalMs)` — polling
    - `useServerMetrics.js`: auto-refresh 30 detik
    - Polling ini menggunakan `ApiClient.request` yang memiliki 401 retry logic — saat token expired, akan memicu `refreshAccessToken()`

## Eliminated

- **Hypothesis: Token refresh logic menggunakan refresh token yang salah.** → Dieliminasi. Backend refresh handler (auth_handlers.rs:237-268) mengambil `AuthUser` dari middleware yang mengekstrak dari cookies. Refresh token cookie di-set ulang pada setiap login/refresh dengan nama cookie yang sama. Tidak ada mekanisme yang bisa menyebabkan refresh token dari user berbeda digunakan.
- **Hypothesis: Session conflict di backend.** → Dieliminasi. Backend menggunakan JWT stateless — tidak ada session store. `user_id` diambil dari JWT claims, bukan dari shared session.
- **Hypothesis: Zustand persist cross-tab sync.** → Dieliminasi. Reproduksi hanya menggunakan satu tab.
