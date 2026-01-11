<script lang="ts">
	import { onMount } from 'svelte';
	import { createTransaction, createUser, getDbHealth, getHealth, getTransactions, getUsers } from '$lib/api';
	import type { Transaction, User } from '$lib/types';

	let health = $state<{ ok: boolean; text: string }>({ ok: false, text: '—' });
	let dbHealth = $state<{ ok: boolean; text: string }>({ ok: false, text: '—' });
	let error = $state<string>('');

	let users = $state<User[]>([]);
	let usersLoading = $state<boolean>(false);

	let selectedUserId = $state<string>('');
	let selectedUserEmail = $state<string>('');

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
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			usersLoading = false;
		}
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
					<div>ID</div>
					<div>Email</div>
					<div>Name</div>
					<div></div>
				</div>
				{#each users as u (u.id)}
					<div class="trow">
						<code class="mono">{u.id}</code>
						<div>{u.email}</div>
						<div>{u.name}</div>
						<div class="actions">
							<button
								class="btn small"
								type="button"
								onclick={() => {
									selectedUserId = u.id;
									selectedUserEmail = u.email;
									newTx = { ...newTx, user_email: u.email };
									refreshTransactions();
								}}
							>
								Use
							</button>
						</div>
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

	<section class="card span2 txs">
		<div class="row">
			<h2>Transactions {#if selectedUserEmail}<span class="muted">for {selectedUserEmail}</span>{/if}</h2>
			<button class="btn" type="button" onclick={refreshTransactions} disabled={txLoading}>
				{txLoading ? 'Loading…' : 'Refresh'}
			</button>
		</div>

		{#if !selectedUserId}
			<p class="hint">Pick a user from the Users table to load transactions.</p>
		{:else if txs.length === 0}
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

	.btn.small {
		height: 30px;
		border-radius: 10px;
		font-size: 0.9rem;
	}

	.table {
		border: 1px solid rgba(15, 23, 42, 0.08);
		border-radius: 12px;
		overflow: hidden;
	}

	.thead,
	.trow {
		display: grid;
		grid-template-columns: 2fr 2fr 1.2fr auto;
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

	.actions {
		display: flex;
		justify-content: flex-end;
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
