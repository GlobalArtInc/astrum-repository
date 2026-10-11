
import { getTestRouter } from './testRouter';
import { UIView } from '@uirouter/react';
import ExtJS from '../interface/ExtJS';

jest.mock('../interface/ExtJS');

describe('createRouter - onBefore - validate permissions and configuration on each route request', () => {
  let router;

  beforeEach(() => {
    jest.clearAllMocks();

    // Default: user has no permissions (NavigationUtils calls AS.Permissions.check directly)
    global.AS.Permissions.check.mockReturnValue(false);

    ExtJS.hasUser = jest.fn().mockReturnValue(false);
    // Default to anonymous access disabled
    ExtJS.state = jest.fn().mockReturnValue({
      getValue: jest.fn().mockReturnValue(false)
    });
    // Default: the client already knows whether a user is signed in, so no deferral
    ExtJS.isAuthStateResolved = jest.fn().mockReturnValue(true);
    ExtJS.whenAuthStateResolved = jest.fn().mockResolvedValue(undefined);
    ExtJS.onStateChange = jest.fn(() => () => {});

    router = getTestRouter();
  });

  it('authenticated user goes to login page, should redirect to welcome page', async () => {
    ExtJS.hasUser.mockReturnValue(true);

    const goSpy = jest.spyOn(router.stateService, 'go');
    await router.stateService.go('login').catch(() => {});

    expect(goSpy).toHaveBeenCalledTimes(2);
    expect(goSpy).toHaveBeenNthCalledWith(1, 'login');
    expect(goSpy).toHaveBeenNthCalledWith(2, 'preview.browse.welcome');
  });

  it('go to visible pages, should be allowed', async () => {
    await router.stateService.go('preview.browse.welcome');
    expect(router.stateService.current.name).toBe('preview.browse.welcome');

    await router.stateService.go('login');
    expect(router.stateService.current.name).toBe('login');
  });

  it('authenticated user goes to an unauthorized page, should open the Astrum One dashboard', async () => {
    ExtJS.hasUser.mockReturnValue(true);
    // Navigate to a known state first to clear any residual state
    await router.stateService.go('preview.browse.welcome');

    const protectedRoute = {
      name: 'protected',
      url: '/protected',
      component: () => null,
      data: { visibilityRequirements: { permissions: ['admin:all'] } }
    };
    router.stateRegistry.register(protectedRoute);
    await router.urlService.sync();

    const goSpy = jest.spyOn(router.stateService, 'go');
    await router.stateService.go('protected').catch(() => {});

    expect(goSpy).toHaveBeenCalledTimes(2);
    expect(goSpy).toHaveBeenNthCalledWith(1, 'protected');
    expect(goSpy).toHaveBeenNthCalledWith(2, 'preview.browse.welcome');
  });

  it('from login user goes to unauthorized page, should stay on login (transition aborted)', async () => {
    const protectedRoute = {
      name: 'protected',
      url: '/protected',
      component: () => null,
      data: { visibilityRequirements: { permissions: ['admin:all'] } }
    };
    router.stateRegistry.register(protectedRoute);
    await router.urlService.sync();

    // First go to login
    await router.stateService.go('login');

    const goSpy = jest.spyOn(router.stateService, 'go');
    await router.stateService.go('protected').catch(() => {});

    // Transition is aborted; user stays on login — no additional go() calls
    expect(goSpy).toHaveBeenCalledTimes(1);
    expect(goSpy).toHaveBeenNthCalledWith(1, 'protected');
    expect(router.stateService.current.name).toBe('login');
  });

  it('unauthenticated user goes to unauthorized page, should redirect to login page with returnTo parameter', async () => {
    const protectedRoute = {
      name: 'protected',
      url: '/protected?filter&sort',
      component: () => null,
      data: { visibilityRequirements: { permissions: ['admin:all'] } }
    };
    router.stateRegistry.register(protectedRoute);
    await router.urlService.sync();

    const goSpy = jest.spyOn(router.stateService, 'go');
    const urlSpy = jest.spyOn(router.urlService, 'url').mockReturnValue('/protected?filter=maven&sort=name');
    await router.stateService.go('protected', { filter: 'maven', sort: 'name' }).catch(() => {});

    const expectedReturnTo = btoa('#/protected?filter=maven&sort=name');
    expect(goSpy).toHaveBeenCalledTimes(2);
    expect(goSpy).toHaveBeenNthCalledWith(1, 'protected', { filter: 'maven', sort: 'name' });
    expect(goSpy).toHaveBeenNthCalledWith(2, 'login', {returnTo: expectedReturnTo});
    expect(urlSpy).toHaveBeenCalled();
  });

  it('returnTo must decode to a plain hash URL without percent-encoding (OIDC server compat)', async () => {
    // OidcCallbackFilter.buildTargetUri only does Base64 decode on returnTo.
    // If the decoded value starts with %23 instead of #, the server treats it as a
    // literal path and the user gets a 404 after SSO login.
    const protectedRoute = {
      name: 'nodoubleencode',
      url: '/nodoubleencode',
      component: () => null,
      data: { visibilityRequirements: { permissions: ['admin:all'] } }
    };
    router.stateRegistry.register(protectedRoute);
    await router.urlService.sync();

    const goSpy = jest.spyOn(router.stateService, 'go');
    jest.spyOn(router.urlService, 'url').mockReturnValue('/nodoubleencode');
    await router.stateService.go('nodoubleencode').catch(() => {});

    const returnTo = goSpy.mock.calls[1][1].returnTo;
    const decoded = atob(returnTo);
    expect(decoded).toBe('#/nodoubleencode');
    expect(decoded).not.toMatch(/^%23/);
  });

  it('unauthenticated user with empty URL should redirect to login without returnTo', async () => {
    const protectedRoute = {
      name: 'protected',
      url: '/protected',
      component: () => null,
      data: { visibilityRequirements: { permissions: ['admin:all'] } }
    };
    router.stateRegistry.register(protectedRoute);
    await router.urlService.sync();

    const goSpy = jest.spyOn(router.stateService, 'go');
    jest.spyOn(router.urlService, 'url').mockReturnValue('');
    await router.stateService.go('protected').catch(() => {});

    expect(goSpy).toHaveBeenCalledTimes(2);
    expect(goSpy).toHaveBeenNthCalledWith(1, 'protected');
    expect(goSpy).toHaveBeenNthCalledWith(2, 'login');
  });

  it('unauthenticated user accessing child route with parent visibilityRequirements should redirect to login', async () => {

    router.stateRegistry.register({
      name: 'preview.admin',
      url: 'admin',
      abstract: true,
      component: UIView,
    });
    router.stateRegistry.register({
      name: 'preview.admin.security',
      url: '/security',
      component: UIView,
      data: {
        visibilityRequirements: { permissions: ['astrum:security:read'] },
      },
    });
    router.stateRegistry.register({
      name: 'preview.admin.security.roles',
      url: '/roles',
      component: () => null,
      data: { title: 'Roles' },
    });
    await router.urlService.sync();

    const goSpy = jest.spyOn(router.stateService, 'go');
    jest.spyOn(router.urlService, 'url').mockReturnValue('admin/security/roles');
    await router.stateService.go('preview.admin.security.roles').catch(() => {});

    expect(goSpy).toHaveBeenCalledWith('login', { returnTo: btoa('#admin/security/roles') });
  });

  it('authenticated user accessing child route with parent visibilityRequirements should redirect to preview welcome', async () => {
    ExtJS.hasUser.mockReturnValue(true);

    router.stateRegistry.register({
      name: 'preview.admin',
      url: 'admin',
      abstract: true,
      component: UIView,
    });
    router.stateRegistry.register({
      name: 'preview.admin.security',
      url: '/security',
      component: UIView,
      data: {
        visibilityRequirements: { permissions: ['astrum:security:read'] },
      },
    });
    router.stateRegistry.register({
      name: 'preview.admin.security.roles',
      url: '/roles',
      component: () => null,
      data: { title: 'Roles' },
    });
    await router.urlService.sync();

    const goSpy = jest.spyOn(router.stateService, 'go');
    await router.stateService.go('preview.admin.security.roles').catch(() => {});

    expect(goSpy).toHaveBeenCalledWith('preview.browse.welcome');
  });

  it('unauthenticated user accessing preview child route with parent visibilityRequirements should redirect to login with returnTo', async () => {
    router.stateRegistry.register({
      name: 'preview.admin',
      url: 'admin',
      abstract: true,
      component: UIView,
    });
    router.stateRegistry.register({
      name: 'preview.admin.security',
      url: '/security',
      component: UIView,
      data: {
        visibilityRequirements: { permissions: ['astrum:security:read'] },
      },
    });
    router.stateRegistry.register({
      name: 'preview.admin.security.roles',
      url: '/roles',
      component: () => null,
      data: { title: 'Roles' },
    });
    router.stateRegistry.register({
      name: 'preview.admin.security.roles.profile',
      url: '/:roleId/profile',
      component: () => null,
      data: { title: 'Role Profile' },
    });
    await router.urlService.sync();

    const goSpy = jest.spyOn(router.stateService, 'go');
    const urlSpy = jest.spyOn(router.urlService, 'url').mockReturnValue('admin/security/roles/asd/profile');
    await router.stateService.go('preview.admin.security.roles.profile', { roleId: 'asd' }).catch(() => {});

    const expectedReturnTo = btoa('#admin/security/roles/asd/profile');
    expect(goSpy).toHaveBeenCalledWith('login', { returnTo: expectedReturnTo });
    expect(urlSpy).toHaveBeenCalled();
  });

  it('authenticated user accessing preview protected route should redirect to preview.browse.welcome', async () => {
    ExtJS.hasUser.mockReturnValue(true);

    router.stateRegistry.register({
      name: 'preview.admin',
      url: 'admin',
      abstract: true,
      component: UIView,
    });
    router.stateRegistry.register({
      name: 'preview.admin.system',
      url: '/system',
      component: () => null,
      data: {
        visibilityRequirements: { permissions: ['astrum:settings:read'] },
      },
    });
    await router.urlService.sync();

    const goSpy = jest.spyOn(router.stateService, 'go');
    await router.stateService.go('preview.admin.system').catch(() => {});

    expect(goSpy).toHaveBeenCalledWith('preview.browse.welcome');
  });

  it('unauthenticated user can navigate to public routes without redirect', async () => {
    await router.stateService.go('preview.browse.welcome');
    expect(router.stateService.current.name).toBe('preview.browse.welcome');

    await router.stateService.go('login');
    expect(router.stateService.current.name).toBe('login');
  });

  it('transition from login to protected route multiple times should remain on login', async () => {
    const protectedRoute = {
      name: 'protected',
      url: '/protected',
      component: () => null,
      data: { visibilityRequirements: { permissions: ['admin:all'] } }
    };
    router.stateRegistry.register(protectedRoute);
    await router.urlService.sync();

    await router.stateService.go('login');

    await router.stateService.go('protected').catch(() => {});
    expect(router.stateService.current.name).toBe('login');

    await router.stateService.go('protected').catch(() => {});
    expect(router.stateService.current.name).toBe('login');

    await router.stateService.go('protected').catch(() => {});
    expect(router.stateService.current.name).toBe('login');
  });
});

