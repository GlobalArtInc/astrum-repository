import { getRouter } from '../routerConfig';

/** The state a hash path opens, or the type of the URL rule that handles it instead. */
function matchOf(router, path) {
  const match = router.urlService.match({ path, search: {}, hash: '' });
  return match?.rule?.state?.name ?? match?.rule?.type;
}

describe('getRouter', () => {
  let router;

  beforeEach(() => {
    router = getRouter();
  });

  it('registers no Classic states: only Astrum One, the sign-in page and the 404 page', () => {
    const names = router.stateRegistry.get().map((state) => state.name).filter(Boolean);

    expect(names.filter((name) => name !== 'preview' && !name.startsWith('preview.')).sort())
      .toEqual(['login', 'missing_route']);
    expect(names).toEqual(expect.arrayContaining(['preview.browse.welcome', 'preview.settings', 'preview.user.account']));
  });

  it.each([
    ['preview.browse.browse', {}, '#browse'],
    ['preview.browse.search.unified', {}, '#browse/search'],
    ['preview.settings', {}, '#settings'],
    ['preview.admin.security.users.list', {}, '#admin/security/users'],
    ['preview.admin.repository.cleanuppolicies.list', {}, '#admin/repository/cleanup-policies'],
    ['preview.user.account', {}, '#user/account'],
    ['preview.browse.browse.repo', { repoName: 'maven-releases' }, '#browse/maven-releases/'],
    ['login', {}, '#/login'],
    ['missing_route', {}, '#404'],
  ])('serves %s at %s without the preview/ prefix', (name, params, expectedHref) => {
    expect(router.stateService.href(name, params)).toBe(expectedHref);
  });

  it.each([
    ['browse/welcome', 'preview.browse.welcome'],
    ['browse', 'preview.browse.browse'],
    ['browse/maven-releases/com/foo', 'preview.browse.browse.repo.path'],
    ['browse/search', 'preview.browse.search.unified'],
    ['browse/search/component/maven:org.example:lib', 'preview.browse.search.component'],
    ['settings', 'preview.settings'],
    ['admin', 'preview.admin.pages'],
    ['admin/security/users/jdoe/default', 'preview.admin.security.users.detail'],
    ['admin/system/capabilities', 'preview.admin.system.capabilities.list'],
    ['user/account', 'preview.user.account'],
    ['/login', 'login'],
    ['404', 'missing_route'],
  ])('opens %s at its Astrum One state', (path, expectedState) => {
    expect(matchOf(router, path)).toBe(expectedState);
  });

  it.each([
    ['preview/browse/welcome'],
    ['/preview/admin/security/users'],
    ['/browse/welcome'],
    ['admin/repository/cleanuppolicies'],
    ['browse/browse:maven-releases'],
    ['user/NuGetApiToken'],
    ['admin/system/api'],
  ])('leaves %s to the URL rules that open its Astrum One page', (path) => {
    expect(matchOf(router, path)).not.toMatch(/^(preview\.|login$|missing_route$)/);
  });

  describe('old URLs', () => {
    afterEach(() => {
      window.location.hash = '';
    });

    it.each([
      ['#preview/browse/welcome', '#browse/welcome'],
      ['#preview/browse/welcome?tab=usage-metrics', '#browse/welcome?tab=usage-metrics'],
      ['#preview/admin/security/users', '#admin/security/users'],
      ['#preview/settings', '#settings'],
      ['#preview/browse/search?q=x', '#browse/search?q=x'],
      ['#/preview/user/account', '#user/account'],
      ['#/admin/system/tasks', '#admin/system/tasks'],
      ['#browse/browse:maven-releases', '#browse/maven-releases/'],
      ['#browse/browse:maven-releases:org%2Fexample', '#browse/maven-releases/org/example'],
      ['#browse/search/maven', '#browse/search?format=maven'],
      ['#browse/search=keyword%3Djunit', '#browse/search?q=junit'],
      ['#admin/repository/cleanuppolicies', '#admin/repository/cleanup-policies'],
      ['#admin/repository/routingrules', '#admin/repository/routing-rules'],
      ['#admin/repository/repositories:maven-central', '#admin/repository/repositories/maven-central'],
      ['#admin/security/users:LDAP%2Fjane.doe', '#admin/security/users/jane.doe/LDAP'],
      ['#admin/support/status', '#admin/support/metrichealth'],
      ['#admin/support/recovery', '#admin/support/recoverymode'],
      ['#admin/system/api', '#browse/api'],
      ['#user/NuGetApiToken', '#user/nugetapitoken'],
      ['#admin/unknown', '#browse/welcome'],
    ])('opens %s at %s', (hash, expectedHash) => {
      window.location.hash = hash;

      router.urlService.sync();

      expect(window.location.hash).toBe(expectedHash);
    });
  });

  it('sends #admin, as the Classic settings link was, to the settings hub', () => {
    expect(router.stateRegistry.get('preview.admin.pages').redirectTo).toBe('preview.settings');
  });
});
