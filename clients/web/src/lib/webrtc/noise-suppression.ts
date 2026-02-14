/// Noise suppression pipeline using @sapphi-red/web-noise-suppressor.
/// Provides three suppression tiers via AudioWorklet: NoiseGate, Speex (DSP), RNNoise (ML).

import type { NoiseSuppression } from '$lib/stores/preferences.svelte';
import {
	NoiseGateWorkletNode,
	RnnoiseWorkletNode,
	SpeexWorkletNode,
	loadRnnoise,
	loadSpeex
} from '@sapphi-red/web-noise-suppressor';

// Vite ?url imports for worklet processors and WASM binaries
import noiseGateWorkletUrl from '@sapphi-red/web-noise-suppressor/noiseGateWorklet.js?url';
import rnnoiseWorkletUrl from '@sapphi-red/web-noise-suppressor/rnnoiseWorklet.js?url';
import rnnoiseWasmUrl from '@sapphi-red/web-noise-suppressor/rnnoise.wasm?url';
import rnnoiseWasmSimdUrl from '@sapphi-red/web-noise-suppressor/rnnoise_simd.wasm?url';
import speexWorkletUrl from '@sapphi-red/web-noise-suppressor/speexWorklet.js?url';
import speexWasmUrl from '@sapphi-red/web-noise-suppressor/speex.wasm?url';

// Cache loaded WASM binaries so we only fetch once
let rnnoiseWasm: ArrayBuffer | null = null;
let speexWasm: ArrayBuffer | null = null;

// Track which worklet modules have been registered on a given AudioContext
const registeredModules = new WeakSet<AudioContext>();

/// Convert data: URLs to blob: URLs so AudioWorklet modules pass CSP checks.
/// CSP allows blob: but not data: in script-src.
function toBlobUrl(url: string): string {
	if (!url.startsWith('data:')) return url;
	const [header, base64] = url.split(',');
	const mime = header.split(':')[1].split(';')[0];
	const binary = atob(base64);
	const bytes = new Uint8Array(binary.length);
	for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
	return URL.createObjectURL(new Blob([bytes], { type: mime }));
}

interface SuppressionState {
	source: MediaStreamAudioSourceNode;
	workletNode: AudioWorkletNode;
	destination: MediaStreamAudioDestinationNode;
	level: NoiseSuppression;
}

let currentState: SuppressionState | null = null;

async function ensureWorkletModules(ctx: AudioContext): Promise<void> {
	if (registeredModules.has(ctx)) return;
	await Promise.all([
		ctx.audioWorklet.addModule(toBlobUrl(noiseGateWorkletUrl)),
		ctx.audioWorklet.addModule(toBlobUrl(rnnoiseWorkletUrl)),
		ctx.audioWorklet.addModule(toBlobUrl(speexWorkletUrl))
	]);
	registeredModules.add(ctx);
}

async function createWorkletNode(
	ctx: AudioContext,
	level: NoiseSuppression
): Promise<AudioWorkletNode> {
	switch (level) {
		case 'noise-gate':
			return new NoiseGateWorkletNode(ctx, {
				openThreshold: -50,
				closeThreshold: -55,
				holdMs: 250,
				maxChannels: 1
			});
		case 'standard': {
			if (!speexWasm) {
				speexWasm = await loadSpeex({ url: speexWasmUrl });
			}
			return new SpeexWorkletNode(ctx, { maxChannels: 1, wasmBinary: speexWasm });
		}
		case 'maximum': {
			if (!rnnoiseWasm) {
				rnnoiseWasm = await loadRnnoise({ url: rnnoiseWasmUrl, simdUrl: rnnoiseWasmSimdUrl });
			}
			return new RnnoiseWorkletNode(ctx, { maxChannels: 1, wasmBinary: rnnoiseWasm });
		}
		default:
			throw new Error(`Invalid suppression level: ${level}`);
	}
}

/// Apply noise suppression to a raw microphone stream.
/// Returns the processed MediaStream with a clean audio track.
export async function applyNoiseSuppression(
	ctx: AudioContext,
	rawStream: MediaStream,
	level: NoiseSuppression,
	gainNode?: GainNode
): Promise<MediaStream> {
	if (level === 'off') return rawStream;

	await ensureWorkletModules(ctx);

	const source = ctx.createMediaStreamSource(rawStream);
	const workletNode = await createWorkletNode(ctx, level);
	const destination = ctx.createMediaStreamDestination();

	source.connect(workletNode);
	if (gainNode) {
		workletNode.connect(gainNode);
		gainNode.connect(destination);
	} else {
		workletNode.connect(destination);
	}

	currentState = { source, workletNode, destination, level };

	// Return a stream that has the processed audio + any video tracks from original
	const processedStream = destination.stream;
	for (const vt of rawStream.getVideoTracks()) {
		processedStream.addTrack(vt);
	}
	return processedStream;
}

/// Remove noise suppression pipeline and clean up nodes.
export function removeNoiseSuppression(): void {
	if (!currentState) return;
	currentState.source.disconnect();
	currentState.workletNode.disconnect();
	if ('destroy' in currentState.workletNode) {
		(currentState.workletNode as RnnoiseWorkletNode | SpeexWorkletNode).destroy();
	}
	currentState = null;
}

/// Change suppression level mid-call. Returns the new processed audio track,
/// or null if switching to 'off' (caller should use the raw track).
export async function changeSuppressionLevel(
	ctx: AudioContext,
	rawStream: MediaStream,
	newLevel: NoiseSuppression,
	gainNode?: GainNode
): Promise<MediaStreamTrack | null> {
	// Tear down existing pipeline
	removeNoiseSuppression();

	if (newLevel === 'off') return null;

	const processed = await applyNoiseSuppression(ctx, rawStream, newLevel, gainNode);
	return processed.getAudioTracks()[0] ?? null;
}

/// Get the current suppression level, or 'off' if not active.
export function getCurrentLevel(): NoiseSuppression {
	return currentState?.level ?? 'off';
}

/// Human-readable label for each level.
export const SUPPRESSION_LABELS: Record<NoiseSuppression, string> = {
	off: 'Off',
	'noise-gate': 'Noise Gate',
	standard: 'Standard',
	maximum: 'Maximum'
};

/// Cycle to the next level.
export function nextLevel(current: NoiseSuppression): NoiseSuppression {
	const order: NoiseSuppression[] = ['off', 'noise-gate', 'standard', 'maximum'];
	const idx = order.indexOf(current);
	return order[(idx + 1) % order.length];
}
