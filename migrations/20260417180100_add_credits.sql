-- Users table for credit balances (references Supabase auth.users)
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY REFERENCES auth.users(id),
    credit_balance INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Append-only credit ledger for audit trails and dispute resolution
CREATE TABLE credit_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    amount INTEGER NOT NULL,            -- positive = credit added, negative = debit
    balance_after INTEGER NOT NULL,     -- snapshot of balance after this transaction
    tx_type VARCHAR(30) NOT NULL,       -- 'STRIPE_PURCHASE', 'UPSCALE_DEBIT', 'REFUND'
    reference_id TEXT,                  -- Stripe checkout session ID or upscale job UUID
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_credit_tx_user ON credit_transactions(user_id, created_at DESC);

-- Prevent duplicate Stripe webhook processing (replay protection)
CREATE UNIQUE INDEX idx_credit_tx_stripe_ref ON credit_transactions(reference_id) WHERE tx_type = 'STRIPE_PURCHASE';

-- Add credits_charged to upscales for refund tracking
ALTER TABLE upscales ADD COLUMN credits_charged INTEGER NOT NULL DEFAULT 0;
