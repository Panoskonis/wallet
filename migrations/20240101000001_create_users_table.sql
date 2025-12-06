-- Migration: Create users table
-- This table stores user account information for the wallet application

CREATE TABLE IF NOT EXISTS users (
    -- Primary key: UUID v4 for unique user identification
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- User email - must be unique and not null
    email VARCHAR(255) NOT NULL UNIQUE,
    
    -- User's full name
    name VARCHAR(255) NOT NULL,
    
    -- Password hash (store hashed passwords, never plain text!)
    password VARCHAR(255) NOT NULL,
    
    -- Timestamps for tracking when records are created/updated
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create an index on email for faster lookups
-- Indexes improve query performance for frequently searched columns
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- Create an index on created_at for sorting/filtering by registration date
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at);

-- Add a comment to the table for documentation
COMMENT ON TABLE users IS 'Stores user account information for the wallet application';

