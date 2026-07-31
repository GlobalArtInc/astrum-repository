

const SUPPRESS_PATTERNS = [
  'NX.app.Loader',
  'NX.app.Application',
  'evaluating transition',
  'Redirecting to login',
  'managed controllers',
  'Configured error handling',
  'Configured Ext.Direct',
  'Configured state provider',
  'responsive plugin is deprecated',
  '[verify]',
];

/**
 * Patches console methods to suppress noisy ExtJS internal chatter during boot.
 * Safe to call multiple times — guarded by a window flag.
 */
export default function configureDebugLogging() {
  if (typeof window === 'undefined' || window.__nxExtJsDebugPatchedGlobal) {
    return;
  }
  window.__nxExtJsDebugPatchedGlobal = true;

  const shouldSuppress = (args) =>
    args.some((arg) => typeof arg === 'string' &&
      SUPPRESS_PATTERNS.some(pattern => arg.includes(pattern)));

  const wrap = (originalFn) => (...args) => {
    if (shouldSuppress(args)) {
      return;
    }
    originalFn && originalFn(...args);
  };

  console.debug = wrap(console.debug?.bind(console));
  console.info = wrap(console.info?.bind(console));
  console.log = wrap(console.log?.bind(console));
  console.warn = wrap(console.warn?.bind(console));
}
