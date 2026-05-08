
// Suppress noisy NX.app.Loader console chatter early so debug logs stay readable.
if (typeof window !== 'undefined' && !window.__nxExtJsDebugPatchedRapture) {
  window.__nxExtJsDebugPatchedRapture = true;
  const shouldSuppressLoader = (args) =>
    args.some((arg) => typeof arg === 'string' && arg.includes('NX.app.Loader'));

  const wrap = (originalFn) => (...args) => {
    if (shouldSuppressLoader(args)) {
      return;
    }
    originalFn && originalFn(...args);
  };

  console.debug = wrap(console.debug?.bind(console));
  console.info = wrap(console.info?.bind(console));
  console.log = wrap(console.log?.bind(console));
  console.warn = wrap(console.warn?.bind(console));
  console.error = wrap(console.error?.bind(console));

  // Also silence NX.Log (used by NX.app.Loader) once it appears.
  const trySilenceNxLog = () => {
    const nxLog = window.NX && window.NX.Log;
    if (!nxLog || nxLog.__silencedForLoader) {
      return false;
    }
    ['logInfo', 'info', 'logDebug', 'debug', 'logWarn', 'warn', 'logError', 'error'].forEach((fn) => {
      if (typeof nxLog[fn] === 'function') {
        nxLog[fn] = () => {};
      }
    });
    nxLog.__silencedForLoader = true;
    return true;
  };

  const nxLogPoll = setInterval(() => {
    if (trySilenceNxLog()) {
      clearInterval(nxLogPoll);
    }
  }, 50);

  // Stop polling after 10s to avoid runaway timers.
  setTimeout(() => clearInterval(nxLogPoll), 10000);
}

console.info('[verify] rapture bundle from ASTRUM_RESOURCE_DIRS loaded');

import exposeDependencies from './exposeDependencies';
import configureDebug from './configureDebug.js';

exposeDependencies();
configureDebug();
