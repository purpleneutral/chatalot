<script lang="ts">
	type Variant = 'message' | 'channel' | 'member' | 'text';

	let { variant = 'text', count = 3 }: { variant?: Variant; count?: number } = $props();
</script>

{#each Array(count) as _, i (i)}
	{#if variant === 'message'}
		<div class="mb-4 flex gap-3 px-2 py-1 animate-pulse">
			<div class="h-9 w-9 shrink-0 rounded-full bg-[var(--bg-tertiary)]"></div>
			<div class="flex-1 space-y-2 pt-1">
				<div class="h-3 w-24 rounded bg-[var(--bg-tertiary)]"></div>
				<div class="h-3 rounded bg-[var(--bg-tertiary)]" style="width: {60 + (i * 13) % 30}%"></div>
			</div>
		</div>
	{:else if variant === 'channel'}
		<div class="flex items-center gap-2 rounded-lg px-3 py-2 animate-pulse">
			<div class="h-4 w-4 rounded bg-[var(--bg-tertiary)]"></div>
			<div class="h-3 rounded bg-[var(--bg-tertiary)]" style="width: {40 + (i * 17) % 40}%"></div>
		</div>
	{:else if variant === 'member'}
		<div class="flex items-center gap-2 rounded-lg px-2 py-1.5 animate-pulse">
			<div class="h-7 w-7 shrink-0 rounded-full bg-[var(--bg-tertiary)]"></div>
			<div class="h-3 rounded bg-[var(--bg-tertiary)]" style="width: {50 + (i * 11) % 30}%"></div>
		</div>
	{:else}
		<div class="animate-pulse py-1">
			<div class="h-3 rounded bg-[var(--bg-tertiary)]" style="width: {60 + (i * 19) % 30}%"></div>
		</div>
	{/if}
{/each}
