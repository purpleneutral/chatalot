<script lang="ts">
	import { api } from '$lib/api/client';
	import { onMount } from 'svelte';

	let title = $state('Privacy Policy');
	let body = $state('');
	let loading = $state(true);
	let error = $state('');

	onMount(async () => {
		try {
			const doc = await api.get<{ title: string; body: string }>('/legal/privacy');
			title = doc.title;
			body = doc.body;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load privacy policy';
		} finally {
			loading = false;
		}
	});
</script>

<div class="flex min-h-screen justify-center bg-[var(--bg-primary)] p-4 sm:p-8">
	<div class="w-full max-w-3xl">
		<nav class="mb-6 flex items-center gap-3 text-sm text-[var(--text-secondary)]">
			<a href="/login" class="transition hover:text-[var(--accent)]">Login</a>
			<span>/</span>
			<span class="text-[var(--text-primary)]">{title}</span>
		</nav>

		{#if loading}
			<div class="flex justify-center py-16">
				<div class="h-8 w-8 animate-spin rounded-full border-2 border-[var(--accent)] border-t-transparent"></div>
			</div>
		{:else if error}
			<div class="rounded-lg bg-red-500/10 p-4 text-sm text-[var(--danger)]">
				{error}
			</div>
		{:else}
			<article class="prose prose-invert max-w-none rounded-2xl bg-[var(--bg-secondary)] p-6 sm:p-8">
				{@html renderMarkdown(body)}
			</article>
		{/if}
	</div>
</div>

<script lang="ts" context="module">
	function renderMarkdown(md: string): string {
		return md
			.replace(/^### (.+)$/gm, '<h3 class="mt-6 mb-2 text-lg font-semibold text-[var(--text-primary)]">$1</h3>')
			.replace(/^## (.+)$/gm, '<h2 class="mt-8 mb-3 text-xl font-bold text-[var(--text-primary)]">$1</h2>')
			.replace(/^# (.+)$/gm, '<h1 class="mb-4 text-2xl font-bold text-[var(--text-primary)]">$1</h1>')
			.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
			.replace(/\*(.+?)\*/g, '<em class="text-[var(--text-secondary)]">$1</em>')
			.replace(/^\| (.+) \|$/gm, (match) => {
				const cells = match.slice(1, -1).split('|').map(c => c.trim());
				return '<tr>' + cells.map(c => `<td class="border border-white/10 px-3 py-2 text-sm">${c}</td>`).join('') + '</tr>';
			})
			.replace(/^\|[-| ]+\|$/gm, '')
			.replace(/(<tr>.*<\/tr>\n?)+/g, (match) => {
				const rows = match.trim().split('\n');
				if (rows.length > 0) {
					const headerRow = rows[0].replace(/<td/g, '<th').replace(/<\/td>/g, '</th>');
					const bodyRows = rows.slice(1).join('\n');
					return `<table class="my-4 w-full border-collapse"><thead class="bg-white/5">${headerRow}</thead><tbody>${bodyRows}</tbody></table>`;
				}
				return match;
			})
			.replace(/^- (.+)$/gm, '<li class="ml-4 list-disc text-sm text-[var(--text-secondary)]">$1</li>')
			.replace(/(<li.*<\/li>\n?)+/g, (match) => `<ul class="my-2 space-y-1">${match}</ul>`)
			.replace(/\n\n/g, '<br/><br/>')
			.replace(/\n(?!<)/g, '\n');
	}
</script>
