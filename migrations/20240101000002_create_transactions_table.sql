-- Migration: Create transactions table
-- This table stores all financial transactions for users' wallets

CREATE TYPE transaction_type AS ENUM ('Expense', 'Income');

CREATE TABLE IF NOT EXISTS transactions (
    -- Primary key: UUID v4 for unique transaction identification
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Foreign key reference to the user who owns this transaction
    -- ON DELETE CASCADE means if a user is deleted, their transactions are also deleted
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Transaction type: deposit, withdrawal, or transfer
    transaction_type transaction_type NOT NULL,
    
    -- Transaction amount - stored as DECIMAL for precise financial calculations
    -- Using DECIMAL(19,4) allows up to 15 digits before decimal, 4 after
    -- This prevents floating-point precision issues with money
    amount DECIMAL(19,4) NOT NULL,

    -- Category of the transaction
    category VARCHAR(255) NOT NULL,
    
    -- Optional description or note about the transaction
    description TEXT,
    
    -- wallet_id UUID NOT NULL REFERENCES wallets(id) ON DELETE CASCADE,
    
    -- Current balance after this transaction
    -- This denormalized field makes balance queries faster
    -- balance_after DECIMAL(19,4) NOT NULL,
    
    -- Timestamps for tracking when transactions occur
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    last_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()


);

-- Create indexes for common query patterns
-- Index on user_id for quickly finding all transactions for a user
CREATE INDEX IF NOT EXISTS idx_transactions_user_id ON transactions(user_id);

-- Index on created_at for sorting transactions by date
CREATE INDEX IF NOT EXISTS idx_transactions_created_at ON transactions(created_at DESC);

-- Composite index for user transactions sorted by date (common query pattern)
CREATE INDEX IF NOT EXISTS idx_transactions_user_created ON transactions(user_id, created_at DESC);

-- Index on transaction_type for filtering by type
CREATE INDEX IF NOT EXISTS idx_transactions_type ON transactions(transaction_type);

-- Add comments for documentation
COMMENT ON TABLE transactions IS 'Stores all financial transactions for user wallets';
-- COMMENT ON COLUMN transactions.balance_after IS 'Account balance after this transaction (denormalized for performance)';

