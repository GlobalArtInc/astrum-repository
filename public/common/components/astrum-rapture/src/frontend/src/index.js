
// Suppress noisy AS.app.Loader console chatter early so debug logs stay readable.
if (typeof window !== 'undefined' && !window.__asExtJsDebugPatchedRapture) {
  window.__asExtJsDebugPatchedRapture = true;
  const shouldSuppressLoader = (args) =>
    args.some((arg) => typeof arg === 'string' && arg.includes('AS.app.Loader'));

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

  // Also silence AS.Log (used by AS.app.Loader) once it appears.
  const trySilenceAsLog = () => {
    const asLog = window.AS && window.AS.Log;
    if (!asLog || asLog.__silencedForLoader) {
      return false;
    }
    ['logInfo', 'info', 'logDebug', 'debug', 'logWarn', 'warn', 'logError', 'error'].forEach((fn) => {
      if (typeof asLog[fn] === 'function') {
        asLog[fn] = () => {};
      }
    });
    asLog.__silencedForLoader = true;
    return true;
  };

  const asLogPoll = setInterval(() => {
    if (trySilenceAsLog()) {
      clearInterval(asLogPoll);
    }
  }, 50);

  // Stop polling after 10s to avoid runaway timers.
  setTimeout(() => clearInterval(asLogPoll), 10000);
}

console.info('[verify] rapture bundle from ASTRUM_RESOURCE_DIRS loaded');

import exposeDependencies from './exposeDependencies';
import configureDebug from './configureDebug.js';

exposeDependencies();
configureDebug();
