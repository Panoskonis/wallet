# Requirements
1. Users accounts - Authentication
2. Keeping track of expenses and income.
3. Each user can have multiple wallets (like one for each bank, one for cash, savings etc).
4. Filtering, grouping sorting to analyze expenses
5. User transaction data is saved both locally (sqllite db) and on server (postgres db) and can be synced.
6. Nice UI.


# Components

1. Postgres db (1 db for all users??)
    - users table (PK user_id, username, email, password, creation_date, last_update_date)
    - transactions table (PK transaction_id, user_id FK, wallet_id FK, category, amount, transaction_type, transaction_date, creation_date, last_update_date, description)
    - wallets table (PK wallet_id, name, wallet_type, user_id FK, creation_date, last_update_date, currency, description)
    - templates (PK template_id, user_id FK, name, category, default_amount, default_wallet FK, description)
2. RUST axum backend
3. Vue.js for frontend
4. Hosted on raspberry pi
5. 