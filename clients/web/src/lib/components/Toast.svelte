<script lang="ts">
	import { toastStore } from '$lib/stores/toast.svelte';

	const typeStyles: Record<string, string> = {
		success: 'border-green-500/30 bg-green-500/10 text-green-400',
		error: 'border-red-500/30 bg-red-500/10 text-red-400',
		info: 'border-[var(--accent)]/30 bg-[var(--accent)]/10 text-[var(--accent)]'
	};

	const typeIcons: Record<string, string> = {
		success: 'M5 13l4 4L19 7',
		error: 'M6 18L18 6M6 6l12 12',
		info: 'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z'
	};
</script>

{#if toastStore.toasts.length > 0}
	<div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2">
		{#each toastStore.toasts as toast (toast.id)}
			<div
				class="flex items-center gap-3 rounded-lg border px-4 py-3 shadow-xl backdrop-blur-sm transition-all animate-in {typeStyles[toast.type]}"
				role="alert"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-5 w-5 shrink-0"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<path d={typeIcons[toast.type]} />
				</svg>
				<span class="text-sm font-medium">{toast.message}</span>
				<button
					onclick={() => toastStore.dismiss(toast.id)}
					class="ml-2 shrink-0 rounded p-0.5 opacity-60 transition hover:opacity-100"
					aria-label="Dismiss notification"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
					</svg>
				</button>
			</div>
		{/each}
	</div>
{/if}

<style>
	.animate-in {
		animation: slide-in 0.3s ease-out;
	}

	@keyframes slide-in {
		from {
			opacity: 0;
			transform: translateX(100%);
		}
		to {
			opacity: 1;
			transform: translateX(0);
		}
	}
</style>
