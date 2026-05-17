# Change: Commerce Payments And Receipts

## Summary
Added a commerce starter flow for product catalog management, order creation, Stripe-backed checkout session creation, payment webhook ingestion, receipt PDF generation, and receipt email delivery.

## Motivation
The backend previously had auth, uploads, and background jobs, but no transactional commerce flow for selling products and confirming payment.

## Affected Flows
- Product CRUD for admins
- Order creation for authenticated users
- Stripe checkout session creation per order
- Webhook-driven payment confirmation
- Receipt generation, storage, and email delivery

## Modules/Services Changed
- `products`
- `orders`
- `payments`
- `receipts`
- shared email, PDF, and workflow orchestration helpers

## Backward Compatibility
Existing auth, upload, user, and job flows remain in place. New config is required only when enabling real Stripe or Resend integration.

## Operational Notes
- Stripe secrets must be configured before checkout/webhook flows will work against the real provider.
- Receipt PDFs are stored locally under the uploads root in the `receipts` subfolder.

## References
- `migrations/0005_create_commerce_tables.up.sql`
- `src/modules/payments`
- `src/modules/receipts`
