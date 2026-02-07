// Rusby Wallet — Content Script (bridge)
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Runs in each web page context. Bridges inpage.js ↔ background.js

'use strict';

// 1. Inject inpage.js into the page context
const script = document.createElement('script');
script.src = chrome.runtime.getURL('inpage.js');
script.onload = () => script.remove();
(document.head || document.documentElement).appendChild(script);

// 2. Connect to background service worker via long-lived port
const port = chrome.runtime.connect({ name: 'rusby-content' });

// 3. Relay: page → background
window.addEventListener('message', (event) => {
  if (event.source !== window) return;
  if (event.data?.target !== 'rusby-contentscript') return;
  const { id, method, params } = event.data.payload;
  port.postMessage({ id, method, params });
});

// 4. Relay: background → page
port.onMessage.addListener((msg) => {
  if (msg.type === 'event') {
    // Broadcast event to inpage provider
    window.postMessage({
      target: 'rusby-inpage',
      type: 'event',
      event: msg.event,
      data: msg.data,
    }, window.location.origin);
  } else {
    // Response to a request
    window.postMessage({
      target: 'rusby-inpage',
      type: 'response',
      id: msg.id,
      result: msg.result,
      error: msg.error,
    }, window.location.origin);
  }
});
