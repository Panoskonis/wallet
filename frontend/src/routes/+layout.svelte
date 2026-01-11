<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<meta name="viewport" content="width=device-width, initial-scale=1" />
</svelte:head>

<div class="app">
	<header class="header">
		<div class="brand">
			<div class="dot"></div>
			<strong>Wallet</strong>
			<span class="muted">SvelteKit + Rust</span>
		</div>
	</header>
	<main class="main">
		{@render children()}
	</main>
</div>

<style>
	:global(body) {
		margin: 0;
		font-family:
			ui-sans-serif,
			system-ui,
			-apple-system,
			Segoe UI,
			Roboto,
			Ubuntu,
			Cantarell,
			Noto Sans,
			sans-serif;
		color: #0b1220;
		background: #f6f7fb;
	}

	:global(a) {
		color: inherit;
	}

	.app {
		min-height: 100vh;
		display: grid;
		grid-template-rows: auto 1fr;
	}

	.header {
		position: sticky;
		top: 0;
		z-index: 10;
		background: rgba(255, 255, 255, 0.85);
		backdrop-filter: blur(10px);
		border-bottom: 1px solid rgba(15, 23, 42, 0.08);
	}

	.brand {
		max-width: 1100px;
		margin: 0 auto;
		padding: 14px 16px;
		display: flex;
		gap: 10px;
		align-items: center;
	}

	.dot {
		width: 10px;
		height: 10px;
		border-radius: 999px;
		background: linear-gradient(135deg, #2563eb, #22c55e);
		box-shadow: 0 8px 18px rgba(37, 99, 235, 0.35);
	}

	.muted {
		color: rgba(15, 23, 42, 0.6);
		font-size: 0.95rem;
	}

	.main {
		max-width: 1100px;
		width: 100%;
		margin: 0 auto;
		padding: 18px 16px 48px;
	}
</style>
