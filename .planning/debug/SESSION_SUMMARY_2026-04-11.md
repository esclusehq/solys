---
title: April 11 2026 - Debug Session Summary
description: Summary of issues fixed during the debug session
status: completed
---

## Issues Fixed

### 1. Lemon Squeezy Checkout API (400/422 Errors)

**Problem:** Lemon Squeezy checkout creation was failing with various errors due to incorrect JSON:API format.

**Root Cause:** Incorrect payload format - using wrong field names and types for JSON:API spec.

**Errors Encountered:**
- 400 Bad Request: `cancel_url`, `custom_data`, `redirect_url` not supported as attributes
- 422 Unprocessable Entity: `checkout_data field must be an array`
- 400 Bad Request: `The member id must be a string` (variant.id)

**Solution:** Use correct JSON:API format:
```json
{
  "data": {
    "type": "checkouts",
    "attributes": {
      "checkout_data": {
        "custom": { "user_id": "..." }
      },
      "product_options": {
        "redirect_url": "http://127.0.0.1:5173/dashboard?checkout=success"
      }
    },
    "relationships": {
      "store": { "data": { "type": "stores", "id": "..." } },
      "variant": { "data": { "type": "variants", "id": "..." } }
    }
  }
}
```

**Key Points:**
- `checkout_data` = object (not array)
- Only valid fields in checkout_data: `email`, `name`, `custom`, `variant_quantities`, `billing_address`, `tax_number`, `discount_code`
- `url` and `cancel_url` NOT valid in checkout_data - use `product_options.redirect_url` instead
- `variant.id` and `store.id` must be strings (not numbers)

**Files Changed:**
- `api/src/infrastructure/billing/lemon_squeezy_service.rs`
- `api/src/config/app_config.rs` (APP_URL default: 8080 → 5173)
- `api/src/bootstrap/container.rs` (app_url default: 8080 → 5173)

---

### 2. Frontend Dashboard Route Missing

**Problem:** Redirect from Lemon Squeezy (`/dashboard?checkout=success`) was showing blank page.

**Root Cause:** Route `/dashboard` was not defined in App.jsx - only `/` was defined.

**Solution:** Added explicit route for `/dashboard` that renders DashboardPage.

**Files Changed:**
- `app/src/app/App.jsx` - Added `<Route path="/dashboard" element={<DashboardPage />} />`

---

### 3. Checkout Success Message Not Displaying

**Problem:** Dashboard page wasn't capturing or displaying the `checkout` query parameter.

**Root Cause:** 
1. `useSearchParams` hook not imported
2. useEffect dependencies not set correctly

**Solution:** 
- Import `useSearchParams` from react-router-dom
- Add separate useEffect to handle query parameters
- Display success/cancelled message based on `checkout` parameter

**Files Changed:**
- `app/src/pages/dashboard/DashboardPage.jsx`

---

### 4. Minecraft Container DNS Error (Earlier Session)

**Problem:** Minecraft server containers couldn't resolve DNS (mojang.com) during startup.

**Root Cause:** Container created without network attachment - `NetworkingConfig` was wrong API. Should use `HostConfig.network_mode = "bridge"`.

**Solution:** Changed container creation to use `HostConfig.network_mode = Some("bridge".to_string())`

**Files Changed:**
- `web-agent/src/handlers/runtime.rs`

---

## Database Notes

- Plan `starter` has variant_id `1490734` stored in `plans` table
- Variant IDs stored in columns: `lemon_squeezy_variant_id_monthly`, `lemon_squeezy_variant_id_yearly`

---

## Verification

All fixes verified:
- ✅ Lemon Squeezy checkout creates successfully and redirects to correct URL
- ✅ Frontend displays "Subscription activated!" message after successful payment
- ✅ Dashboard shows correct stats (1 server, running)
- ✅ Minecraft containers (when tested) should have proper network connectivity