---
status: testing
phase: 15-billing-plans-subscription-integration
source:
  - 15-01-SUMMARY.md
  - 15-02-SUMMARY.md
  - 15-03-SUMMARY.md
started: 2026-04-12T00:00:00Z
updated: 2026-04-12T00:00:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Webhook with Valid Signature
expected: Send a webhook request to the billing endpoint with a valid HMAC-SHA256 signature in the X-Signature header. The request should be accepted and processed successfully (200 OK).
result: skipped
reason: Requires backend API testing with webhook signatures

### 2. Webhook with Invalid Signature
expected: Send a webhook request with an invalid or missing X-Signature header. The request should be rejected with 400 status and INVALID_SIGNATURE error.
result: skipped
reason: Requires backend API testing with webhook signatures

### 3. Subscription Limit - Under Quota
expected: Create a server when under the subscription's server limit. The server should be created successfully.
result: skipped
reason: Requires running node agent - testing requires actual server deployment

### 4. Subscription Limit - Over Quota
expected: Attempt to create a server when the subscription's server limit is already exceeded. The request should be rejected with QUOTA_EXCEEDED error containing limit details.
result: skipped
reason: Requires running node agent - testing requires quota to be at limit

### 5. View Current Subscription on Billing Page
expected: Navigate to the billing page. The current subscription plan name, status (active/inactive), and term should be displayed at the top of the page.
result: pass

### 6. View Subscription Limits
expected: On the billing page, view the subscription limits section. It should show max servers, max RAM (MB), max CPU cores, and max disk (GB) for the current plan.
result: pass

## Summary

total: 6
passed: 2
issues: 0
pending: 0
skipped: 4
blocked: 0

## Gaps

[none yet]