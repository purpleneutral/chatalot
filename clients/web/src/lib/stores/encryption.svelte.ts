/** Tracks unacknowledged identity key changes (TOFU violations). */
class EncryptionStore {
	private keyChangedUsers = $state<Set<string>>(new Set());

	hasKeyChanged(userId: string): boolean {
		return this.keyChangedUsers.has(userId);
	}

	get hasAnyKeyChange(): boolean {
		return this.keyChangedUsers.size > 0;
	}

	addKeyChange(userId: string) {
		const next = new Set(this.keyChangedUsers);
		next.add(userId);
		this.keyChangedUsers = next;
	}

	acknowledgeKeyChange(userId: string) {
		const next = new Set(this.keyChangedUsers);
		next.delete(userId);
		this.keyChangedUsers = next;
	}

	clear() {
		this.keyChangedUsers = new Set();
	}
}

export const encryptionStore = new EncryptionStore();