describe('createRouter - otherwise handler - unrecognized URLs', () => {
  let router;

  beforeEach(() => {
    jest.clearAllMocks();
    global.AS.Permissions.check.mockReturnValue(false);
    if (!global.AS.Security) {
      global.AS.Security = { hasUser: jest.fn() };
    }
    ExtJS.hasUser = jest.fn().mockReturnValue(false);
    ExtJS.state = jest.fn().mockReturnValue({
      getValue: jest.fn().mockReturnValue(false)
    });
    ExtJS.isAuthStateResolved = jest.fn().mockReturnValue(true);
    ExtJS.whenAuthStateResolved = jest.fn().mockResolvedValue(undefined);
    ExtJS.onStateChange = jest.fn(() => () => {});
    router = getTestRouter();
  });

  afterEach(() => {
    window.location.hash = '';
  });

  it('unauthenticated user navigating to an unrecognized settings URL should redirect to login with returnTo', async () => {
    global.AS.Security.hasUser.mockReturnValue(false);
    window.location.hash = '#settings/unknown/page';

    const goSpy = jest.spyOn(router.stateService, 'go');
    jest.spyOn(router.urlService, 'url').mockReturnValue('settings/unknown/page');

    await router.urlService.sync();
    await new Promise(resolve => setTimeout(resolve, 0));

    const expectedReturnTo = btoa('#settings/unknown/page');
    expect(goSpy).toHaveBeenCalledWith('login', { returnTo: expectedReturnTo });
  });

  it('authenticated user navigating to an unrecognized settings URL should go to 404', async () => {
    global.AS.Security.hasUser.mockReturnValue(true);
    window.location.hash = '#settings/nonexistent/page';

    const goSpy = jest.spyOn(router.stateService, 'go');

    await router.urlService.sync();
    await new Promise(resolve => setTimeout(resolve, 0));

    expect(goSpy).toHaveBeenCalledWith('missing.route', {}, { location: false });
  });

  it('unauthenticated user navigating to an unrecognized non-protected URL should go to 404', async () => {
    global.AS.Security.hasUser.mockReturnValue(false);
    window.location.hash = '#nonexistent/page';

    const goSpy = jest.spyOn(router.stateService, 'go');

    await router.urlService.sync();
    await new Promise(resolve => setTimeout(resolve, 0));

    expect(goSpy).toHaveBeenCalledWith('missing.route', {}, { location: false });
  });

  it('falls back to bootstrap data when AS.Security is not available', async () => {
    delete global.AS.Security;
    window.__asRestBootstrap = { user: { authenticated: false } };
    window.location.hash = '#settings/some/path';

    const goSpy = jest.spyOn(router.stateService, 'go');
    jest.spyOn(router.urlService, 'url').mockReturnValue('settings/some/path');

    await router.urlService.sync();
    await new Promise(resolve => setTimeout(resolve, 0));

    const expectedReturnTo = btoa('#settings/some/path');
    expect(goSpy).toHaveBeenCalledWith('login', { returnTo: expectedReturnTo });
  });

  it('redirects to login without returnTo when url is empty for protected path', async () => {
    global.AS.Security = { hasUser: jest.fn().mockReturnValue(false) };
    window.location.hash = '#settings/some/path';

    const goSpy = jest.spyOn(router.stateService, 'go');
    jest.spyOn(router.urlService, 'url').mockReturnValue('');

    await router.urlService.sync();
    await new Promise(resolve => setTimeout(resolve, 0));

    expect(goSpy).toHaveBeenCalledWith('login');
  });
});

