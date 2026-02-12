export interface Toast {
	id: number;
	message: string;
	type: 'success' | 'error' | 'info';
}

let nextId = 0;

class ToastStore {
	toasts = $state<Toast[]>([]);

	show(message: string, type: Toast['type'] = 'info', duration = 4000) {
		const id = nextId++;
		this.toasts = [...this.toasts, { id, message, type }];
		setTimeout(() => this.dismiss(id), duration);
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
		this.toasts = this.toasts.filter((t) => t.id !== id);
	}
}

export const toastStore = new ToastStore();
