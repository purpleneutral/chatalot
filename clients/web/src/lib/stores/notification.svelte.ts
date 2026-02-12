export type NotificationLevel = 'all' | 'mentions' | 'nothing';

interface NotificationPreferences {
	desktopEnabled: boolean;
	channelLevels: Record<string, NotificationLevel>;
	defaultChannelLevel: NotificationLevel;
}

const DEFAULTS: NotificationPreferences = {
	desktopEnabled: false,
	channelLevels: {},
	defaultChannelLevel: 'all'
};

class NotificationStore {
	preferences = $state<NotificationPreferences>({ ...DEFAULTS });
	private _pageHidden = $state(false);

	get pageHidden(): boolean {
		return this._pageHidden;
	}

	get permissionState(): NotificationPermission | 'unsupported' {
		if (typeof Notification === 'undefined') return 'unsupported';
		return Notification.permission;
	}

	constructor() {
		if (typeof localStorage !== 'undefined') {
			const saved = localStorage.getItem('notification_preferences');
			if (saved) {
				try {
					this.preferences = { ...DEFAULTS, ...JSON.parse(saved) };
				} catch { /* ignore */ }
			}
		}

		if (typeof document !== 'undefined') {
			document.addEventListener('visibilitychange', () => {
				this._pageHidden = document.hidden;
			});
			this._pageHidden = document.hidden;
		}
	}

	save() {
		localStorage.setItem('notification_preferences', JSON.stringify(this.preferences));
	}

	async requestPermission(): Promise<boolean> {
		if (typeof Notification === 'undefined') return false;
		const result = await Notification.requestPermission();
		if (result === 'granted') {
			this.preferences = { ...this.preferences, desktopEnabled: true };
			this.save();
		}
		return result === 'granted';
	}

	getChannelLevel(channelId: string): NotificationLevel {
		return this.preferences.channelLevels[channelId] ?? this.preferences.defaultChannelLevel;
	}

	setChannelLevel(channelId: string, level: NotificationLevel) {
		if (level === this.preferences.defaultChannelLevel) {
			const { [channelId]: _, ...rest } = this.preferences.channelLevels;
			this.preferences = { ...this.preferences, channelLevels: rest };
		} else {
			this.preferences = {
				...this.preferences,
				channelLevels: { ...this.preferences.channelLevels, [channelId]: level }
			};
		}
		this.save();
	}

	showDesktopNotification(opts: {
		title: string;
		body: string;
		channelId: string;
	}) {
		if (!this.preferences.desktopEnabled) return;
		if (typeof Notification === 'undefined') return;
		if (Notification.permission !== 'granted') return;

		const notification = new Notification(opts.title, {
			body: opts.body,
			tag: opts.channelId,
			silent: true
		});

		notification.onclick = () => {
			window.focus();
			window.dispatchEvent(
				new CustomEvent('chatalot:navigate-channel', { detail: opts.channelId })
			);
			notification.close();
		};
	}
}

export const notificationStore = new NotificationStore();
