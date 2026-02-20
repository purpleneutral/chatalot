import { isTauri } from '$lib/env';

/**
 * Check for app updates in the Tauri desktop client.
 * No-op in the web browser. Shows a notification if an update is available.
 */
export async function checkForDesktopUpdate(): Promise<void> {
	if (!isTauri()) return;

	try {
		const { check } = await import('@tauri-apps/plugin-updater');
		const { relaunch } = await import('@tauri-apps/plugin-process');

		const update = await check();
		if (!update) return;

		console.log(`[updater] Update available: ${update.version}`);

		// Store update info so the UI can show a banner
		window.dispatchEvent(
			new CustomEvent('chatalot:desktop-update-available', {
				detail: {
					version: update.version,
					body: update.body,
					install: async () => {
						await update.downloadAndInstall();
						await relaunch();
					}
				}
			})
		);
	} catch (err) {
		// Updater failures should never block the app
		console.warn('[updater] Update check failed:', err);
	}
}
