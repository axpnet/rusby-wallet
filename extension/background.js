// Rusby Wallet — Background Service Worker
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Pure JS (no WASM) — handles message routing, session state, pending requests.
// Private keys NEVER pass through this context.

// --- WalletConnect v2 ---
import { rusbyWC } from './wc-bundle.js';

// --- State (in-memory, reconstructed from chrome.storage on wake) ---
let walletLocked = true;
let pendingRequests = new Map(); // requestId -> { id, method, params, origin, tabId }
let approvedOrigins = {};        // origin -> [address]
let activeChainId = '0x1';       // Ethereum mainnet default
let activeAccounts = [];         // Current unlocked accounts
let wcInitialized = false;
let wcPendingProposals = new Map(); // proposalId -> proposal

// --- Initialization ---
chrome.runtime.onInstalled.addListener(() => {
  restoreState();
});

// Service worker can be killed and restarted — always restore state
self.addEventListener('activate', () => {
  restoreState();
});

async function restoreState() {
  try {
    const data = await chrome.storage.local.get([
      'approvedOrigins', 'activeChainId', 'walletLocked',
      'pendingRequests', 'activeAccounts'
    ]);
    approvedOrigins = data.approvedOrigins || {};
    activeChainId = data.activeChainId || '0x1';
    walletLocked = data.walletLocked !== false;
    activeAccounts = data.activeAccounts || [];
    if (data.pendingRequests) {
      pendingRequests = new Map(Object.entries(data.pendingRequests));
    }
  } catch (e) {
    console.error('[Rusby BG] restoreState error:', e);
  }
}

async function persistState() {
  try {
    await chrome.storage.local.set({
      approvedOrigins,
      activeChainId,
      walletLocked,
      activeAccounts,
      pendingRequests: Object.fromEntries(pendingRequests),
    });
  } catch (e) {
    console.error('[Rusby BG] persistState error:', e);
  }
}

// --- WalletConnect Initialization ---
async function initWalletConnect() {
  try {
    const data = await chrome.storage.local.get('wc_project_id');
    const projectId = data.wc_project_id;
    if (!projectId) {
      console.log('[Rusby BG] No WC project ID configured');
      return;
    }
    await rusbyWC.init(projectId);
    wcInitialized = true;
    console.log('[Rusby BG] WalletConnect initialized');

    rusbyWC.on('session_proposal', (proposal) => {
      const id = proposal.id;
      wcPendingProposals.set(id, proposal);
      // Open popup for proposal approval
      try {
        chrome.windows.create({
          url: chrome.runtime.getURL(`index.html?wc_proposal=${id}`),
          type: 'popup',
          width: 400,
          height: 620,
          focused: true,
        });
      } catch (e) {
        console.error('[Rusby BG] Failed to open WC proposal popup:', e);
      }
    });

    rusbyWC.on('session_request', (event) => {
      const { topic, params, id } = event;
      const { request, chainId } = params;
      const requestId = crypto.randomUUID();
      pendingRequests.set(requestId, {
        id,
        method: request.method,
        params: request.params,
        origin: `walletconnect:${topic}`,
        tabId: null,
        requestId,
        wcTopic: topic,
        wcRequestId: id,
        wcChainId: chainId,
        source: 'walletconnect',
        timestamp: Date.now(),
      });
      persistState();
      try {
        chrome.windows.create({
          url: chrome.runtime.getURL(`index.html?approve=${requestId}`),
          type: 'popup',
          width: 400,
          height: 620,
          focused: true,
        });
      } catch (e) {
        console.error('[Rusby BG] Failed to open WC request popup:', e);
      }
    });

    rusbyWC.on('session_delete', (event) => {
      console.log('[Rusby BG] WC session deleted:', event.topic);
    });
  } catch (e) {
    console.error('[Rusby BG] WalletConnect init failed:', e);
  }
}

// Keep-alive alarm for WC WebSocket (SW gets killed after ~30s)
chrome.alarms.create('wc-keepalive', { periodInMinutes: 0.4 });
chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name === 'wc-keepalive' && wcInitialized) {
    // Ping to keep WebSocket alive
    try { rusbyWC.getActiveSessions(); } catch (_) {}
  }
});

// Init WC on startup
restoreState().then(() => initWalletConnect());

// --- Message Handling ---
// We use long-lived connections (ports) for content scripts so responses
// can be sent asynchronously after popup approval.

const contentPorts = new Map(); // tabId -> port