describe('createRouter - URL rules - Astrum One at the root of the hash', () => {
  let router;

  /** Sets the hash, lets the router follow it, and waits for the redirects to settle. */
  async function open(hash) {
    window.location.hash = hash;
    await router.urlService.sync();
    for (let i = 0; i < 5; i++) {
      await new Promise(resolve => setTimeout(resolve, 0));
    }
  }

  beforeEach(() => {
    jest.clearAllMocks();
    global.AS.Permissions.check.mockReturnValue(true);
    global.AS.Security = { hasUser: jest.fn().mockReturnValue(true) };
    ExtJS.hasUser = jest.fn().mockReturnValue(true);
    ExtJS.state = jest.fn().mockReturnValue({ getValue: jest.fn().mockReturnValue(false) });
    ExtJS.isAuthStateResolved = jest.fn().mockReturnValue(true);
    ExtJS.whenAuthStateResolved = jest.fn().mockResolvedValue(undefined);
    ExtJS.onStateChange = jest.fn(() => () => {});
    router = getTestRouter();
    router.stateRegistry.register({ name: 'preview.admin', url: 'admin', abstract: true, component: UIView });
    router.stateRegistry.register({ name: 'preview.admin.repository', url: '/repository', abstract: true, component: UIView });
    router.stateRegistry.register({
      name: 'preview.admin.repository.cleanuppolicies',
      url: '/cleanup-policies',
      component: () => null,
    });
    router.stateRegistry.register({ name: 'preview.settings', url: 'settings', component: () => null });
    router.urlService.listen();
  });

  afterEach(() => {
    router.urlService.listen(false);
    window.location.hash = '';
  });

  it.each([
    ['#preview/browse/welcome', '#browse/welcome', 'preview.browse.welcome'],
    ['#/preview/browse/welcome', '#browse/welcome', 'preview.browse.welcome'],
    ['#preview/settings', '#settings', 'preview.settings'],
    ['#preview', '#browse/welcome', 'preview.browse.welcome'],
  ])('opens %s at %s, without the old preview/ prefix', async (hash, expectedHash, expectedState) => {
    await open(hash);

    expect(window.location.hash).toBe(expectedHash);
    expect(router.stateService.current.name).toBe(expectedState);
  });

  it('keeps the query string when it drops the preview/ prefix', async () => {
    await open('#preview/browse/welcome?tab=usage-metrics&debug');

    expect(window.location.hash).toBe('#browse/welcome?tab=usage-metrics&debug');
    expect(router.stateService.current.name).toBe('preview.browse.welcome');
  });

  it('maps a Classic search URL before a state takes it for a repository named search', async () => {
    router.stateRegistry.register({ name: 'preview.browse.repo', url: '/{repoName:[^/]+}/*path', component: () => null });
    const replaced = [];
    const onHashChange = () => replaced.push(window.location.hash);
    window.addEventListener('hashchange', onHashChange);
    try {
      await open('#browse/search/maven=keyword%3Djunit');
    } finally {
      window.removeEventListener('hashchange', onHashChange);
    }

    expect(replaced).toContain('#browse/search?q=junit&format=maven');
    expect(router.stateService.current.name).not.toBe('preview.browse.repo');
  });

  it('drops the leading slash of a path no state matches', async () => {
    await open('#/browse/welcome');

    expect(window.location.hash).toBe('#browse/welcome');
    expect(router.stateService.current.name).toBe('preview.browse.welcome');
  });

  it.each([
    ['#admin/repository/cleanuppolicies', '#admin/repository/cleanup-policies', 'preview.admin.repository.cleanuppolicies'],
    ['#admin', '#settings', 'preview.settings'],
    ['#browse/unknown', '#browse/welcome', 'preview.browse.welcome'],
    ['#user/NuGetApiToken', '#browse/welcome', 'preview.browse.welcome'],
  ])('opens the Classic URL %s at %s', async (hash, expectedHash, expectedState) => {
    await open(hash);

    expect(window.location.hash).toBe(expectedHash);
    expect(router.stateService.current.name).toBe(expectedState);
  });

  it.each([
    ['browse/welcome', 'preview.browse.welcome'],
    ['admin/repository/cleanup-policies', 'preview.admin.repository.cleanuppolicies'],
    ['/login', 'login'],
  ])('lets the state of %s match before the rules for old URLs', (path, expectedState) => {
    const match = router.urlService.match({ path, search: {}, hash: '' });

    expect(match.rule.state.name).toBe(expectedState);
  });

  it('opens the 404 page for a Classic URL whose Astrum One page is not registered', async () => {
    const goSpy = jest.spyOn(router.stateService, 'go');

    await open('#admin/system/capabilities');

    expect(window.location.hash).toBe('#admin/system/capabilities');
    expect(goSpy).toHaveBeenCalledWith('missing.route', {}, { location: false });
  });

  it('keeps the sign-in page and the 404 page', async () => {
    ExtJS.hasUser.mockReturnValue(false);

    await open('#/login');
    expect(router.stateService.current.name).toBe('login');

    const goSpy = jest.spyOn(router.stateService, 'go');
    await open('#404');
    expect(goSpy).not.toHaveBeenCalledWith('preview.browse.welcome');
  });
});

