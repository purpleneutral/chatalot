import type { UserPublic } from '$lib/api/users';

class UserStore {
	private users = $state<Map<string, UserPublic>>(new Map());

	getUser(id: string): UserPublic | undefined {
		return this.users.get(id);
	}

	getDisplayName(id: string): string {
		return this.users.get(id)?.display_name ?? id.slice(0, 8);
	}

	getInitial(id: string): string {
		const name = this.users.get(id)?.display_name;
		return name ? name[0].toUpperCase() : id.slice(0, 2).toUpperCase();
	}

	setUser(user: UserPublic) {
		const next = new Map(this.users);
		next.set(user.id, user);
		this.users = next;
	}

	getAllUsers(): UserPublic[] {
		return Array.from(this.users.values());
	}

	clear() {
		this.users = new Map();
	}

	setUsers(users: UserPublic[]) {
		const next = new Map(this.users);
		for (const user of users) {
			next.set(user.id, user);
		}
		this.users = next;
	}
}

export const userStore = new UserStore();