chrome.runtime.onConnect.addListener((port) => {
  if (port.name !== 'rusby-content') return;

  const tabId = port.sender?.tab?.id;
  if (tabId != null) {
    contentPorts.set(tabId, port);
    port.onDisconnect.addListener(() => contentPorts.delete(tabId));
  }

  port.onMessage.addListener(async (msg) => {
    const origin = port.sender?.tab?.url
      ? new URL(port.sender.tab.url).origin
      : '';
    const response = await handleDappMessage(msg, origin, tabId);
    if (response) {
      port.postMessage({ id: msg.id, ...response });
    }
  });
});

// Simple messages from popup (approve/reject/lock state)
// SECURITY: Only accept messages from our own extension
chrome.runtime.onMessage.addListener((msg, sender, sendResponse) => {
  if (msg.target !== 'rusby-background') return false;
  // Validate sender is our own extension (prevent cross-extension attacks)
  if (sender.id !== chrome.runtime.id) {
    console.warn('[Rusby BG] Rejected message from foreign extension:', sender.id);
    return false;
  }
  handlePopupMessage(msg).then(sendResponse);
  return true; // async
});

// --- dApp Request Handler ---
async function handleDappMessage(msg, origin, tabId) {
  const { id, method, params } = msg;

  switch (method) {
    case 'eth_requestAccounts':
      return handleRequestAccounts(id, origin, tabId);

    case 'eth_accounts':
      if (walletLocked || !approvedOrigins[origin]) {
        return { result: [] };
      }
      return { result: approvedOrigins[origin] };

    case 'eth_chainId':
      return { result: activeChainId };

    case 'net_version':
      return { result: String(parseInt(activeChainId, 16)) };

    case 'eth_sendTransaction':
    case 'personal_sign':
    case 'eth_signTypedData_v4':
      if (walletLocked) {
        return { error: { code: 4100, message: 'Wallet locked' } };
      }
      if (!approvedOrigins[origin]) {
        return { error: { code: 4100, message: 'Not connected' } };
      }
      return enqueueForApproval(id, method, params, origin, tabId);

    case 'wallet_switchEthereumChain':
      if (!approvedOrigins[origin]) {
        return { error: { code: 4100, message: 'Not connected' } };
      }
      return handleSwitchChain(id, params);

    case 'wallet_addEthereumChain':
      // For now, reject — custom chains not supported yet
      return { error: { code: 4200, message: 'Custom chains not supported' } };

    default:
      return { error: { code: 4200, message: `Method not supported: ${method}` } };
  }
}

async function handleRequestAccounts(id, origin, tabId) {
  if (walletLocked) {
    return enqueueForApproval(id, 'eth_requestAccounts', [], origin, tabId);
  }
  if (approvedOrigins[origin]) {
    return { result: approvedOrigins[origin] };
  }
  return enqueueForApproval(id, 'eth_requestAccounts', [], origin, tabId);
}

async function enqueueForApproval(id, method, params, origin, tabId) {
  const requestId = crypto.randomUUID();
  pendingRequests.set(requestId, {
    id, method, params, origin, tabId, requestId,
    timestamp: Date.now(),
  });
  await persistState();

  // Try to open popup for approval
  try {
    await chrome.windows.create({
      url: chrome.runtime.getURL(`index.html?approve=${requestId}`),
      type: 'popup',
      width: 400,
      height: 620,
      focused: true,
    });
  } catch (e) {
    console.error('[Rusby BG] Failed to open popup:', e);
  }

  // Return null — response will be sent via port when popup approves/rejects
  return null;
}

function handleSwitchChain(id, params) {
  if (!params || !params[0] || !params[0].chainId) {
    return { error: { code: -32602, message: 'Invalid params' } };
  }
  const newChainId = params[0].chainId;
  // Validate known chain IDs
  const knownChains = ['0x1', '0x89', '0x38', '0xa', '0x2105', '0xa4b1'];
  if (!knownChains.includes(newChainId)) {
    return { error: { code: 4902, message: 'Unrecognized chain ID' } };
  }
  activeChainId = newChainId;
  persistState();
  // Notify all content scripts of chain change
  broadcastEvent('chainChanged', newChainId);
  return { result: null };
}