describe('createRouter - preserve URL hash on browser refresh', () => {
  let router;

  beforeEach(() => {
    jest.clearAllMocks();
    global.AS.Permissions.check.mockReturnValue(false);
    if (!global.AS.Security) {
      global.AS.Security = { hasUser: jest.fn() };
    }
    ExtJS.hasUser = jest.fn().mockReturnValue(true);
    ExtJS.state = jest.fn().mockReturnValue({ getValue: jest.fn().mockReturnValue(false) });
    ExtJS.isAuthStateResolved = jest.fn().mockReturnValue(true);
    ExtJS.whenAuthStateResolved = jest.fn().mockResolvedValue(undefined);
    ExtJS.onStateChange = jest.fn(() => () => {});
  });

  afterEach(() => {
    window.location.hash = '';
  });

  it('honors the URL hash for a registered route on browser refresh, not overriding with initialRoute', async () => {
    // Uses #/login (not the initialRoute preview.browse.welcome) — if the initial rule fired
    // it would override to the welcome page; with the fix the hash wins and router lands on login.
    ExtJS.hasUser = jest.fn().mockReturnValue(false);
    window.location.hash = '#/login';
    router = getTestRouter();

    await router.urlService.sync();
    await new Promise(resolve => setTimeout(resolve, 0));

    expect(router.stateService.current.name).toBe('login');
  });

  it('does not redirect to initialRoute when URL has an unrecognized settings hash', async () => {
    // Without the fix the initial rule fires for unrecognized URLs and redirects to the
    // welcome page. With the fix the otherwise handler runs instead, routing authenticated
    // users to missing.route (404).
    global.AS.Security.hasUser.mockReturnValue(true);
    window.location.hash = '#settings/unknown';
    router = getTestRouter();

    const goSpy = jest.spyOn(router.stateService, 'go');

    await router.urlService.sync();
    await new Promise(resolve => setTimeout(resolve, 0));

    expect(goSpy).toHaveBeenCalledWith('missing.route', {}, { location: false });
    expect(goSpy).not.toHaveBeenCalledWith('preview.browse.welcome');
  });

  it('applies initial route rule when URL has only #/', async () => {
    // #/ is treated as empty — initial rule should fire, so the otherwise handler (missing.route)
    // is NOT used; the router falls through to the initial route instead.
    global.AS.Security.hasUser.mockReturnValue(true);
    window.location.hash = '#/';
    router = getTestRouter();

    const goSpy = jest.spyOn(router.stateService, 'go');

    await router.urlService.sync();
    await new Promise(resolve => setTimeout(resolve, 0));

    expect(goSpy).not.toHaveBeenCalledWith('missing.route', expect.anything(), expect.anything());
  });
});

