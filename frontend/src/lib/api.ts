import type { Health, Transaction, User } from './types';

async function readJson(res: Response) {
	const text = await res.text();
	if (!text) return null;
	try {
		return JSON.parse(text);
	} catch {
		return text;
	}
}

function joinUrl(baseUrl: string, path: string) {
	if (!baseUrl) return path;
	return `${baseUrl.replace(/\/$/, '')}${path.startsWith('/') ? '' : '/'}${path}`;
}

async function api<T>(baseUrl: string, path: string, init?: RequestInit): Promise<T> {
	const res = await fetch(joinUrl(baseUrl, path), {
		headers: { 'content-type': 'application/json', ...(init?.headers || {}) },
		...init
	});

	if (!res.ok) {
		const body = await readJson(res);
		throw new Error(
			`HTTP ${res.status} ${res.statusText}${body ? ` — ${typeof body === 'string' ? body : JSON.stringify(body)}` : ''}`
		);
	}

	return (await readJson(res)) as T;
}

export type ApiClient = ReturnType<typeof createApiClient>;

export function createApiClient(baseUrl = '') {
	return {
		getHealth: async () => api<Health>(baseUrl, '/health'),
		getDbHealth: async () => api<Health>(baseUrl, '/health/db'),

		createUser: async (input: { email: string; name: string; password: string }) =>
			api<{ message: string; name: string }>(baseUrl, '/api/users', {
				method: 'POST',
				body: JSON.stringify(input)
			}),

		getUsers: async () => api<{ message: string; users: User[] }>(baseUrl, '/api/users'),

		createTransaction: async (input: {
			user_email: string;
			transaction_type: 'Expense' | 'Income';
			amount: number;
			category?: string;
			description?: string;
		}) =>
			api<{ message: string }>(baseUrl, '/api/transactions', {
				method: 'POST',
				body: JSON.stringify(input)
			}),

		getTransactions: async (params: {
			user_id?: string;
			category?: string;
			transaction_type?: string;
			amount_min?: string;
			amount_max?: string;
			start_timestamp?: string;
			end_timestamp?: string;
		}) => {
			const qs = new URLSearchParams();
			for (const [k, v] of Object.entries(params)) {
				if (v === undefined || v === '') continue;
				qs.set(k, String(v));
			}

			const data = await api<Record<string, unknown>>(
				baseUrl,
				`/api/transactions${qs.size ? `?${qs}` : ''}`
			);

			// Backend currently responds with { message, users: [...] } (typo), so accept both shapes.
			const items = (data['transactions'] || data['users'] || []) as Transaction[];
			return { message: (data['message'] as string) || '', transactions: items };
		},
		getAmount: async (params: {
			user_id?: string;
			category?: string;
			transaction_type?: string;
			start_timestamp?: string;
			end_timestamp?: string;
		}) => {
			const qs = new URLSearchParams();
			for (const [k, v] of Object.entries(params)) {
				if (v === undefined || v === '') continue;
				qs.set(k, String(v));
			}

			const data = await api<Record<string, unknown>>(
				baseUrl,
				`/api/transactions/amount${qs.size ? `?${qs}` : ''}`
			);

			return { message: (data['message'] as string) || '', amount: data['amount'] as number };
		} 
	};
}

// Browser default (relative paths, uses Vite dev proxy)
const defaultClient = createApiClient('');

export const getHealth = defaultClient.getHealth;
export const getDbHealth = defaultClient.getDbHealth;
export const createUser = defaultClient.createUser;
export const getUsers = defaultClient.getUsers;
export const createTransaction = defaultClient.createTransaction;
export const getTransactions = defaultClient.getTransactions;
export const getAmount = defaultClient.getAmount;

