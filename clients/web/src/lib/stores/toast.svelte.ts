export interface Toast {
	id: number;
	message: string;
	type: 'success' | 'error' | 'info';
}

let nextId = 0;
const timers = new Map<number, ReturnType<typeof setTimeout>>();

class ToastStore {
	toasts = $state<Toast[]>([]);

	show(message: string, type: Toast['type'] = 'info', duration = 4000) {
		// Deduplicate: skip if same message+type is already visible
		if (this.toasts.some(t => t.message === message && t.type === type)) return;
		const id = nextId++;
		this.toasts = [...this.toasts, { id, message, type }];
		timers.set(id, setTimeout(() => this.dismiss(id), duration));
	}

	success(message: string) {
		this.show(message, 'success');
	}

	error(message: string) {
		this.show(message, 'error', 6000);
	}

	info(message: string) {
		this.show(message, 'info');
	}

	dismiss(id: number) {
		const timer = timers.get(id);
		if (timer) {
			clearTimeout(timer);
			timers.delete(id);
		}
		this.toasts = this.toasts.filter((t) => t.id !== id);
	}
}

export const toastStore = new ToastStore();