describe('createRouter - onBefore - state with no visibilityRequirements', () => {
  let router;

  beforeEach(() => {
    jest.clearAllMocks();
    global.AS.Permissions.check.mockReturnValue(true);
    ExtJS.hasUser = jest.fn().mockReturnValue(true);
    ExtJS.state = jest.fn().mockReturnValue({ getValue: jest.fn().mockReturnValue(false) });
    ExtJS.isAuthStateResolved = jest.fn().mockReturnValue(true);
    ExtJS.whenAuthStateResolved = jest.fn().mockResolvedValue(undefined);
    ExtJS.onStateChange = jest.fn(() => () => {});
    router = getTestRouter();
  });

  it('allows navigation to a state with no visibilityRequirements', async () => {
    router.stateRegistry.register({
      name: 'unrestricted',
      url: '/unrestricted',
      component: () => null,
    });
    await router.urlService.sync();

    await router.stateService.go('unrestricted');

    expect(router.stateService.current.name).toBe('unrestricted');
  });
});

/*
 * ASTRUM-54290: under header/token authentication (rutauth-realm) the server-inlined
 * AS.app.state carries no user and there is no session cookie, so during bootstrap the client
 * cannot tell "no user" from "user not known yet". These tests cover both halves of the fix:
 * waiting for an authoritative answer before redirecting to login, and leaving login again if
 * the user turns up after the redirect already happened.
 */