// --- Popup Message Handler ---
async function handlePopupMessage(msg) {
  switch (msg.method) {
    case '__rusby_approve': {
      const { requestId, result } = msg;
      const req = pendingRequests.get(requestId);
      if (!req) return { error: 'Request not found' };

      // If eth_requestAccounts, store permission
      if (req.method === 'eth_requestAccounts' && result) {
        approvedOrigins[req.origin] = result;
      }

      if (req.source === 'walletconnect' && wcInitialized) {
        // WC request — respond via WC protocol
        try {
          await rusbyWC.respondRequest(req.wcTopic, req.wcRequestId, result);
        } catch (e) {
          console.error('[Rusby BG] WC respond error:', e);
        }
      } else {
        // dApp request — respond via content script port
        const port = contentPorts.get(req.tabId);
        if (port) {
          port.postMessage({ id: req.id, result });
        }
      }

      pendingRequests.delete(requestId);
      await persistState();
      return { ok: true };
    }

    case '__rusby_reject': {
      const { requestId } = msg;
      const req = pendingRequests.get(requestId);
      if (!req) return { error: 'Request not found' };

      if (req.source === 'walletconnect' && wcInitialized) {
        try {
          await rusbyWC.rejectRequest(req.wcTopic, req.wcRequestId, 'User rejected');
        } catch (e) {
          console.error('[Rusby BG] WC reject error:', e);
        }
      } else {
        const port = contentPorts.get(req.tabId);
        if (port) {
          port.postMessage({
            id: req.id,
            error: { code: 4001, message: 'User rejected request' },
          });
        }
      }

      pendingRequests.delete(requestId);
      await persistState();
      return { ok: true };
    }

    case '__rusby_lock_state': {
      walletLocked = msg.locked;
      if (walletLocked) {
        activeAccounts = [];
      }
      await persistState();
      if (walletLocked) {
        broadcastEvent('accountsChanged', []);
      }
      return { ok: true };
    }

    case '__rusby_set_accounts': {
      activeAccounts = msg.accounts || [];
      await persistState();
      return { ok: true };
    }

    case '__rusby_get_pending': {
      return { requests: Array.from(pendingRequests.values()) };
    }

    case '__rusby_revoke_origin': {
      const { origin } = msg;
      delete approvedOrigins[origin];
      await persistState();
      // Notify that origin's tabs
      broadcastEvent('accountsChanged', []);
      return { ok: true };
    }

    case '__rusby_get_approved_origins': {
      return { origins: approvedOrigins };
    }

    // --- WalletConnect methods ---
    case '__rusby_wc_pair': {
      if (!wcInitialized) return { error: 'WalletConnect not initialized' };
      try {
        await rusbyWC.pair(msg.uri);
        return { ok: true };
      } catch (e) {
        return { error: e.message };
      }
    }

    case '__rusby_wc_get_proposals': {
      return { proposals: Array.from(wcPendingProposals.entries()).map(([id, p]) => ({ id, ...p })) };
    }

    case '__rusby_wc_approve_proposal': {
      if (!wcInitialized) return { error: 'WalletConnect not initialized' };
      try {
        const session = await rusbyWC.approveSession(msg.proposalId, msg.namespaces);
        wcPendingProposals.delete(msg.proposalId);
        return { ok: true, session };
      } catch (e) {
        return { error: e.message };
      }
    }

    case '__rusby_wc_reject_proposal': {
      if (!wcInitialized) return { error: 'WalletConnect not initialized' };
      try {
        await rusbyWC.rejectSession(msg.proposalId);
        wcPendingProposals.delete(msg.proposalId);
        return { ok: true };
      } catch (e) {
        return { error: e.message };
      }
    }

    case '__rusby_wc_approve_request': {
      if (!wcInitialized) return { error: 'WalletConnect not initialized' };
      try {
        await rusbyWC.respondRequest(msg.topic, msg.wcRequestId, msg.result);
        return { ok: true };
      } catch (e) {
        return { error: e.message };
      }
    }

    case '__rusby_wc_reject_request': {
      if (!wcInitialized) return { error: 'WalletConnect not initialized' };
      try {
        await rusbyWC.rejectRequest(msg.topic, msg.wcRequestId, msg.message || 'User rejected');
        return { ok: true };
      } catch (e) {
        return { error: e.message };
      }
    }

    case '__rusby_wc_disconnect': {
      if (!wcInitialized) return { error: 'WalletConnect not initialized' };
      try {
        await rusbyWC.disconnectSession(msg.topic);
        return { ok: true };
      } catch (e) {
        return { error: e.message };
      }
    }

    case '__rusby_wc_get_sessions': {
      if (!wcInitialized) return { sessions: {} };
      return { sessions: rusbyWC.getActiveSessions() };
    }

    case '__rusby_wc_set_project_id': {
      await chrome.storage.local.set({ wc_project_id: msg.projectId });
      if (!wcInitialized) {
        await initWalletConnect();
      }
      return { ok: true };
    }

    default:
      return { error: 'Unknown method' };
  }
}

// --- Broadcast Events to Content Scripts ---
function broadcastEvent(event, data) {
  for (const [, port] of contentPorts) {
    try {
      port.postMessage({ type: 'event', event, data });
    } catch (_) {
      // Port may be disconnected
    }
  }
}
