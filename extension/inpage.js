// Rusby Wallet — Injected Provider (EIP-1193 + EIP-6963)
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Injected into the page context by content-script.js.
// Exposes window.rusby as an EIP-1193 compatible provider.

'use strict';

(function () {
  if (window.rusby) return; // Already injected

  class RusbyProvider {
    constructor() {
      this._events = {};
      this._requestId = 0;
      this._pending = new Map(); // id -> { resolve, reject }
      this.isRusby = true;
      this.isMetaMask = false;
      this.chainId = null;
      this.selectedAddress = null;
      this._connected = false;

      // Listen for responses from content script
      window.addEventListener('message', (event) => {
        if (event.source !== window) return;
        if (event.data?.target !== 'rusby-inpage') return;

        if (event.data.type === 'event') {
          this._handleEvent(event.data.event, event.data.data);
        } else if (event.data.type === 'response') {
          this._handleResponse(event.data);
        }
      });
    }

    // --- EIP-1193 request ---
    async request({ method, params }) {
      const id = ++this._requestId;
      return new Promise((resolve, reject) => {
        this._pending.set(id, { resolve, reject, method });

        window.postMessage({
          target: 'rusby-contentscript',
          payload: { id, method, params },
        }, window.location.origin);

        // Timeout after 5 minutes (user may take time to approve)
        setTimeout(() => {
          if (this._pending.has(id)) {
            this._pending.delete(id);
            reject(new Error('Request timeout'));
          }
        }, 300000);
      });
    }

    // --- Legacy methods (backwards compatibility) ---
    async enable() {
      return this.request({ method: 'eth_requestAccounts' });
    }

    async send(method, params) {
      return this.request({ method, params });
    }

    sendAsync(payload, callback) {
      this.request({ method: payload.method, params: payload.params })
        .then(result => callback(null, { id: payload.id, jsonrpc: '2.0', result }))
        .catch(err => callback(err));
    }

    // --- Events (EIP-1193) ---
    on(event, callback) {
      if (!this._events[event]) {
        this._events[event] = [];
      }
      this._events[event].push(callback);
      return this;
    }

    removeListener(event, callback) {
      if (this._events[event]) {
        this._events[event] = this._events[event].filter(cb => cb !== callback);
      }
      return this;
    }

    removeAllListeners(event) {
      if (event) {
        delete this._events[event];
      } else {
        this._events = {};
      }
      return this;
    }

    // --- Internal ---
    _emit(event, data) {
      const listeners = this._events[event];
      if (listeners) {
        listeners.forEach(cb => {
          try { cb(data); } catch (_) {}
        });
      }
    }

    _handleEvent(event, data) {
      switch (event) {
        case 'chainChanged':
          this.chainId = data;
          this._emit('chainChanged', data);
          break;
        case 'accountsChanged':
          this.selectedAddress = data?.[0] || null;
          this._emit('accountsChanged', data);
          if (data && data.length > 0 && !this._connected) {
            this._connected = true;
            this._emit('connect', { chainId: this.chainId });
          } else if (!data || data.length === 0) {
            this._connected = false;
            this._emit('disconnect', { code: 4900, message: 'Disconnected' });
          }
          break;
      }
    }

    _handleResponse(msg) {
      const pending = this._pending.get(msg.id);
      if (!pending) return;
      this._pending.delete(msg.id);

      if (msg.error) {
        const err = new Error(msg.error.message || 'Unknown error');
        err.code = msg.error.code || -32603;
        pending.reject(err);
      } else {
        // Update local state on successful connects
        if (pending.method === 'eth_requestAccounts' && msg.result) {
          this.selectedAddress = msg.result[0] || null;
          this._connected = true;
          this._emit('connect', { chainId: this.chainId });
          this._emit('accountsChanged', msg.result);
        } else if (pending.method === 'eth_chainId' && msg.result) {
          this.chainId = msg.result;
        }
        pending.resolve(msg.result);
      }
    }
  }

  // --- Create and expose provider ---
  const provider = new RusbyProvider();

  // Expose as window.rusby (not window.ethereum to avoid conflicts)
  Object.defineProperty(window, 'rusby', {
    value: provider,
    writable: false,
    configurable: false,
  });

  // --- EIP-6963: Multi-provider discovery ---
  const providerInfo = Object.freeze({
    uuid: 'f8c3de3d-1fea-4d7c-a8b0-29f63c4c3454', // Fixed UUID for Rusby
    name: 'Rusby Wallet',
    icon: 'data:image/svg+xml;base64,' + btoa(
      '<svg xmlns="http://www.w3.org/2000/svg" width="128" height="128" viewBox="0 0 128 128">' +
      '<rect width="128" height="128" rx="25" fill="#6c5ce7"/>' +
      '<text x="50%" y="55%" text-anchor="middle" dominant-baseline="middle" ' +
      'fill="white" font-family="sans-serif" font-weight="bold" font-size="52">W</text>' +
      '</svg>'
    ),
    rdns: 'io.rusby.wallet',
  });

  const announceProvider = () => {
    window.dispatchEvent(
      new CustomEvent('eip6963:announceProvider', {
        detail: Object.freeze({ info: providerInfo, provider }),
      })
    );
  };

  // Announce on load
  announceProvider();

  // Re-announce when requested
  window.addEventListener('eip6963:requestProvider', announceProvider);
})();