describe('createRouter - onBefore - unresolved auth state (ASTRUM-54290)', () => {
  let router;
  let originalGetValue;

  /** Models the real Welcome page, which is gated on anonymousAccessOrHasUser. */
  const GATED_ROUTE = {
    name: 'gated',
    url: '/gated',
    component: () => null,
    data: { visibilityRequirements: { anonymousAccessOrHasUser: true } }
  };

  const GATED_RETURN_TO = btoa('#/gated');

  beforeEach(() => {
    jest.clearAllMocks();
    global.AS.Permissions.check.mockReturnValue(true);
    // An earlier suite deletes AS.Security; NavigationUtils needs it for the visibility check.
    global.AS.Security = { hasUser: jest.fn().mockReturnValue(false) };
    originalGetValue = global.AS.State.getValue;
    // No anonymous access, so visibility hinges entirely on there being a user
    global.AS.State.getValue = jest.fn().mockReturnValue(undefined);

    ExtJS.hasUser = jest.fn(() => global.AS.Security.hasUser());
    ExtJS.state = jest.fn().mockReturnValue({ getValue: jest.fn().mockReturnValue(false) });
    ExtJS.isAuthStateResolved = jest.fn().mockReturnValue(false);
    ExtJS.whenAuthStateResolved = jest.fn().mockResolvedValue(undefined);
    ExtJS.onStateChange = jest.fn(() => () => {});

    router = getTestRouter();
    router.stateRegistry.register(GATED_ROUTE);
  });

  afterEach(() => {
    global.AS.State.getValue = originalGetValue;
  });

  /** Returns the listener createRouter subscribed to 'userchanged', or undefined. */
  function recoveryListener() {
    const call = ExtJS.onStateChange.mock.calls.find(([events]) => events.includes('userchanged'));
    return call?.[1];
  }

  it('waits for the auth state, then allows the route when a user turns up', async () => {
    ExtJS.whenAuthStateResolved.mockImplementation(() => {
      global.AS.Security.hasUser.mockReturnValue(true);
      return Promise.resolve();
    });

    const goSpy = jest.spyOn(router.stateService, 'go');
    await router.stateService.go('gated').catch(() => {});

    expect(ExtJS.whenAuthStateResolved).toHaveBeenCalled();
    expect(goSpy).toHaveBeenCalledTimes(1);
    expect(goSpy).not.toHaveBeenCalledWith('login', expect.anything());
    expect(router.stateService.current.name).toBe('gated');
  });

  it('does not redirect to login while the auth state is still in flight', async () => {
    let resolveAuthState;
    ExtJS.whenAuthStateResolved.mockReturnValue(new Promise((resolve) => {
      resolveAuthState = resolve;
    }));

    const goSpy = jest.spyOn(router.stateService, 'go');
    jest.spyOn(router.urlService, 'url').mockReturnValue('/gated');
    const transition = router.stateService.go('gated').catch(() => {});

    // Let the hook run up to the await; nothing should have been decided yet
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(goSpy).not.toHaveBeenCalledWith('login', expect.anything());

    global.AS.Security.hasUser.mockReturnValue(true);
    resolveAuthState();
    await transition;

    expect(goSpy).not.toHaveBeenCalledWith('login', expect.anything());
    expect(router.stateService.current.name).toBe('gated');
  });

  it('redirects to login once the auth state resolves with no user', async () => {
    const goSpy = jest.spyOn(router.stateService, 'go');
    jest.spyOn(router.urlService, 'url').mockReturnValue('/gated');
    await router.stateService.go('gated').catch(() => {});

    expect(ExtJS.whenAuthStateResolved).toHaveBeenCalled();
    expect(goSpy).toHaveBeenCalledWith('login', { returnTo: GATED_RETURN_TO });
  });

  it('does not wait when the auth state is already resolved', async () => {
    ExtJS.isAuthStateResolved.mockReturnValue(true);

    const goSpy = jest.spyOn(router.stateService, 'go');
    jest.spyOn(router.urlService, 'url').mockReturnValue('/gated');
    await router.stateService.go('gated').catch(() => {});

    expect(ExtJS.whenAuthStateResolved).not.toHaveBeenCalled();
    expect(goSpy).toHaveBeenCalledWith('login', { returnTo: GATED_RETURN_TO });
  });

  it('resumes the original destination when a user arrives after the login redirect', async () => {
    jest.spyOn(router.urlService, 'url').mockReturnValue('/gated');
    await router.stateService.go('gated').catch(() => {});
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(router.stateService.current.name).toBe('login');

    const onUserChanged = recoveryListener();
    expect(onUserChanged).toBeDefined();

    const urlSpy = jest.spyOn(router.urlService, 'url');
    global.AS.Security.hasUser.mockReturnValue(true);
    onUserChanged();

    expect(urlSpy).toHaveBeenCalledWith('#/gated');
  });

  it('leaves login for the welcome page when a user arrives and there is no returnTo', async () => {
    const goSpy = jest.spyOn(router.stateService, 'go');
    await router.stateService.go('login');
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(router.stateService.current.name).toBe('login');

    const onUserChanged = recoveryListener();
    expect(onUserChanged).toBeDefined();

    global.AS.Security.hasUser.mockReturnValue(true);
    onUserChanged();

    expect(goSpy).toHaveBeenCalledWith('preview.browse.welcome');
  });

  it('stays on login when the state change brings no user', async () => {
    const goSpy = jest.spyOn(router.stateService, 'go');
    await router.stateService.go('login');
    await new Promise((resolve) => setTimeout(resolve, 0));

    const onUserChanged = recoveryListener();
    expect(onUserChanged).toBeDefined();

    goSpy.mockClear();
    onUserChanged();

    expect(goSpy).not.toHaveBeenCalled();
    expect(router.stateService.current.name).toBe('login');
  });

  it('does not watch for a late user when the auth state is already resolved', async () => {
    ExtJS.isAuthStateResolved.mockReturnValue(true);

    await router.stateService.go('login');

    expect(ExtJS.onStateChange).not.toHaveBeenCalled();
  });
});

