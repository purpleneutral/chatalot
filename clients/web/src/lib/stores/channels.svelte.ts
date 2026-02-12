import type { Channel } from '$lib/api/channels';

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

	removeChannel(channelId: string) {
		this.channels = this.channels.filter(c => c.id !== channelId);
		if (this.activeChannelId === channelId) {
			this.activeChannelId = null;
		}
	}

	setActive(channelId: string | null) {
		this.activeChannelId = channelId;
	}
}

export const channelStore = new ChannelStore();
