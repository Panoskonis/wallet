<script lang="ts">
	import { onMount } from 'svelte';
	import { createTransaction, createUser, getDbHealth, getHealth, getTransactions, getUsers, getAmount } from '$lib/api';
	import type { Transaction, User } from '$lib/types';

	let health = $state<{ ok: boolean; text: string }>({ ok: false, text: '—' });
	let dbHealth = $state<{ ok: boolean; text: string }>({ ok: false, text: '—' });
	let error = $state<string>('');

	let users = $state<User[]>([]);
	let usersLoading = $state<boolean>(false);
	let userAmounts = $state<Record<string, number | null>>({});
	let userAmountsLoading = $state<Record<string, boolean>>({});

	let userExpenses = $state<Record<string, number | null>>({});
	let userExpensesLoading = $state<Record<string, boolean>>({});

	let selectedUserId = $state<string>('');
	let selectedUserEmail = $state<string>('');

	let userSearchInput = $state<string>('');
	let showUserSuggestions = $state<boolean>(false);
	let focusedSuggestionIndex = $state<number>(-1);

	let txs = $state<Transaction[]>([]);
	let txLoading = $state<boolean>(false);

	let newUser = $state({ email: '', name: '', password: '' });
	let newTx = $state({
		user_email: '',
		transaction_type: 'Expense' as const,
		amount: 0,
		category: 'Other',
		description: ''
	});

	let amountLoading = $state<boolean>(false);
	let userAmount = $state<number>(0);

	async function refreshHealth() {
		error = '';
		try {
			const h = await getHealth();
			health = { ok: true, text: `${h.status}${h.message ? ` — ${h.message}` : ''}` };
		} catch (e) {
			health = { ok: false, text: e instanceof Error ? e.message : String(e) };
		}

		try {
			const h = await getDbHealth();
			dbHealth = { ok: true, text: `${h.status}${h.database ? ` — ${h.database}` : ''}` };
		} catch (e) {
			dbHealth = { ok: false, text: e instanceof Error ? e.message : String(e) };
		}
	}

	async function refreshUsers() {
		usersLoading = true;
		error = '';
		try {
			const res = await getUsers();
			users = res.users || [];
			await refreshUserAmounts(users);
			await refreshUserExpenses(users);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			usersLoading = false;
		}
	}

	async function refreshUserAmounts(list: User[] = users) {
		userAmountsLoading = {};
		const nextAmounts: Record<string, number | null> = {};
		const requests = list.map(async (u) => {
			userAmountsLoading[u.id] = true;
			try {
				const res = await getAmount({ user_id: u.id });
				nextAmounts[u.id] = res.amount ?? 0;
			} catch {
				nextAmounts[u.id] = null;
			} finally {
				userAmountsLoading[u.id] = false;
			}
		});
		await Promise.all(requests);
		userAmounts = { ...userAmounts, ...nextAmounts };
	}

	async function refreshUserExpenses(list: User[] = users) {
		userExpensesLoading = {};
		const nextExpenses: Record<string, number | null> = {};
		const requests = list.map(async (u) => {
			userExpensesLoading[u.id] = true;
			try {
				const res = await getAmount({ user_id: u.id, transaction_type: 'Expense' });
				nextExpenses[u.id] = res.amount ?? 0;
			} catch (e) {
				error = e instanceof Error ? e.message : String(e);
				nextExpenses[u.id] = null;
			} finally {
				userExpensesLoading[u.id] = false;
			}
		});
		await Promise.all(requests);
		userExpenses = { ...userExpenses, ...nextExpenses };
	}

	function displayAmount(userId: string) {
		const amount = userAmounts[userId];
		if (userAmountsLoading[userId]) return 'Loading…';
		if (amount == null) return '—';
		return amount.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
	}

	function displayExpenses(userId: string) {
		const expenses = userExpenses[userId];
		if (userExpensesLoading[userId]) return 'Loading…';
		if (expenses == null) return '—';
		return expenses.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
	}

	async function refreshTransactions() {
		txLoading = true;
		error = '';
		try {
			const res = await getTransactions({ user_id: selectedUserId || undefined });
			txs = res.transactions || [];
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			txLoading = false;
		}
	}

	async function refreshAmount() {
		amountLoading = true;
		error = '';
		try {
			const res = await getAmount({user_id: selectedUserId || undefined});
			userAmount = res.amount || 0;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			amountLoading = false;
		}
	}

	function selectUser(user: User) {
		selectedUserId = user.id;
		selectedUserEmail = user.email;
		userSearchInput = user.email;
		showUserSuggestions = false;
		focusedSuggestionIndex = -1;
		newTx = { ...newTx, user_email: user.email };
		refreshTransactions();
		refreshAmount();
	}

	function getFilteredUsers(): User[] {
		if (!userSearchInput.trim()) return users;
		const query = userSearchInput.toLowerCase();
		return users.filter(
			(u) =>
				u.email.toLowerCase().includes(query) ||
				u.name.toLowerCase().includes(query) ||
				u.id.toLowerCase().includes(query)
		);
	}

	function handleUserInputKeydown(e: KeyboardEvent) {
		const filtered = getFilteredUsers();
		if (e.key === 'ArrowDown') {
			e.preventDefault();
			focusedSuggestionIndex = Math.min(focusedSuggestionIndex + 1, filtered.length - 1);
			showUserSuggestions = true;
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			focusedSuggestionIndex = Math.max(focusedSuggestionIndex - 1, -1);
		} else if (e.key === 'Enter' && focusedSuggestionIndex >= 0 && filtered[focusedSuggestionIndex]) {
			e.preventDefault();
			selectUser(filtered[focusedSuggestionIndex]);
		} else if (e.key === 'Escape') {
			showUserSuggestions = false;
			focusedSuggestionIndex = -1;
		}
	}

	async function submitUser() {
		error = '';
		try {
			await createUser(newUser);
			newUser = { email: '', name: '', password: '' };
			await refreshUsers();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		}
	}

	async function submitTx() {
		error = '';
		try {
			await createTransaction({
				user_email: newTx.user_email,
				transaction_type: newTx.transaction_type,
				amount: Number(newTx.amount),
				category: newTx.category || undefined,
				description: newTx.description || undefined
			});
			newTx = { ...newTx, amount: 0, description: '' };
			await refreshTransactions();
			await refreshAmount();
			await refreshUserAmounts(users);
			await refreshUserExpenses(users);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		}
	}

	onMount(async () => {
		// Client-side only so Vite proxy handles /api/* and /health*
		await refreshHealth();
		await refreshUsers();
	});