describe('createRouter - onBefore - unresolved permissions state (ASTRUM-54422)', () => {
  let router;

  /**
   * Models a permission-gated preview route (e.g. the unified search page, gated
   * on astrum:search:read). On a hard reload, ASTRUM-52583's fast-render bootstrap
   * lets the router evaluate this before AS.Permissions has loaded, so the
   * visibility check below fails not because the user lacks the permission, but
   * because the answer has not arrived yet.
   */
  const GATED_ROUTE = {
    name: 'preview.gated',
    url: '/gated',
    component: () => null,
    data: { visibilityRequirements: { permissions: ['astrum:search:read'] } }
  };

  beforeEach(() => {
    jest.clearAllMocks();

    // A session cookie resolves hasUser()/isAuthStateResolved() immediately, well
    // before the separate permissions poll lands — the two are not the same signal.
    ExtJS.hasUser = jest.fn().mockReturnValue(true);
    ExtJS.isAuthStateResolved = jest.fn().mockReturnValue(true);
    ExtJS.whenAuthStateResolved = jest.fn().mockResolvedValue(undefined);
    ExtJS.onStateChange = jest.fn(() => () => {});
    ExtJS.state = jest.fn().mockReturnValue({ getValue: jest.fn().mockReturnValue(false) });

    // Permissions have not loaded yet: AS.Permissions.permissions is undefined
    // (see ExtJS.arePermissionsReady), so every check reads as denied.
    ExtJS.arePermissionsReady = jest.fn().mockReturnValue(false);
    global.AS.Permissions.check.mockReturnValue(false);

    router = getTestRouter();
    router.stateRegistry.register(GATED_ROUTE);
  });

  it('waits for permissions, then allows the route once they grant it', async () => {
    ExtJS.waitForPermissions = jest.fn().mockImplementation(() => {
      ExtJS.arePermissionsReady.mockReturnValue(true);
      global.AS.Permissions.check.mockReturnValue(true);
      return Promise.resolve();
    });

    const goSpy = jest.spyOn(router.stateService, 'go');
    await router.stateService.go('preview.gated').catch(() => {});

    // Bounded to a short settle window, NOT waitForPermissions()'s 30s default. This
    // wait only avoids a welcome-page flash; correctness comes from the recovery
    // subscription. Blocking for 30s here stalled navigation on a slow permissions
    // endpoint and timed out cloudui's App tests (ASTRUM-54422).
    expect(ExtJS.waitForPermissions).toHaveBeenCalledWith(500);
    expect(goSpy).not.toHaveBeenCalledWith('preview.browse.welcome');
    expect(router.stateService.current.name).toBe('preview.gated');
  });

  it('never blocks navigation for longer than the settle window', async () => {
    // Pins the regression directly: any caller passing no argument (or a large one)
    // inherits the 30s default and stalls the transition.
    ExtJS.waitForPermissions = jest.fn().mockResolvedValue(undefined);

    await router.stateService.go('preview.gated').catch(() => {});

    const [timeout] = ExtJS.waitForPermissions.mock.calls[0];
    expect(timeout).toBeLessThanOrEqual(1000);
  });

  it('falls back to the welcome redirect if permissions resolve without granting access', async () => {
    ExtJS.waitForPermissions = jest.fn().mockImplementation(() => {
      ExtJS.arePermissionsReady.mockReturnValue(true);
      // permissions loaded, but genuinely does not include astrum:search:read
      return Promise.resolve();
    });

    const goSpy = jest.spyOn(router.stateService, 'go');
    await router.stateService.go('preview.gated').catch(() => {});

    expect(ExtJS.waitForPermissions).toHaveBeenCalled();
    expect(goSpy).toHaveBeenCalledWith('preview.browse.welcome');
  });

  it('falls back to the welcome redirect if the permissions wait times out', async () => {
    ExtJS.waitForPermissions = jest.fn().mockRejectedValue(new Error('Permissions load timed out'));

    const goSpy = jest.spyOn(router.stateService, 'go');
    await router.stateService.go('preview.gated').catch(() => {});

    expect(goSpy).toHaveBeenCalledWith('preview.browse.welcome');
  });

  it('does not wait when permissions are already loaded', async () => {
    ExtJS.arePermissionsReady.mockReturnValue(true);
    ExtJS.waitForPermissions = jest.fn().mockResolvedValue(undefined);

    const goSpy = jest.spyOn(router.stateService, 'go');
    await router.stateService.go('preview.gated').catch(() => {});

    expect(ExtJS.waitForPermissions).not.toHaveBeenCalled();
    expect(goSpy).toHaveBeenCalledWith('preview.browse.welcome');
  });
});

