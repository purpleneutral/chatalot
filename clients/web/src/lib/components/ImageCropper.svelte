<script lang="ts">
	import { fade } from 'svelte/transition';

	interface Props {
		imageFile: File;
		aspectRatio?: number;
		circular?: boolean;
		onConfirm: (blob: Blob) => void;
		onCancel: () => void;
	}

	let { imageFile, aspectRatio, circular = false, onConfirm, onCancel }: Props = $props();

	let canvasEl: HTMLCanvasElement | undefined = $state();
	let containerEl: HTMLDivElement | undefined = $state();

	// Image state
	let img: HTMLImageElement | null = $state(null);
	let scale = $state(1);
	let offsetX = $state(0);
	let offsetY = $state(0);
	let minScale = $state(0.1);

	// Drag state
	let dragging = $state(false);
	let dragStartX = 0;
	let dragStartY = 0;
	let dragOffsetStartX = 0;
	let dragOffsetStartY = 0;

	// Pinch state
	let lastPinchDist = 0;

	// Canvas size
	const CANVAS_W = 400;
	const CANVAS_H = 400;

	// Crop region (centered, fixed)
	let cropW = $derived(aspectRatio ? Math.min(CANVAS_W - 40, (CANVAS_H - 40) * (aspectRatio ?? 1)) : CANVAS_W - 40);
	let cropH = $derived(aspectRatio ? cropW / (aspectRatio ?? 1) : CANVAS_H - 40);
	let cropX = $derived((CANVAS_W - cropW) / 2);
	let cropY = $derived((CANVAS_H - cropH) / 2);

	let exporting = $state(false);

	$effect(() => {
		const url = URL.createObjectURL(imageFile);
		const image = new Image();
		image.onload = () => {
			img = image;
			// Fit image so it covers the crop area
			const scaleX = cropW / image.width;
			const scaleY = cropH / image.height;
			scale = Math.max(scaleX, scaleY);
			minScale = Math.min(scaleX, scaleY) * 0.5;
			// Center
			offsetX = (CANVAS_W - image.width * scale) / 2;
			offsetY = (CANVAS_H - image.height * scale) / 2;
			draw();
		};
		image.src = url;
		return () => URL.revokeObjectURL(url);
	});

	function draw() {
		if (!canvasEl || !img) return;
		const ctx = canvasEl.getContext('2d');
		if (!ctx) return;

		ctx.clearRect(0, 0, CANVAS_W, CANVAS_H);

		// Draw image
		ctx.save();
		ctx.drawImage(img, offsetX, offsetY, img.width * scale, img.height * scale);
		ctx.restore();

		// Draw dark overlay outside crop area
		ctx.save();
		ctx.fillStyle = 'rgba(0, 0, 0, 0.6)';

		if (circular) {
			// Draw full overlay, then cut out circle
			ctx.fillRect(0, 0, CANVAS_W, CANVAS_H);
			ctx.globalCompositeOperation = 'destination-out';
			ctx.beginPath();
			ctx.ellipse(cropX + cropW / 2, cropY + cropH / 2, cropW / 2, cropH / 2, 0, 0, Math.PI * 2);
			ctx.fill();
			ctx.globalCompositeOperation = 'source-over';
			// Draw circle border
			ctx.strokeStyle = 'rgba(255, 255, 255, 0.8)';
			ctx.lineWidth = 2;
			ctx.beginPath();
			ctx.ellipse(cropX + cropW / 2, cropY + cropH / 2, cropW / 2, cropH / 2, 0, 0, Math.PI * 2);
			ctx.stroke();
		} else {
			// Top
			ctx.fillRect(0, 0, CANVAS_W, cropY);
			// Bottom
			ctx.fillRect(0, cropY + cropH, CANVAS_W, CANVAS_H - cropY - cropH);
			// Left
			ctx.fillRect(0, cropY, cropX, cropH);
			// Right
			ctx.fillRect(cropX + cropW, cropY, CANVAS_W - cropX - cropW, cropH);
			// Border
			ctx.strokeStyle = 'rgba(255, 255, 255, 0.8)';
			ctx.lineWidth = 2;
			ctx.strokeRect(cropX, cropY, cropW, cropH);
			// Rule of thirds grid
			ctx.strokeStyle = 'rgba(255, 255, 255, 0.2)';
			ctx.lineWidth = 1;
			ctx.beginPath();
			ctx.moveTo(cropX + cropW / 3, cropY);
			ctx.lineTo(cropX + cropW / 3, cropY + cropH);
			ctx.moveTo(cropX + 2 * cropW / 3, cropY);
			ctx.lineTo(cropX + 2 * cropW / 3, cropY + cropH);
			ctx.moveTo(cropX, cropY + cropH / 3);
			ctx.lineTo(cropX + cropW, cropY + cropH / 3);
			ctx.moveTo(cropX, cropY + 2 * cropH / 3);
			ctx.lineTo(cropX + cropW, cropY + 2 * cropH / 3);
			ctx.stroke();
		}
		ctx.restore();
	}

	$effect(() => {
		// Redraw when scale/offset changes
		scale; offsetX; offsetY;
		draw();
	});

	function handlePointerDown(e: PointerEvent) {
		if (e.button !== 0) return;
		dragging = true;
		dragStartX = e.clientX;
		dragStartY = e.clientY;
		dragOffsetStartX = offsetX;
		dragOffsetStartY = offsetY;
		(e.target as HTMLElement).setPointerCapture(e.pointerId);
	}

	function handlePointerMove(e: PointerEvent) {
		if (!dragging) return;
		offsetX = dragOffsetStartX + (e.clientX - dragStartX);
		offsetY = dragOffsetStartY + (e.clientY - dragStartY);
	}

	function handlePointerUp() {
		dragging = false;
	}

	function handleWheel(e: WheelEvent) {
		e.preventDefault();
		const delta = e.deltaY > 0 ? 0.95 : 1.05;
		zoom(delta, e.offsetX, e.offsetY);
	}

	function handleTouchStart(e: TouchEvent) {
		if (e.touches.length === 2) {
			e.preventDefault();
			const dx = e.touches[0].clientX - e.touches[1].clientX;
			const dy = e.touches[0].clientY - e.touches[1].clientY;
			lastPinchDist = Math.hypot(dx, dy);
		}
	}

	function handleTouchMove(e: TouchEvent) {
		if (e.touches.length === 2) {
			e.preventDefault();
			const dx = e.touches[0].clientX - e.touches[1].clientX;
			const dy = e.touches[0].clientY - e.touches[1].clientY;
			const dist = Math.hypot(dx, dy);
			if (lastPinchDist > 0) {
				const delta = dist / lastPinchDist;
				zoom(delta, CANVAS_W / 2, CANVAS_H / 2);
			}
			lastPinchDist = dist;
		}
	}

	function handleTouchEnd() {
		lastPinchDist = 0;
	}

	function zoom(factor: number, cx: number, cy: number) {
		const newScale = Math.max(minScale, Math.min(scale * factor, 10));
		// Zoom towards cursor position
		offsetX = cx - (cx - offsetX) * (newScale / scale);
		offsetY = cy - (cy - offsetY) * (newScale / scale);
		scale = newScale;
	}

	function zoomIn() { zoom(1.2, CANVAS_W / 2, CANVAS_H / 2); }
	function zoomOut() { zoom(0.8, CANVAS_W / 2, CANVAS_H / 2); }

	function resetView() {
		if (!img) return;
		const scaleX = cropW / img.width;
		const scaleY = cropH / img.height;
		scale = Math.max(scaleX, scaleY);
		offsetX = (CANVAS_W - img.width * scale) / 2;
		offsetY = (CANVAS_H - img.height * scale) / 2;
	}

	async function handleConfirm() {
		if (!img || exporting) return;
		exporting = true;

		try {
			const output = document.createElement('canvas');
			const outW = Math.round(cropW / scale);
			const outH = Math.round(cropH / scale);
			// Limit output to reasonable size (max 1024px on either axis)
			const maxDim = 1024;
			const finalScale = Math.min(1, maxDim / Math.max(outW, outH));
			output.width = Math.round(outW * finalScale);
			output.height = Math.round(outH * finalScale);

			const ctx = output.getContext('2d');
			if (!ctx) return;

			// Calculate source region in image coordinates
			const sx = (cropX - offsetX) / scale;
			const sy = (cropY - offsetY) / scale;
			const sw = cropW / scale;
			const sh = cropH / scale;

			ctx.drawImage(img, sx, sy, sw, sh, 0, 0, output.width, output.height);

			const blob = await new Promise<Blob | null>((resolve) => {
				// Use webp for smaller output, fall back to png for transparency
				output.toBlob(resolve, 'image/webp', 0.9);
			});

			if (blob) onConfirm(blob);
		} finally {
			exporting = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') onCancel();
		else if (e.key === '+' || e.key === '=') zoomIn();
		else if (e.key === '-') zoomOut();
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_no_noninteractive_element_interactions a11y_click_events_have_key_events -->
<div
	class="fixed inset-0 z-[100] flex items-center justify-center bg-black/70 backdrop-blur-sm"
	role="dialog"
	aria-modal="true"
	aria-label="Crop image"
	onclick={(e) => { if (e.target === e.currentTarget) onCancel(); }}
	transition:fade={{ duration: 150 }}
>
	<div class="flex flex-col items-center gap-4 rounded-2xl bg-[var(--bg-secondary)] p-6 shadow-xl" bind:this={containerEl}>
		<h3 class="text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Crop Image</h3>

		<canvas
			bind:this={canvasEl}
			width={CANVAS_W}
			height={CANVAS_H}
			class="rounded-lg cursor-grab touch-none {dragging ? 'cursor-grabbing' : ''}"
			style="max-width: 90vw; max-height: 60vh; width: {CANVAS_W}px; height: {CANVAS_H}px;"
			onpointerdown={handlePointerDown}
			onpointermove={handlePointerMove}
			onpointerup={handlePointerUp}
			onwheel={handleWheel}
			ontouchstart={handleTouchStart}
			ontouchmove={handleTouchMove}
			ontouchend={handleTouchEnd}
		></canvas>

		<!-- Zoom controls -->
		<div class="flex items-center gap-3">
			<button
				onclick={zoomOut}
				class="rounded-lg bg-white/10 p-2 text-[var(--text-secondary)] transition hover:bg-white/15 hover:text-[var(--text-primary)]"
				title="Zoom out (-)"
			>
				<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/><line x1="8" y1="11" x2="14" y2="11"/></svg>
			</button>
			<button
				onclick={resetView}
				class="rounded-lg bg-white/10 px-3 py-2 text-xs font-medium text-[var(--text-secondary)] transition hover:bg-white/15 hover:text-[var(--text-primary)]"
				title="Reset view"
			>
				Reset
			</button>
			<button
				onclick={zoomIn}
				class="rounded-lg bg-white/10 p-2 text-[var(--text-secondary)] transition hover:bg-white/15 hover:text-[var(--text-primary)]"
				title="Zoom in (+)"
			>
				<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/><line x1="11" y1="8" x2="11" y2="14"/><line x1="8" y1="11" x2="14" y2="11"/></svg>
			</button>
		</div>

		<p class="text-xs text-[var(--text-secondary)]">Drag to reposition. Scroll or pinch to zoom.</p>

		<!-- Actions -->
		<div class="flex gap-3">
			<button
				onclick={onCancel}
				class="rounded-lg border border-white/10 px-4 py-2 text-sm font-medium text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
			>
				Cancel
			</button>
			<button
				onclick={handleConfirm}
				disabled={exporting}
				class="rounded-lg bg-[var(--accent)] px-4 py-2 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:cursor-not-allowed disabled:opacity-50"
			>
				{exporting ? 'Cropping...' : 'Confirm'}
			</button>
		</div>
	</div>
</div>