</script>

<div class="grid">
	<section class="card">
		<h2>Backend status</h2>
		<div class="kv">
			<div class="k">/health</div>
			<div class="v">
				<span class:ok={health.ok} class:bad={!health.ok}>{health.text}</span>
			</div>
			<div class="k">/health/db</div>
			<div class="v">
				<span class:ok={dbHealth.ok} class:bad={!dbHealth.ok}>{dbHealth.text}</span>
			</div>
		</div>
		<button class="btn" type="button" onclick={refreshHealth}>Refresh</button>
		<p class="hint">
			Dev calls go to <code>/api/*</code> and are proxied to your Rust server (default
			<code>http://localhost:3000</code>). Override with <code>VITE_BACKEND_ORIGIN</code>.
		</p>
	</section>

	<section class="card">
		<h2>Create user</h2>
		<form
			class="form"
			onsubmit={(e) => {
				e.preventDefault();
				submitUser();
			}}
		>
			<label>
				<span>Email</span>
				<input required autocomplete="email" bind:value={newUser.email} placeholder="alice@example.com" />
			</label>
			<label>
				<span>Name</span>
				<input required bind:value={newUser.name} placeholder="Alice" />
			</label>
			<label>
				<span>Password</span>
				<input required type="password" autocomplete="new-password" bind:value={newUser.password} />
			</label>
			<button class="btn primary" type="submit">Create user</button>
		</form>
	</section>

	<section class="card span2">
		<div class="row">
			<h2>Users</h2>
			<button class="btn" type="button" onclick={refreshUsers} disabled={usersLoading}>
				{usersLoading ? 'Loading…' : 'Refresh'}
			</button>
		</div>

		{#if users.length === 0}
			<p class="hint">No users yet.</p>
		{:else}
			<div class="table">
				<div class="thead">
					<div>Email</div>
					<div>Name</div>
					<div>Amount</div>
					<div>Expenses</div>
				</div>
				{#each users as u (u.id)}
					<div class="trow">
						<div>{u.email}</div>
						<div>{u.name}</div>
						<div>{displayAmount(u.id)}</div>
						<div>{displayExpenses(u.id)}</div>
					</div>
				{/each}
			</div>
		{/if}
	</section>

	<section class="card">
		<h2>Create transaction</h2>
		<form
			class="form"
			onsubmit={(e) => {
				e.preventDefault();
				submitTx();
			}}
		>
			<label>
				<span>User email</span>
				<input required bind:value={newTx.user_email} placeholder="alice@example.com" />
			</label>

			<div class="cols">
				<label>
					<span>Type</span>
					<select bind:value={newTx.transaction_type}>
						<option value="Expense">Expense</option>
						<option value="Income">Income</option>
					</select>
				</label>
				<label>
					<span>Amount</span>
					<input required type="number" step="0.01" bind:value={newTx.amount} />
				</label>
			</div>

			<div class="cols">
				<label>
					<span>Category</span>
					<select bind:value={newTx.category}>
						<option>Groceries</option>
						<option>Restaurant</option>
						<option>Housing</option>
						<option>Holidays</option>
						<option>Shopping</option>
						<option>Entertainment</option>
						<option>Other</option>
					</select>
				</label>
				<label>
					<span>Description</span>
					<input bind:value={newTx.description} placeholder="Optional" />
				</label>
			</div>

			<button class="btn primary" type="submit">Create transaction</button>
		</form>
		<p class="hint">
			Transactions are stored by <code>user_id</code>; the backend create endpoint takes <code>user_email</code>
			and resolves it.
		</p>
	</section>

	<section class="card span2">
		<h2>Select User</h2>
		<div class="autocomplete-wrapper">
			<input
				type="text"
				class="autocomplete-input"
				role="combobox"
				aria-expanded={showUserSuggestions}
				aria-autocomplete="list"
				aria-controls="user-suggestions"
				placeholder="Type to search users by email, name, or ID..."
				bind:value={userSearchInput}
				onfocus={() => (showUserSuggestions = true)}
				onblur={() => {
					// Delay to allow click events on suggestions
					setTimeout(() => (showUserSuggestions = false), 200);
				}}
				onkeydown={handleUserInputKeydown}
			/>
			{#if showUserSuggestions && getFilteredUsers().length > 0}
				<div class="autocomplete-dropdown" id="user-suggestions" role="listbox">
					{#each getFilteredUsers() as user, index (user.id)}
						<div
							class="autocomplete-item"
							class:focused={index === focusedSuggestionIndex}
							role="option"
							aria-selected={selectedUserId === user.id}
							tabindex="0"
							onclick={() => selectUser(user)}
							onkeydown={(e) => {
								if (e.key === 'Enter' || e.key === ' ') {
									e.preventDefault();
									selectUser(user);
								}
							}}
							onmouseenter={() => (focusedSuggestionIndex = index)}
						>
							<div class="autocomplete-item-email">{user.email}</div>
							<div class="autocomplete-item-name">{user.name}</div>
							<code class="autocomplete-item-id">{user.id}</code>
						</div>
					{/each}
				</div>
			{/if}
		</div>
		{#if selectedUserId && selectedUserEmail}
			<p class="hint">Viewing: <strong>{selectedUserEmail}</strong></p>
		{/if}
	</section>

	{#if selectedUserId}
		<section class="card span2">
			<div class="row">
				<h2>Total Amount <span class="muted">— {selectedUserEmail}</span></h2>
				<button class="btn" type="button" onclick={refreshAmount} disabled={amountLoading}>
					{amountLoading ? 'Loading…' : 'Refresh'}
				</button>
			</div>
			<div class="amount-display">
				<div class="amount-value" class:positive={userAmount > 0} class:negative={userAmount < 0}>
					{#if amountLoading}
						<span class="loading">Loading…</span>
					{:else}
						<span>
							{userAmount >= 0 ? '+' : ''}
							{userAmount.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
						</span>
					{/if}
				</div>
				<p class="hint">Sum of all transactions (Income - Expense)</p>
			</div>
		</section>

		<section class="card span2 txs">
			<div class="row">
				<h2>Transactions <span class="muted">— {selectedUserEmail}</span></h2>
				<button class="btn" type="button" onclick={refreshTransactions} disabled={txLoading}>
					{txLoading ? 'Loading…' : 'Refresh'}
				</button>
			</div>

			{#if txs.length === 0}
				<p class="hint">No transactions yet.</p>
			{:else}
				<div class="table">
					<div class="thead">
						<div>ID</div>
						<div>Type</div>
						<div>Amount</div>
						<div>Category</div>
						<div>Description</div>
					</div>
					{#each txs as t (t.id)}
						<div class="trow">
							<code class="mono">{t.id}</code>
							<div>{t.transaction_type}</div>
							<div class="mono">{t.amount}</div>
							<div>{t.category}</div>
							<div>{t.description}</div>
						</div>
					{/each}
				</div>
			{/if}
		</section>
	{/if}
</div>

{#if error}
	<div class="error">
		<strong>Error:</strong> {error}
	</div>
{/if}

<style>
	h2 {
		margin: 0 0 12px;
		font-size: 1.05rem;
	}

	code {
		background: rgba(15, 23, 42, 0.06);
		border: 1px solid rgba(15, 23, 42, 0.08);
		border-radius: 8px;
		padding: 1px 6px;
	}

	.mono {
		font-family:
			ui-monospace,
			SFMono-Regular,
			Menlo,
			Monaco,
			Consolas,
			Liberation Mono,
			Courier New,
			monospace;
		font-size: 0.9rem;
	}

	.grid {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 14px;
	}

	.span2 {
		grid-column: span 2;
	}

	.card {
		background: white;
		border: 1px solid rgba(15, 23, 42, 0.08);
		border-radius: 14px;
		padding: 14px;
		box-shadow: 0 16px 40px rgba(15, 23, 42, 0.06);
	}

	.kv {
		display: grid;
		grid-template-columns: 110px 1fr;
		gap: 8px 12px;
		align-items: center;
		margin: 0 0 12px;
	}

	.k {
		color: rgba(15, 23, 42, 0.65);
		font-size: 0.95rem;
	}

	.ok {
		color: #166534;
	}
	.bad {
		color: #991b1b;
	}

	.row {
		display: flex;
		justify-content: space-between;
		gap: 12px;
		align-items: center;
		margin-bottom: 10px;
	}

	.form {
		display: grid;
		gap: 10px;
	}

	label {
		display: grid;
		gap: 6px;
	}

	label > span {
		font-size: 0.9rem;
		color: rgba(15, 23, 42, 0.75);
	}

	input,
	select {
		height: 38px;
		border-radius: 10px;
		border: 1px solid rgba(15, 23, 42, 0.14);
		padding: 0 10px;
		outline: none;
		background: white;
	}

	input:focus,
	select:focus {
		border-color: rgba(37, 99, 235, 0.6);
		box-shadow: 0 0 0 4px rgba(37, 99, 235, 0.15);
	}

	.cols {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 10px;
	}

	.btn {
		height: 38px;
		border-radius: 10px;
		border: 1px solid rgba(15, 23, 42, 0.14);
		background: white;
		padding: 0 12px;
		cursor: pointer;
	}

	.btn:hover {
		background: rgba(15, 23, 42, 0.03);
	}

	.btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.btn.primary {
		background: #2563eb;
		border-color: rgba(37, 99, 235, 0.2);
		color: white;
	}

	.btn.primary:hover {
		background: #1d4ed8;
	}

	.table {
		border: 1px solid rgba(15, 23, 42, 0.08);
		border-radius: 12px;
		overflow: hidden;
	}

	.thead,
	.trow {
		display: grid;
		grid-template-columns: 2fr 2fr 1.2fr 1.2fr;
		gap: 10px;
		align-items: center;
		padding: 10px 12px;
	}

	.thead {
		background: rgba(15, 23, 42, 0.03);
		font-size: 0.9rem;
		color: rgba(15, 23, 42, 0.65);
	}

	.trow {
		border-top: 1px solid rgba(15, 23, 42, 0.06);
	}

	/* Transactions table is 5 columns; override only for that section */
	.txs .table .thead,
	.txs .table .trow {
		grid-template-columns: 2fr 0.9fr 0.9fr 1fr 2fr;
	}

	.hint {
		margin: 10px 0 0;
		color: rgba(15, 23, 42, 0.65);
		font-size: 0.92rem;
	}

	.autocomplete-wrapper {
		position: relative;
		width: 100%;
	}

	.autocomplete-input {
		width: 100%;
		height: 38px;
		border-radius: 10px;
		border: 1px solid rgba(15, 23, 42, 0.14);
		padding: 0 10px;
		outline: none;
		background: white;
		font-size: 1rem;
	}

	.autocomplete-input:focus {
		border-color: rgba(37, 99, 235, 0.6);
		box-shadow: 0 0 0 4px rgba(37, 99, 235, 0.15);
	}

	.autocomplete-dropdown {
		position: absolute;
		top: calc(100% + 4px);
		left: 0;
		right: 0;
		background: white;
		border: 1px solid rgba(15, 23, 42, 0.14);
		border-radius: 10px;
		box-shadow: 0 4px 12px rgba(15, 23, 42, 0.1);
		max-height: 300px;
		overflow-y: auto;
		z-index: 100;
	}

	.autocomplete-item {
		padding: 12px;
		cursor: pointer;
		border-bottom: 1px solid rgba(15, 23, 42, 0.06);
		transition: background-color 0.15s;
	}

	.autocomplete-item:last-child {
		border-bottom: none;
	}

	.autocomplete-item:hover,
	.autocomplete-item.focused {
		background: rgba(37, 99, 235, 0.08);
	}

	.autocomplete-item:focus {
		outline: 2px solid rgba(37, 99, 235, 0.5);
		outline-offset: -2px;
	}

	.autocomplete-item-email {
		font-weight: 500;
		color: rgba(15, 23, 42, 0.9);
		margin-bottom: 4px;
	}

	.autocomplete-item-name {
		font-size: 0.9rem;
		color: rgba(15, 23, 42, 0.7);
		margin-bottom: 4px;
	}

	.autocomplete-item-id {
		font-size: 0.8rem;
		color: rgba(15, 23, 42, 0.5);
		font-family:
			ui-monospace,
			SFMono-Regular,
			Menlo,
			Monaco,
			Consolas,
			Liberation Mono,
			Courier New,
			monospace;
	}

	.amount-display {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 24px;
		min-height: 120px;
	}

	.amount-value {
		font-size: 2.5rem;
		font-weight: 700;
		font-family:
			ui-monospace,
			SFMono-Regular,
			Menlo,
			Monaco,
			Consolas,
			Liberation Mono,
			Courier New,
			monospace;
		margin-bottom: 8px;
		color: rgba(15, 23, 42, 0.8);
	}

	.amount-value.positive {
		color: #166534;
	}

	.amount-value.negative {
		color: #991b1b;
	}

	.amount-value .loading {
		font-size: 1.2rem;
		font-weight: 400;
		color: rgba(15, 23, 42, 0.5);
	}

	.error {
		margin-top: 14px;
		padding: 10px 12px;
		border-radius: 12px;
		background: rgba(220, 38, 38, 0.08);
		border: 1px solid rgba(220, 38, 38, 0.25);
		color: #7f1d1d;
	}

	@media (max-width: 900px) {
		.grid {
			grid-template-columns: 1fr;
		}
		.span2 {
			grid-column: auto;
		}
		.cols {
			grid-template-columns: 1fr;
		}
	}
</style>