describe('createRouter - onBefore - permissions arriving after the redirect (ASTRUM-54422)', () => {
  let router;
  let originalHash;

  /**
   * Models the empty-map state observed in the browser: arePermissionsReady() reads
   * true, so the wait is skipped, yet every check() is false. This is the case no
   * amount of waiting or timeout tuning can fix, and it is why recovery exists.
   */
  const GATED_ROUTE = {
    name: 'preview.gated',
    url: '/gated',
    component: () => null,
    data: { visibilityRequirements: { permissions: ['astrum:search:read'] } }
  };

  const SEARCH_HASH = '#gated?format=maven&nameOrVersion=commons';

  beforeEach(() => {
    jest.clearAllMocks();

    ExtJS.hasUser = jest.fn().mockReturnValue(true);
    ExtJS.isAuthStateResolved = jest.fn().mockReturnValue(true);
    ExtJS.whenAuthStateResolved = jest.fn().mockResolvedValue(undefined);
    ExtJS.onStateChange = jest.fn(() => () => {});
    ExtJS.state = jest.fn().mockReturnValue({ getValue: jest.fn().mockReturnValue(false) });

    // The empty-install state: "ready", but granting nothing.
    ExtJS.arePermissionsReady = jest.fn().mockReturnValue(true);
    ExtJS.waitForPermissions = jest.fn().mockResolvedValue(undefined);
    global.AS.Permissions.check.mockReturnValue(false);

    ExtJS.onPermissionsChange = jest.fn(() => () => {});

    originalHash = window.location.hash;
    window.location.hash = SEARCH_HASH;

    router = getTestRouter();
    router.stateRegistry.register(GATED_ROUTE);
  });

  afterEach(() => {
    window.location.hash = originalHash;
  });

  /** Returns the listener createRouter subscribed to permission changes. */
  function permissionsListener() {
    return ExtJS.onPermissionsChange.mock.calls[0]?.[0];
  }

  it('subscribes to permission changes when it redirects away', async () => {
    await router.stateService.go('preview.gated').catch(() => {});

    expect(ExtJS.onPermissionsChange).toHaveBeenCalled();
    expect(permissionsListener()).toBeDefined();
  });

  it('restores the original destination once permissions grant it', async () => {
    await router.stateService.go('preview.gated').catch(() => {});
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(router.stateService.current.name).toBe('preview.browse.welcome');

    // The real permission set lands.
    global.AS.Permissions.check.mockReturnValue(true);
    permissionsListener()();

    // Restored by hash assignment, not stateService.go, so undeclared params survive.
    expect(window.location.hash).toContain('nameOrVersion=commons');
    expect(window.location.hash).toContain('format=maven');
  });

  it('keeps waiting when a permission change still does not grant the route', async () => {
    await router.stateService.go('preview.gated').catch(() => {});
    await new Promise((resolve) => setTimeout(resolve, 0));

    // An empty install fires 'changed' too — it must not consume the subscription.
    permissionsListener()();
    expect(router.stateService.current.name).toBe('preview.browse.welcome');

    global.AS.Permissions.check.mockReturnValue(true);
    permissionsListener()();
    expect(window.location.hash).toContain('nameOrVersion=commons');
  });

  it('does not hijack navigation if the user has moved on', async () => {
    await router.stateService.go('preview.gated').catch(() => {});
    await new Promise((resolve) => setTimeout(resolve, 0));

    // User navigates somewhere else before permissions land.
    router.stateRegistry.register({
      name: 'preview.browse.elsewhere',
      url: '/elsewhere',
      component: () => null,
      data: { visibilityRequirements: {} },
    });
    await router.stateService.go('preview.browse.elsewhere').catch(() => {});
    const hashAfterUserNavigation = window.location.hash;

    global.AS.Permissions.check.mockReturnValue(true);
    permissionsListener()();

    expect(window.location.hash).toBe(hashAfterUserNavigation);
  });

  it('stops watching after the recovery timeout', async () => {
    jest.useFakeTimers();
    try {
      const unsubscribe = jest.fn();
      ExtJS.onPermissionsChange = jest.fn(() => unsubscribe);

      router = getTestRouter();
      router.stateRegistry.register(GATED_ROUTE);

      router.stateService.go('preview.gated').catch(() => {});
      // The transition resolves across a promise chain; alternate microtask flushes
      // with zero-length timer advances until the redirect branch has run.
      for (let i = 0; i < 50; i++) {
        jest.advanceTimersByTime(0);
        await Promise.resolve();
      }
      expect(ExtJS.onPermissionsChange).toHaveBeenCalled();

      jest.advanceTimersByTime(30000);
      expect(unsubscribe).toHaveBeenCalled();
    } finally {
      jest.useRealTimers();
    }
  });
});

