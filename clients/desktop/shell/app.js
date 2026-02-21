(function() {
	var STORAGE_KEY = 'chatalot_server_url';
	var ALLOWED_ACTIONS = {
		'store_key': true, 'get_key': true, 'delete_key': true,
		'check-update': true, 'set-server-url': true, 'clear-server-url': true,
		'reload-app': true
	};

	var form = document.getElementById('connect-form');
	var input = document.getElementById('server-input');
	var btn = document.getElementById('connect-btn');
	var errorEl = document.getElementById('connect-error');
	var loading = document.getElementById('loading');
	var iframe = document.getElementById('app-frame');
	var serverUrl = localStorage.getItem(STORAGE_KEY);

	function loadApp(url) {
		form.style.display = 'none';
		loading.style.display = 'flex';
		iframe.src = url;
		iframe.onload = function() {
			loading.style.display = 'none';
			iframe.style.display = 'block';
		};
	}

	function handleConnect() {
		errorEl.textContent = '';
		var url = input.value.trim().replace(/\/+$/, '');
		if (!url) { errorEl.textContent = 'Please enter a server URL.'; return; }
		if (!/^https?:\/\//i.test(url)) url = 'https://' + url;
		btn.disabled = true;
		btn.textContent = 'Connecting...';

		fetch(url + '/api/auth/config', { mode: 'cors' })
			.then(function(resp) {
				if (!resp.ok) throw new Error('Server returned ' + resp.status);
				return resp.json();
			})
			.then(function(data) {
				if (!data.registration_mode) throw new Error('Not a Chatalot server');
				localStorage.setItem(STORAGE_KEY, url);
				serverUrl = url;
				loadApp(url);
			})
			.catch(function(err) {
				errorEl.textContent = 'Could not reach server: ' + (err.message || err);
				btn.disabled = false;
				btn.textContent = 'Connect';
			});
	}

	function sendResponse(id, result, error) {
		if (!iframe.contentWindow) return;
		iframe.contentWindow.postMessage({
			source: 'chatalot-shell',
			id: id,
			result: result,
			error: error
		}, '*');
	}

	if (serverUrl) {
		input.value = serverUrl;
		loadApp(serverUrl);
	}

	btn.addEventListener('click', handleConnect);
	input.addEventListener('keydown', function(e) {
		if (e.key === 'Enter') handleConnect();
	});

	window.addEventListener('message', function(event) {
		if (!serverUrl) return;
		try {
			var expected = new URL(serverUrl).origin;
			if (event.origin !== expected) return;
		} catch(e) { return; }

		var msg = event.data;
		if (!msg || typeof msg !== 'object' || msg.source !== 'chatalot-bridge') return;
		if (!ALLOWED_ACTIONS[msg.action]) {
			sendResponse(msg.id, null, 'Action not allowed: ' + msg.action);
			return;
		}

		try {
			var invoke = window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke;
			var result = null;

			switch (msg.action) {
				case 'store_key':
				case 'get_key':
				case 'delete_key': {
					if (!invoke) { sendResponse(msg.id, null, 'Tauri IPC not available'); return; }
					var cmd = msg.action;
					var payload = msg.action === 'get_key'
						? { keyName: msg.payload.keyName }
						: msg.action === 'delete_key'
						? { keyName: msg.payload.keyName }
						: { keyName: msg.payload.keyName, value: msg.payload.value };
					invoke(cmd, payload)
						.then(function(val) { sendResponse(msg.id, val === undefined ? true : val, null); })
						.catch(function(err) { sendResponse(msg.id, null, err.message || String(err)); });
					return;
				}
				case 'check-update': {
					result = { available: false };
					break;
				}
				case 'set-server-url': {
					if (msg.payload && msg.payload.url) {
						localStorage.setItem(STORAGE_KEY, msg.payload.url);
						serverUrl = msg.payload.url;
					}
					result = true;
					break;
				}
				case 'clear-server-url': {
					localStorage.removeItem(STORAGE_KEY);
					serverUrl = null;
					iframe.style.display = 'none';
					form.style.display = 'flex';
					result = true;
					break;
				}
				case 'reload-app': {
					iframe.src = serverUrl + '?_v=' + Date.now();
					result = true;
					break;
				}
			}

			sendResponse(msg.id, result, null);
		} catch(err) {
			sendResponse(msg.id, null, err.message || String(err));
		}
	});
})();
