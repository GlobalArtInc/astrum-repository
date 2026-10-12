
import {
  heritageToPreviewPath,
  isClassicPath,
  isClassicSearchPath,
  PREVIEW_WELCOME_PATH,
  withoutPreviewPrefix,
} from '../previewHeritageNavigation';

describe('heritageToPreviewPath', () => {
  describe.each([
    ['dashboard', [
      ['', 'browse/welcome'],
      ['/', 'browse/welcome'],
      ['browse', 'browse/welcome'],
      ['browse/welcome', 'browse/welcome'],
      ['#browse/welcome', 'browse/welcome'],
      ['/browse/welcome', 'browse/welcome'],
      ['browse/welcome?tab=info', 'browse/welcome?tab=info'],
    ]],
    ['browse', [
      ['browse/browse', 'browse'],
      ['browse/browse:maven-releases', 'browse/maven-releases/'],
      ['browse/browse:my-repo:', 'browse/my-repo/'],
      ['browse/browse:my-repo:com%2Ffoo', 'browse/my-repo/com/foo'],
      ['browse/browse:my-repo:com/foo', 'browse/my-repo/com/foo'],
      ['browse/browse:my%20repo:org%2Fapache%2Fcommons%2F1.0', 'browse/my%20repo/org/apache/commons/1.0'],
      ['browse/browse:raw-hosted:docs%2Fmy%20file.txt', 'browse/raw-hosted/docs/my%20file.txt'],
      ['browse/browse:docker-hosted:v2%2Fsha256%3Aabc', 'browse/docker-hosted/v2/sha256%3Aabc'],
      ['browse/browse:my-repo?tab=info', 'browse/my-repo/?tab=info'],
    ]],
    ['search', [
      ['browse/search', 'browse/search'],
      ['browse/search=keyword%3Djunit', 'browse/search?q=junit'],
      ['browse/search/generic', 'browse/search'],
      ['browse/search/generic=keyword=hello', 'browse/search?q=hello'],
      ['browse/search/generic=keyword%3Dhello%20world', 'browse/search?q=hello%20world'],
      ['browse/search/generic=keyword%3Djunit%20AND%20repository_name%3Dmaven-central',
        'browse/search?q=junit&repository=maven-central'],
      ['browse/search/maven', 'browse/search?format=maven'],
      ['browse/search/maven2', 'browse/search?format=maven'],
      ['browse/search/maven=attributes.maven2.artifactId%3Djunit', 'browse/search?q=junit&format=maven'],
      ['browse/search/maven=keyword%3Djunit%20AND%20attributes.maven2.groupId%3Dorg.junit',
        'browse/search?q=junit&format=maven'],
      ['browse/search/maven/spring', 'browse/search?q=spring&format=maven'],
      ['browse/search/npm=name.raw%3Dreact%20AND%20version%3D18.0.0', 'browse/search?q=react%2018.0.0&format=npm'],
      ['browse/search/golang', 'browse/search?format=go'],
      ['browse/search/hugging_face', 'browse/search?format=huggingface'],
      ['browse/search/docker=attributes.docker.imageName%3Dnginx', 'browse/search?q=nginx&format=docker'],
      ['browse/search/keyword=keyword%3Dfoo', 'browse/search?q=foo'],
      ['browse/search/custom', 'browse/search/custom'],
      ['browse/search/custom=name.raw%3Dfoo', 'browse/search/custom'],
      ['browse/search/saved/my-search', 'browse/search'],
    ]],
    ['upload and tags', [
      ['browse/upload', 'browse/upload'],
      ['browse/upload/maven-releases', 'browse/upload/maven-releases'],
      ['browse/upload:raw%20hosted', 'browse/upload/raw%20hosted'],
      ['browse/tags', 'browse/tags'],
      ['browse/tags:release-1.0', 'browse/tags/release-1.0'],
    ]],
    ['settings sections', [
      ['admin', 'settings'],
      ['admin/', 'settings'],
      ['admin/hub', 'settings'],
      ['admin/repository', 'admin/repository/repositories'],
      ['admin/security', 'admin/security/users'],
      ['admin/support', 'admin/support/systeminformation'],
      ['admin/system', 'admin/system/tasks'],
    ]],
    ['repository', [
      ['admin/repository/repositories', 'admin/repository/repositories'],
      ['admin/repository/repositories:maven-central', 'admin/repository/repositories/maven-central'],
      ['admin/repository/repositories:Maven%20Central', 'admin/repository/repositories/Maven%20Central'],
      ['admin/repository/blobstores', 'admin/repository/blobstores'],
      ['admin/repository/blobstores/create', 'admin/repository/blobstores/create'],
      ['admin/repository/blobstores/edit/file/default', 'admin/repository/blobstores/file/default'],
      ['admin/repository/blobstores/edit/s3/my%20store', 'admin/repository/blobstores/s3/my%20store'],
      ['admin/repository/blobstores:default', 'admin/repository/blobstores'],
      ['admin/repository/selectors', 'admin/repository/selectors'],
      ['admin/repository/selectors/create', 'admin/repository/selectors/create'],
      ['admin/repository/selectors/edit/my-selector', 'admin/repository/selectors/my-selector'],
      ['admin/repository/selectors:my-selector', 'admin/repository/selectors/my-selector'],
      ['admin/repository/cleanuppolicies', 'admin/repository/cleanup-policies'],
      ['admin/repository/cleanuppolicies/create', 'admin/repository/cleanup-policies/create'],
      ['admin/repository/cleanuppolicies/edit/old-snapshots', 'admin/repository/cleanup-policies/old-snapshots'],
      ['admin/repository/routingrules', 'admin/repository/routing-rules'],
      ['admin/repository/routingrules/create', 'admin/repository/routing-rules/create'],
      ['admin/repository/routingrules/preview', 'admin/repository/routing-rules/preview'],
      ['admin/repository/routingrules/edit/block-internal', 'admin/repository/routing-rules/block-internal'],
      ['admin/repository/datastore', 'admin/repository/datastore'],
      ['admin/repository/proprietary', 'admin/repository/proprietary'],
    ]],
    ['security', [
      ['admin/security/privileges', 'admin/security/privileges'],
      ['admin/security/privileges/create', 'admin/security/privileges/create'],
      ['admin/security/privileges/edit/astrum-all', 'admin/security/privileges/astrum-all'],
      ['admin/security/roles', 'admin/security/roles'],
      ['admin/security/roles/create', 'admin/security/roles/create'],
      ['admin/security/roles/edit/astrum-admin', 'admin/security/roles/astrum-admin'],
      ['admin/security/users', 'admin/security/users'],
      ['admin/security/users:admin', 'admin/security/users/admin/default'],
      ['admin/security/users:default%2Fadmin', 'admin/security/users/admin/default'],
      ['admin/security/users:LDAP%2Fjane.doe', 'admin/security/users/jane.doe/LDAP'],
      ['admin/security/ldap', 'admin/security/ldap'],
      ['admin/security/ldap:corp', 'admin/security/ldap/corp'],
      ['admin/security/anonymous', 'admin/security/anonymous'],
      ['admin/security/realms', 'admin/security/realms'],
      ['admin/security/sslcertificates', 'admin/security/sslcertificates'],
      ['admin/security/sslcertificates/create', 'admin/security/sslcertificates'],
      ['admin/security/sslcertificates/edit/abc', 'admin/security/sslcertificates'],
      ['admin/security/oauth2', 'admin/security/oauth2'],
      ['admin/security/usertoken', 'admin/security/user-tokens'],
      ['admin/security/satokens', 'admin/security/service-account-tokens'],
      ['admin/security/atlassiancrowd', 'admin/security/crowd'],
      ['admin/security/saml', 'admin/security/saml'],
    ]],
    ['support', [
      ['admin/support/systeminformation', 'admin/support/systeminformation'],
      ['admin/support/status', 'admin/support/metrichealth'],
      ['admin/support/status:deadlocks', 'admin/support/metrichealth'],
      ['admin/support/supportzip', 'admin/support/supportzip'],
      ['admin/support/supportrequest', 'admin/support/supportrequest'],
      ['admin/support/logs', 'admin/support/logs'],
      ['admin/support/logs:astrum.log', 'admin/support/logs/astrum.log'],
      ['admin/support/logs:tasks%2Fcleanup.log', 'admin/support/logs/tasks/cleanup.log'],
      ['admin/support/recovery', 'admin/support/recoverymode'],
      ['admin/support/logging', 'admin/support/logging'],
      ['admin/support/logging/create', 'admin/support/logging/create'],
      ['admin/support/logging/edit/org.globalart', 'admin/support/logging/org.globalart'],
    ]],
    ['system', [
      ['admin/system/tasks', 'admin/system/tasks'],
      ['admin/system/tasks:3f1c', 'admin/system/tasks/3f1c'],
      ['admin/system/capabilities', 'admin/system/capabilities'],
      ['admin/system/capabilities/create', 'admin/system/capabilities/create'],
      ['admin/system/capabilities/edit/abc123', 'admin/system/capabilities/abc123'],
      ['admin/system/capabilities-extjs', 'admin/system/capabilities'],
      ['admin/system/capabilities-extjs:abc123', 'admin/system/capabilities/abc123'],
      ['admin/system/api', 'browse/api'],
      ['admin/system/emailserver', 'admin/system/emailserver'],
      ['admin/system/http', 'admin/system/http'],
      ['admin/system/nodes', 'admin/system/nodes'],
      ['admin/system/notifications', 'admin/system/notifications'],
      ['admin/system/previewui', 'settings'],
    ]],
    ['user', [
      ['user', 'user/account'],
      ['user/account', 'user/account'],
      ['user/account/settings', 'user/account'],
      ['user/accesstokens', 'user/accesstokens'],
      ['user/NuGetApiToken', 'user/nugetapitoken'],
      ['user/usertoken', 'user/usertoken'],
    ]],
    ['unknown', [
      ['user/other', PREVIEW_WELCOME_PATH],
      ['some/other', PREVIEW_WELCOME_PATH],
      ['admin/repository/unknown', PREVIEW_WELCOME_PATH],
      ['admin/security/users-and-more', PREVIEW_WELCOME_PATH],
      ['browse/unknown', PREVIEW_WELCOME_PATH],
      ['404', PREVIEW_WELCOME_PATH],
      ['login', PREVIEW_WELCOME_PATH],
      [null, PREVIEW_WELCOME_PATH],
      [undefined, PREVIEW_WELCOME_PATH],
    ]],
    ['Astrum One paths under the old preview/ prefix', [
      ['preview/browse/welcome', 'browse/welcome'],
      ['#preview/admin/security/users', 'admin/security/users'],
      ['/preview/browse/search?q=junit', 'browse/search?q=junit'],
      ['preview/admin/security/users/jdoe/default', 'admin/security/users/jdoe/default'],
      ['preview/browse/maven-releases/com/foo?tab=usage', 'browse/maven-releases/com/foo?tab=usage'],
      ['preview', PREVIEW_WELCOME_PATH],
      ['preview/', PREVIEW_WELCOME_PATH],
    ]],
  ])('%s', (_group, cases) => {
    it.each(cases)('maps %p to %p', (classic, preview) => {
      expect(heritageToPreviewPath(classic)).toBe(preview);
    });
  });

  it('opens the dashboard at the root of the hash', () => {
    expect(PREVIEW_WELCOME_PATH).toBe('browse/welcome');
  });

  it('opens the repository of a Classic repository URL', () => {
    expect(heritageToPreviewPath('admin/repository/repositories:npm-proxy-central'))
      .toBe('admin/repository/repositories/npm-proxy-central');
  });
});

