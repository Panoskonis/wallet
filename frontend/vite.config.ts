import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig, loadEnv } from 'vite';

export default defineConfig(({ mode }) => {
	const env = loadEnv(mode, process.cwd(), '');
	const backendOrigin = env.VITE_BACKEND_ORIGIN || 'http://localhost:3000';

	return {
		plugins: [sveltekit()],
		server: {
			proxy: {
				// Your Rust backend listens on /api/* and /health* (default :3000)
				'/api': {
					target: backendOrigin,
					changeOrigin: true
				},
				'/health': {
					target: backendOrigin,
					changeOrigin: true
				}
			}
		}
	};
});
