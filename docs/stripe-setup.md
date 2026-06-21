# Stripe Setup — WireSentinel Cloud

Phase 14 billing integration via `cloud-billing` and Stripe Checkout.

## Local / CI (mock)

Set `STRIPE_MOCK=1` (default in CI). Mock provider accepts any webhook payload JSON and skips signature validation.

```bash
export STRIPE_MOCK=1
cargo run -p cloud-api
```

Test webhook:

```bash
curl -X POST http://127.0.0.1:8090/api/v1/billing/webhook \
  -H 'Content-Type: application/json' \
  -d '{"type":"checkout.session.completed","data":{"object":{"metadata":{"tenant_id":"<tenant>","plan_id":"team"}}}}'
```

## Production

1. Create Stripe account and products/prices for `team`, `enterprise`, `enterprise_plus`.
2. Set secrets:

```bash
export STRIPE_SECRET_KEY=sk_live_...
export STRIPE_WEBHOOK_SECRET=whsec_...
unset STRIPE_MOCK
```

3. Build API with Stripe feature (optional native SDK):

```bash
cargo build -p cloud-billing --features stripe
cargo build -p cloud-api
```

4. Configure webhook endpoint in Stripe Dashboard:
   - URL: `https://<cloud-host>/api/v1/billing/webhook`
   - Events: `checkout.session.completed`, `customer.subscription.created`, `invoice.paid`

5. Metadata required on checkout sessions (set automatically by `BillingManager`):
   - `tenant_id`
   - `plan_id`

## Security (14-O)

- `CloudSecurityPolicy::validate_billing_webhook` requires `Stripe-Signature` header (unless `STRIPE_MOCK=1`).
- Failed validation records `BillingSecurityViolation` in `billing_events` and returns HTTP 401.
- Successful subscription changes write `audit_events` with action `billing.subscription.create` or `billing.webhook.*`.

## Checkout flow

1. Client calls `POST /api/v1/billing/checkout` with `plan_id`, `success_url`, `cancel_url`.
2. User completes Stripe Checkout.
3. Webhook creates/updates subscription and optional invoice row.

## Troubleshooting

| Symptom | Check |
|---------|-------|
| 401 on webhook | Signature header, `STRIPE_WEBHOOK_SECRET`, clock skew |
| No subscription after pay | Webhook delivery logs, `billing_events` table |
| Wrong plan | Checkout metadata `plan_id` |
