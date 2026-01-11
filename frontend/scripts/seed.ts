import { createApiClient } from '../src/lib/api.js';
import type { User } from '../src/lib/types.js';

function env(name: string) {
	const v = process.env[name];
	return v && v.trim() ? v.trim() : undefined;
}

const BACKEND_ORIGIN = env('BACKEND_ORIGIN') ?? env('VITE_BACKEND_ORIGIN') ?? 'http://localhost:3000';

const api = createApiClient(BACKEND_ORIGIN);

type SeedUser = { email: string; name: string; password: string };
type SeedTx = {
	user_email: string;
	transaction_type: 'Expense' | 'Income';
	amount: number;
	category?: string;
	description?: string;
};

const seedUsers: SeedUser[] = [
	{ email: 'alice@example.com', name: 'Alice', password: 'password123' },
	{ email: 'bob@example.com', name: 'Bob', password: 'password123' },
	{ email: 'carol@example.com', name: 'Carol', password: 'password123' }
];

const seedTxs: SeedTx[] = [
	{
		user_email: 'alice@example.com',
		transaction_type: 'Income',
		amount: 2500,
		category: 'Other',
		description: 'seed: salary'
	},
	{
		user_email: 'alice@example.com',
		transaction_type: 'Expense',
		amount: 42.75,
		category: 'Groceries',
		description: 'seed: groceries'
	},
	{
		user_email: 'bob@example.com',
		transaction_type: 'Expense',
		amount: 18,
		category: 'Restaurant',
		description: 'seed: lunch'
	},
	{
		user_email: 'carol@example.com',
		transaction_type: 'Income',
		amount: 120,
		category: 'Other',
		description: 'seed: refund'
	}
];

async function ensureUser(u: SeedUser, existingByEmail: Map<string, User>) {
	if (existingByEmail.has(u.email)) return;
	await api.createUser(u);
}

async function ensureTransaction(tx: SeedTx, existingUsersByEmail: Map<string, User>) {
	const user = existingUsersByEmail.get(tx.user_email);
	if (!user) throw new Error(`User not found (needed for tx): ${tx.user_email}`);

	// Transactions endpoint filters by user_id; avoid duplicates by checking description.
	const current = await api.getTransactions({ user_id: user.id });
	const already = current.transactions.some((t) => t.description === (tx.description ?? ''));
	if (already) return;

	await api.createTransaction(tx);
}

async function main() {
	console.log(`🌱 Seeding via API at ${BACKEND_ORIGIN}`);

	// Quick connectivity check
	await api.getHealth();

	const usersBefore = await api.getUsers();
	const byEmail = new Map<string, User>((usersBefore.users ?? []).map((u) => [u.email, u]));

	for (const u of seedUsers) {
		await ensureUser(u, byEmail);
	}

	// refresh users (to get IDs)
	const usersAfter = await api.getUsers();
	const byEmailAfter = new Map<string, User>((usersAfter.users ?? []).map((u) => [u.email, u]));

	for (const tx of seedTxs) {
		await ensureTransaction(tx, byEmailAfter);
	}

	console.log('✅ Seed complete.');
	console.log('Try these in the UI:', seedUsers.map((u) => u.email).join(', '));
}

main().catch((err) => {
	console.error('❌ Seed failed:', err);
	process.exit(1);
});

