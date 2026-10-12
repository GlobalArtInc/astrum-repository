

// Console filtering setup - runs before all other setup files
// This ensures console calls from imported modules are properly filtered

// In CI, suppress all console output to reduce build log noise.
// Locally, apply pattern-based filtering to suppress known noisy messages.
// CI=true is set via frontend-maven-plugin environmentVariables in the root pom.xml.
const IS_CI = process.env.CI === 'true';

const original = {
  error: console.error,
  warn: console.warn,
  log: console.log,
  debug: console.debug
};

const IGNORED = {
  error: [
    /validateDOMNesting/,
  ],
  warn: [
    /predictableActionArguments/,
    /could not determine visibility/,
    /state is not visible for navigation/,
    /url not recognized/,
    /login unsuccessful/
  ],
  log: [
    /does not support upload through the web UI/
  ],
  debug: [
    /rechecking visiblity requirements/,
    /user still does not have permissions/,
    /bundleActive=false/,
    /licenseValid=false/,
    /statesEnabled=false/,
    /permissions=false/,
    /permissionPrefix=false/,
    /editions=false/,
    /capability=false/,
    /requiresUser=false/,
    /anonymousAccessOrHasUser=false/,
    /notClustered=/,
    /evaluating transition from/,
    /permission.*was not present/,
    /No permissions found with prefix:/,
    /Found permission with prefix/
  ]
};

function makeFiltered(level) {
  return (...args) => {
    if (IS_CI) return;

    // Convert all args to string and join with spaces, handling objects/arrays
    const text = args.map(arg => {
      if (typeof arg === 'object') {
        try {
          return JSON.stringify(arg);
        } catch {
          return String(arg);
        }
      }
      return String(arg);
    }).join(' ');

    const shouldIgnore = IGNORED[level].some((re) => re.test(text));
    if (!shouldIgnore) (original[level])(...args);
  };
}

const consoleSpies = {
  error: jest.spyOn(console, 'error').mockImplementation(makeFiltered('error')),
  warn: jest.spyOn(console, 'warn').mockImplementation(makeFiltered('warn')),
  log: jest.spyOn(console, 'log').mockImplementation(makeFiltered('log')),
  debug: jest.spyOn(console, 'debug').mockImplementation(makeFiltered('debug'))
};

afterAll(() => {
  consoleSpies.error.mockRestore?.();
  consoleSpies.warn.mockRestore?.();
  consoleSpies.log.mockRestore?.();
  consoleSpies.debug.mockRestore?.();
});
