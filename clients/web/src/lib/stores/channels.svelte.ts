import type { Channel } from '$lib/api/channels';
import { memberStore } from './members.svelte';
import { messageStore } from './messages.svelte';
import { presenceStore } from './presence.svelte';

class ChannelStore {
	channels = $state<Channel[]>([]);
	activeChannelId = $state<string | null>(null);

	get activeChannel(): Channel | undefined {
		return this.channels.find(c => c.id === this.activeChannelId);
	}

	setChannels(channels: Channel[]) {
		this.channels = channels;
	}

	addChannel(channel: Channel) {
		if (!this.channels.find(c => c.id === channel.id)) {
			this.channels = [...this.channels, channel];
		}
	}

	updateChannel(channel: Channel) {
		this.channels = this.channels.map(c => c.id === channel.id ? channel : c);
	}

	removeChannel(channelId: string) {
		this.channels = this.channels.filter(c => c.id !== channelId);
		messageStore.clearChannel(channelId);
		memberStore.clearChannel(channelId);
		presenceStore.clearChannel(channelId);
		if (this.activeChannelId === channelId) {
			this.activeChannelId = null;
		}
	}

	removeChannelsForGroup(groupId: string) {
		const removed = this.channels.filter(c => c.group_id === groupId);
		this.channels = this.channels.filter(c => c.group_id !== groupId);
		for (const ch of removed) {
			messageStore.clearChannel(ch.id);
			memberStore.clearChannel(ch.id);
			presenceStore.clearChannel(ch.id);
		}
		if (removed.some(c => c.id === this.activeChannelId)) {
			this.activeChannelId = null;
		}
	}

	setActive(channelId: string | null) {
		this.activeChannelId = channelId;
	}

	clear() {
		this.channels = [];
		this.activeChannelId = null;
	}
}

export const channelStore = new ChannelStore();
