import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';
import { readFileSync, writeFileSync, mkdirSync } from 'fs';

const pkg = JSON.parse(readFileSync('package.json', 'utf-8'));

// Write version.json so the server can read and push it to clients
mkdirSync('static', { recursive: true });
writeFileSync('static/version.json', JSON.stringify({ version: pkg.version }) + '\n');

export default defineConfig({
	define: {
		__APP_VERSION__: JSON.stringify(pkg.version)
	},
	build: {
		target: ['es2020', 'safari14'],
		rollupOptions: {
			output: {
				// Merge all dependencies into one chunk to avoid circular imports
				// that trigger WebKitGTK's JSC "uninitialized variable" bug
				manualChunks(id) {
					if (id.includes('node_modules')) return 'vendor';
					if (id.includes('/src/lib/')) return 'app-lib';
				}
			}
		}
	},
	plugins: [tailwindcss(), sveltekit()],
	optimizeDeps: {
		exclude: ['chatalot-crypto-wasm']
	},
	server: {
		proxy: {
			'/api': 'http://localhost:8080',
			'/ws': {
				target: 'ws://localhost:8080',
				ws: true
			}
		}
	}
});
