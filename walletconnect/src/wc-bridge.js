// Rusby Wallet — WalletConnect v2 bridge
// Wraps @walletconnect/web3wallet for use in Chrome extension background SW

import { Web3Wallet } from '@walletconnect/web3wallet';
import { Core } from '@walletconnect/core';

const RELAY_URL = 'wss://relay.walletconnect.com';

// Chrome storage adapter for WC persistence (survives SW restarts)
const chromeStorageAdapter = {
  async getItem(key) {
    const result = await chrome.storage.local.get(`wc_${key}`);
    return result[`wc_${key}`] ?? null;
  },
  async setItem(key, value) {
    await chrome.storage.local.set({ [`wc_${key}`]: value });
  },
  async removeItem(key) {
    await chrome.storage.local.remove(`wc_${key}`);
  },
  async getKeys() {
    const all = await chrome.storage.local.get(null);
    return Object.keys(all)
      .filter(k => k.startsWith('wc_'))
      .map(k => k.slice(3));
  },
  async getEntries() {
    const all = await chrome.storage.local.get(null);
    return Object.entries(all)
      .filter(([k]) => k.startsWith('wc_'))
      .map(([k, v]) => [k.slice(3), v]);
  },
};

class RusbyWC {
  constructor() {
    this.wallet = null;
    this.listeners = {};
  }

  async init(projectId) {
    const core = new Core({
      projectId,
      relayUrl: RELAY_URL,
      storage: chromeStorageAdapter,
    });

    this.wallet = await Web3Wallet.init({
      core,
      metadata: {
        name: 'Rusby Wallet',
        description: 'Multi-chain wallet in pure Rust',
        url: 'https://rusby.io',
        icons: ['https://rusby.io/icon.png'],
      },
    });

    // Forward WC events
    this.wallet.on('session_proposal', (proposal) => {
      this._emit('session_proposal', proposal);
    });

    this.wallet.on('session_request', (request) => {
      this._emit('session_request', request);
    });

    this.wallet.on('session_delete', (event) => {
      this._emit('session_delete', event);
    });

    return true;
  }

  async pair(uri) {
    if (!this.wallet) throw new Error('WC not initialized');
    await this.wallet.pair({ uri });
  }

  async approveSession(id, namespaces) {
    if (!this.wallet) throw new Error('WC not initialized');
    return await this.wallet.approveSession({ id, namespaces });
  }

  async rejectSession(id) {
    if (!this.wallet) throw new Error('WC not initialized');
    await this.wallet.rejectSession({
      id,
      reason: { code: 4001, message: 'User rejected' },
    });
  }

  async respondRequest(topic, id, result) {
    if (!this.wallet) throw new Error('WC not initialized');
    await this.wallet.respondSessionRequest({
      topic,
      response: { id, jsonrpc: '2.0', result },
    });
  }

  async rejectRequest(topic, id, message = 'User rejected') {
    if (!this.wallet) throw new Error('WC not initialized');
    await this.wallet.respondSessionRequest({
      topic,
      response: {
        id,
        jsonrpc: '2.0',
        error: { code: 4001, message },
      },
    });
  }

  async disconnectSession(topic) {
    if (!this.wallet) throw new Error('WC not initialized');
    await this.wallet.disconnectSession({
      topic,
      reason: { code: 6000, message: 'User disconnected' },
    });
  }

  getActiveSessions() {
    if (!this.wallet) return {};
    return this.wallet.getActiveSessions();
  }

  getPendingProposals() {
    if (!this.wallet) return {};
    return this.wallet.getPendingSessionProposals();
  }

  // Event emitter
  on(event, cb) {
    if (!this.listeners[event]) this.listeners[event] = [];
    this.listeners[event].push(cb);
  }

  off(event, cb) {
    if (!this.listeners[event]) return;
    this.listeners[event] = this.listeners[event].filter(f => f !== cb);
  }

  _emit(event, data) {
    (this.listeners[event] || []).forEach(cb => {
      try { cb(data); } catch (e) { console.error('RusbyWC event error:', e); }
    });
  }
}

// Singleton export
export const rusbyWC = new RusbyWC();
export default rusbyWC;
