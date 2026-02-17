import type { Group } from '$lib/api/groups';

class GroupStore {
	groups = $state<Group[]>([]);
	activeGroupId = $state<string | null>(null);

	get activeGroup(): Group | undefined {
		return this.groups.find(g => g.id === this.activeGroupId);
	}

	setGroups(groups: Group[]) {
		this.groups = groups;
	}

	addGroup(group: Group) {
		if (!this.groups.find(g => g.id === group.id)) {
			this.groups = [...this.groups, group];
		}
	}

	updateGroup(id: string, updates: Partial<Group>) {
		this.groups = this.groups.map(g =>
			g.id === id ? { ...g, ...updates } : g
		);
	}

	removeGroup(id: string) {
		this.groups = this.groups.filter(g => g.id !== id);
		if (this.activeGroupId === id) {
			this.activeGroupId = null;
		}
	}

	setActive(groupId: string | null) {
		this.activeGroupId = groupId;
	}

	clear() {
		this.groups = [];
		this.activeGroupId = null;
	}
}

export const groupStore = new GroupStore();