describe('isClassicPath', () => {
  it.each([
    [''],
    ['/'],
    ['#browse/welcome'],
    ['browse'],
    ['browse/browse:maven-releases'],
    ['browse/search=keyword%3Djunit'],
    ['admin'],
    ['admin/repository/cleanuppolicies'],
    ['/admin/security/users:admin'],
    ['user'],
    ['user/NuGetApiToken'],
  ])('is true for %p', (path) => {
    expect(isClassicPath(path)).toBe(true);
  });

  it.each([
    ['settings'],
    ['settings/unknown'],
    ['preview/browse/welcome'],
    ['login'],
    ['404'],
    ['browser'],
    ['administrator'],
    ['users'],
    ['test'],
  ])('is false for %p', (path) => {
    expect(isClassicPath(path)).toBe(false);
  });
});

describe('isClassicSearchPath', () => {
  it.each([
    ['browse/search=keyword%3Djunit'],
    ['browse/search/maven'],
    ['#browse/search/maven?x=1'],
    ['/browse/search/maven=attributes.maven2.artifactId%3Djunit'],
    ['browse/search/maven/spring'],
    ['browse/search/generic'],
    ['browse/search/custom=name.raw%3Dfoo'],
    ['browse/search/custom/keyword'],
    ['browse/search/saved/my-search'],
  ])('is true for %p', (path) => {
    expect(isClassicSearchPath(path)).toBe(true);
  });

  it.each([
    ['browse/search'],
    ['browse/search?q=junit'],
    ['browse/search/'],
    ['browse/search/custom'],
    ['browse/search/component/maven%3Aorg.example%3Alib'],
    ['browse/search/component/maven%3Aorg.example%3Alib/versions'],
    ['browse/maven-releases/search/maven'],
    ['admin/search/maven'],
  ])('is false for %p', (path) => {
    expect(isClassicSearchPath(path)).toBe(false);
  });
});

describe('withoutPreviewPrefix', () => {
  it.each([
    ['preview/browse/welcome', 'browse/welcome'],
    ['#preview/admin/security/users', 'admin/security/users'],
    ['/preview/browse/search?q=junit&format=maven', 'browse/search?q=junit&format=maven'],
    ['preview//settings', 'settings'],
    ['preview', ''],
    ['preview?tab=x', '?tab=x'],
  ])('maps %p to %p', (path, expected) => {
    expect(withoutPreviewPrefix(path)).toBe(expected);
  });

  it.each([['browse/welcome'], ['previewui'], ['admin/repository/routing-rules/preview'], [''], [null], [undefined]])(
    'is null for %p', (path) => {
      expect(withoutPreviewPrefix(path)).toBeNull();
    });
});
