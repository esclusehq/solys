---
status: resolved
trigger: "Fix Lemon Squeezy checkout API error - Multiple 400/422 errors due to incorrect JSON:API format"
created: 2026-04-11T00:00:00Z
updated: 2026-04-11T12:45:00Z
---

## Summary

Fixed Lemon Squeezy checkout API that was returning 400/422 errors due to incorrect JSON:API payload format. The fix went through multiple iterations before finding the correct format.

## Errors Encountered

1. **400 Bad Request**: `cancel_url`, `custom_data`, `redirect_url` not supported as top-level attributes
2. **422 Unprocessable Entity**: `checkout_data field must be an array`
3. **400 Bad Request**: `The member id must be a string` (for variant.id)

## Root Cause

The Lemon Squeezy API (JSON:API format) has specific requirements:
- `checkout_data` must be an **object** (not array) containing specific allowed fields
- `variant.id` must be a **string** (not number)
- `url` and `cancel_url` are NOT valid fields inside `checkout_data`
- Redirect URL must be placed in `product_options.redirect_url`

## Correct JSON Format

```json
{
  "data": {
    "type": "checkouts",
    "attributes": {
      "checkout_data": {
        "custom": {
          "user_id": "USER_UUID"
        }
      },
      "product_options": {
        "redirect_url": "http://127.0.0.1:5173/dashboard?checkout=success"
      }
    },
    "relationships": {
      "store": {
        "data": { "type": "stores", "id": "STORE_ID" }
      },
      "variant": {
        "data": { "type": "variants", "id": "VARIANT_ID" }
      }
    }
  }
}
```

## Key Points

| Field | Format | Notes |
|-------|--------|-------|
| `checkout_data` | **Object** | Only valid fields: `email`, `name`, `custom`, `variant_quantities`, `billing_address`, `tax_number`, `discount_code` |
| `checkout_data.custom` | **Object** | Custom data passed to checkout |
| `product_options.redirect_url` | **String** | URL after successful payment |
| `variant.id` | **String** | NOT a number |
| `store.id` | **String** | NOT a number |

## Invalid Fields (Caused Errors)

- `checkout_data.url` - ❌ Not valid
- `checkout_data.cancel_url` - ❌ Not valid
- `checkout_data` as array - ❌ Not valid (should be object)
- `variant.id` as number - ❌ Must be string

## Files Changed

- `api/src/infrastructure/billing/lemon_squeezy_service.rs` - Fixed JSON payload format
- `api/src/config/app_config.rs` - Changed default APP_URL from 8080 to 5173
- `api/src/bootstrap/container.rs` - Changed default app_url from 8080 to 5173

## Verification

Checkout now works - redirects to `http://127.0.0.1:5173/dashboard?checkout=success` with success message displayed on dashboard.