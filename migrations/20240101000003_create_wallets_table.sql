-- Migration: Create wallets table
-- This table stores the current balance for each user's wallet

CREATE TABLE IF NOT EXISTS wallets (
    -- Primary key: UUID v4 for unique wallet identification
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Foreign key reference to the user who owns this wallet
    -- ON DELETE CASCADE ensures wallet is deleted when user is deleted
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    
    -- Current wallet balance
    -- Using DECIMAL(19,4) for precise financial calculations
    balance DECIMAL(19,4) NOT NULL DEFAULT 0.0000 CHECK (balance >= 0),
    
    -- Currency code (ISO 4217 format, e.g., 'USD', 'EUR')
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    
    -- Timestamps for tracking when wallet is created/updated
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index on user_id for fast wallet lookups by user
CREATE INDEX IF NOT EXISTS idx_wallets_user_id ON wallets(user_id);

-- Add comment for documentation
COMMENT ON TABLE wallets IS 'Stores current balance information for each user wallet';

