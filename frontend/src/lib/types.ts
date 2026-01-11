export type ApiOk<T> = T & { message?: string };

export type User = {
	id: string;
	email: string;
	name: string;
	password?: string;
	created_at?: string;
	updated_at?: string;
};

export type Transaction = {
	id: string;
	user_id: string;
	transaction_type: 'Expense' | 'Income';
	amount: string | number;
	category:
		| 'Groceries'
		| 'Restaurant'
		| 'Housing'
		| 'Holidays'
		| 'Shopping'
		| 'Entertainment'
		| 'Other';
	description: string;
	created_at?: string;
	last_updated_at?: string;
};

export type Health = {
	status: string;
	message?: string;
	database?: string;
};

