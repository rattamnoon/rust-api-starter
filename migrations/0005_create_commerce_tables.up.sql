CREATE TABLE products (
    id uuid PRIMARY KEY DEFAULT uuidv7(),
    sku text NOT NULL UNIQUE,
    name text NOT NULL,
    description text NULL,
    price_amount bigint NOT NULL CHECK (price_amount >= 0),
    currency text NOT NULL,
    is_active boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE orders (
    id uuid PRIMARY KEY DEFAULT uuidv7(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status text NOT NULL,
    subtotal_amount bigint NOT NULL CHECK (subtotal_amount >= 0),
    total_amount bigint NOT NULL CHECK (total_amount >= 0),
    currency text NOT NULL,
    stripe_checkout_session_id text NULL,
    stripe_payment_intent_id text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE order_items (
    id uuid PRIMARY KEY DEFAULT uuidv7(),
    order_id uuid NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    product_id uuid NOT NULL REFERENCES products(id),
    product_name_snapshot text NOT NULL,
    sku_snapshot text NOT NULL,
    unit_price_amount bigint NOT NULL CHECK (unit_price_amount >= 0),
    quantity integer NOT NULL CHECK (quantity > 0),
    line_total_amount bigint NOT NULL CHECK (line_total_amount >= 0),
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE payments (
    id uuid PRIMARY KEY DEFAULT uuidv7(),
    order_id uuid NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    provider text NOT NULL,
    provider_payment_id text NULL,
    provider_session_id text NULL,
    status text NOT NULL,
    amount bigint NOT NULL CHECK (amount >= 0),
    currency text NOT NULL,
    raw_payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (order_id, provider, provider_session_id)
);

CREATE TABLE payment_webhook_events (
    id uuid PRIMARY KEY DEFAULT uuidv7(),
    provider_event_id text NOT NULL UNIQUE,
    event_type text NOT NULL,
    payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE receipts (
    id uuid PRIMARY KEY DEFAULT uuidv7(),
    order_id uuid NOT NULL UNIQUE REFERENCES orders(id) ON DELETE CASCADE,
    receipt_number text NOT NULL UNIQUE,
    status text NOT NULL,
    upload_id uuid NULL REFERENCES uploaded_files(id) ON DELETE SET NULL,
    issued_at timestamptz NOT NULL DEFAULT now(),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE email_deliveries (
    id uuid PRIMARY KEY DEFAULT uuidv7(),
    receipt_id uuid NOT NULL REFERENCES receipts(id) ON DELETE CASCADE,
    template_key text NOT NULL,
    recipient text NOT NULL,
    subject text NOT NULL,
    status text NOT NULL,
    provider_message_id text NULL,
    error_message text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE workflow_runs (
    id uuid PRIMARY KEY DEFAULT uuidv7(),
    order_id uuid NOT NULL UNIQUE REFERENCES orders(id) ON DELETE CASCADE,
    workflow_id text NOT NULL,
    namespace text NOT NULL,
    task_queue text NOT NULL,
    status text NOT NULL,
    last_error text NULL,
    started_at timestamptz NULL,
    finished_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
