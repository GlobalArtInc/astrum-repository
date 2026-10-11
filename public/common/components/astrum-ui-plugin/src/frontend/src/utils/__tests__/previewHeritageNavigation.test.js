
import {
  getHeritageEquivalent,
  heritageToPreviewPath,
  previewBrowsePathToHeritageBrowseParam,
  PREVIEW_WELCOME_PATH,
} from '../previewHeritageNavigation';

describe('previewBrowsePathToHeritageBrowseParam', () => {
  it('maps repo-only path to browse param with encoded repo name', () => {
    expect(previewBrowsePathToHeritageBrowseParam('preview/browse/my-repo/'))
      .toBe('browse/browse:my-repo');
  });

  it('maps repo + node path to colon-separated browse param', () => {
    expect(previewBrowsePathToHeritageBrowseParam('preview/browse/my-repo/com/foo'))
      .toBe('browse/browse:my-repo:com%2Ffoo');
  });

  it('falls back to browse/browse for bare prefix', () => {
    expect(previewBrowsePathToHeritageBrowseParam('preview/browse/')).toBe('browse/browse');
  });

  it('falls back for non-matching prefix', () => {
    expect(previewBrowsePathToHeritageBrowseParam('other/path')).toBe('browse/browse');
  });
});

describe('getHeritageEquivalent', () => {
  it.each([
    ['preview/browse/welcome', 'browse/welcome'],
    ['preview/admin/repository/repositories', 'admin/repository/repositories'],
    ['preview/admin/repository/cleanup-policies', 'admin/repository/cleanuppolicies'],
    ['preview/admin/security/privileges', 'admin/security/privileges'],
    ['preview/admin/security/crowd', 'admin/security/atlassiancrowd'],
    ['preview/admin/support/metrichealth', 'admin/support/status'],
    ['preview/admin/support/recoverymode', 'admin/support/recovery'],
    ['preview/admin/system/tasks', 'admin/system/tasks'],
    ['preview/admin/repository/repositories/npm-proxy-central', 'admin/repository/repositories:npm-proxy-central'],
    ['preview/admin/repository/repositories/Maven%20Central', 'admin/repository/repositories:Maven%20Central'],
    ['preview/admin/repository/repositories/my-repo?tab=config', 'admin/repository/repositories:my-repo?tab=config'],
    ['preview/browse/welcome?tab=overview', 'browse/welcome?tab=overview'],
    ['preview/browse/my-repo/', 'browse/browse:my-repo'],
    ['preview/browse/my-repo/?tab=info', 'browse/browse:my-repo?tab=info'],
    ['preview/browse/my-repo/com/foo', 'browse/browse:my-repo:com%2Ffoo'],
    ['preview/browse/welcome/', 'browse/welcome'],
    ['/preview/admin/system/nodes', 'admin/system/nodes'],
  ])('maps %s to %s', (preview, classic) => {
    expect(getHeritageEquivalent(preview)).toBe(classic);
  });

  it.each([['admin/repository/repositories'], [''], [null], [undefined]])(
    'returns null for %p', (path) => {
      expect(getHeritageEquivalent(path)).toBeNull();
    });
});

describe('heritageToPreviewPath', () => {
  describe.each([
    ['dashboard', [
      ['', 'preview/browse/welcome'],
      ['/', 'preview/browse/welcome'],
      ['browse', 'preview/browse/welcome'],
      ['browse/welcome', 'preview/browse/welcome'],
      ['#browse/welcome', 'preview/browse/welcome'],
      ['/browse/welcome', 'preview/browse/welcome'],
      ['browse/welcome?tab=info', 'preview/browse/welcome?tab=info'],
    ]],
    ['browse', [
      ['browse/browse', 'preview/browse'],
      ['browse/browse:maven-releases', 'preview/browse/maven-releases/'],
      ['browse/browse:my-repo:', 'preview/browse/my-repo/'],
      ['browse/browse:my-repo:com%2Ffoo', 'preview/browse/my-repo/com/foo'],
      ['browse/browse:my-repo:com/foo', 'preview/browse/my-repo/com/foo'],
      ['browse/browse:my%20repo:org%2Fapache%2Fcommons%2F1.0', 'preview/browse/my%20repo/org/apache/commons/1.0'],
      ['browse/browse:raw-hosted:docs%2Fmy%20file.txt', 'preview/browse/raw-hosted/docs/my%20file.txt'],
      ['browse/browse:docker-hosted:v2%2Fsha256%3Aabc', 'preview/browse/docker-hosted/v2/sha256%3Aabc'],
      ['browse/browse:my-repo?tab=info', 'preview/browse/my-repo/?tab=info'],
    ]],
    ['search', [
      ['browse/search', 'preview/browse/search'],
      ['browse/search=keyword%3Djunit', 'preview/browse/search?q=junit'],
      ['browse/search/generic', 'preview/browse/search'],
      ['browse/search/generic=keyword=hello', 'preview/browse/search?q=hello'],
      ['browse/search/generic=keyword%3Dhello%20world', 'preview/browse/search?q=hello%20world'],
      ['browse/search/generic=keyword%3Djunit%20AND%20repository_name%3Dmaven-central',
        'preview/browse/search?q=junit&repository=maven-central'],
      ['browse/search/maven', 'preview/browse/search?format=maven'],
      ['browse/search/maven2', 'preview/browse/search?format=maven'],
      ['browse/search/maven=attributes.maven2.artifactId%3Djunit', 'preview/browse/search?q=junit&format=maven'],
      ['browse/search/maven=keyword%3Djunit%20AND%20attributes.maven2.groupId%3Dorg.junit',
        'preview/browse/search?q=junit&format=maven'],
      ['browse/search/maven/spring', 'preview/browse/search?q=spring&format=maven'],
      ['browse/search/npm=name.raw%3Dreact%20AND%20version%3D18.0.0', 'preview/browse/search?q=react%2018.0.0&format=npm'],
      ['browse/search/golang', 'preview/browse/search?format=go'],
      ['browse/search/hugging_face', 'preview/browse/search?format=huggingface'],
      ['browse/search/docker=attributes.docker.imageName%3Dnginx', 'preview/browse/search?q=nginx&format=docker'],
      ['browse/search/keyword=keyword%3Dfoo', 'preview/browse/search?q=foo'],
      ['browse/search/custom', 'preview/browse/search/custom'],
      ['browse/search/custom=name.raw%3Dfoo', 'preview/browse/search/custom'],
      ['browse/search/saved/my-search', 'preview/browse/search'],
    ]],
    ['upload and tags', [
      ['browse/upload', 'preview/browse/upload'],
      ['browse/upload/maven-releases', 'preview/browse/upload/maven-releases'],
      ['browse/upload:raw%20hosted', 'preview/browse/upload/raw%20hosted'],
      ['browse/tags', 'preview/browse/tags'],
      ['browse/tags:release-1.0', 'preview/browse/tags/release-1.0'],
    ]],
    ['settings sections', [
      ['admin', 'preview/settings'],
      ['admin/', 'preview/settings'],
      ['admin/hub', 'preview/settings'],
      ['admin/repository', 'preview/admin/repository/repositories'],
      ['admin/security', 'preview/admin/security/users'],
      ['admin/support', 'preview/admin/support/systeminformation'],
      ['admin/system', 'preview/admin/system/tasks'],
    ]],
    ['repository', [
      ['admin/repository/repositories', 'preview/admin/repository/repositories'],
      ['admin/repository/repositories:maven-central', 'preview/admin/repository/repositories/maven-central'],
      ['admin/repository/repositories:Maven%20Central', 'preview/admin/repository/repositories/Maven%20Central'],
      ['admin/repository/blobstores', 'preview/admin/repository/blobstores'],
      ['admin/repository/blobstores/create', 'preview/admin/repository/blobstores/create'],
      ['admin/repository/blobstores/edit/file/default', 'preview/admin/repository/blobstores/file/default'],
      ['admin/repository/blobstores/edit/s3/my%20store', 'preview/admin/repository/blobstores/s3/my%20store'],
      ['admin/repository/blobstores:default', 'preview/admin/repository/blobstores'],
      ['admin/repository/selectors', 'preview/admin/repository/selectors'],
      ['admin/repository/selectors/create', 'preview/admin/repository/selectors/create'],
      ['admin/repository/selectors/edit/my-selector', 'preview/admin/repository/selectors/my-selector'],
      ['admin/repository/selectors:my-selector', 'preview/admin/repository/selectors/my-selector'],
      ['admin/repository/cleanuppolicies', 'preview/admin/repository/cleanup-policies'],
      ['admin/repository/cleanuppolicies/create', 'preview/admin/repository/cleanup-policies/create'],
      ['admin/repository/cleanuppolicies/edit/old-snapshots', 'preview/admin/repository/cleanup-policies/old-snapshots'],
      ['admin/repository/routingrules', 'preview/admin/repository/routing-rules'],
      ['admin/repository/routingrules/create', 'preview/admin/repository/routing-rules/create'],
      ['admin/repository/routingrules/preview', 'preview/admin/repository/routing-rules/preview'],
      ['admin/repository/routingrules/edit/block-internal', 'preview/admin/repository/routing-rules/block-internal'],
      ['admin/repository/datastore', 'preview/admin/repository/datastore'],
      ['admin/repository/proprietary', 'preview/admin/repository/proprietary'],
    ]],
    ['security', [
      ['admin/security/privileges', 'preview/admin/security/privileges'],
      ['admin/security/privileges/create', 'preview/admin/security/privileges/create'],
      ['admin/security/privileges/edit/astrum-all', 'preview/admin/security/privileges/astrum-all'],
      ['admin/security/roles', 'preview/admin/security/roles'],
      ['admin/security/roles/create', 'preview/admin/security/roles/create'],
      ['admin/security/roles/edit/astrum-admin', 'preview/admin/security/roles/astrum-admin'],
      ['admin/security/users', 'preview/admin/security/users'],
      ['admin/security/users:admin', 'preview/admin/security/users/admin/default'],
      ['admin/security/users:default%2Fadmin', 'preview/admin/security/users/admin/default'],
      ['admin/security/users:LDAP%2Fjane.doe', 'preview/admin/security/users/jane.doe/LDAP'],
      ['admin/security/ldap', 'preview/admin/security/ldap'],
      ['admin/security/ldap:corp', 'preview/admin/security/ldap/corp'],
      ['admin/security/anonymous', 'preview/admin/security/anonymous'],
      ['admin/security/realms', 'preview/admin/security/realms'],
      ['admin/security/sslcertificates', 'preview/admin/security/sslcertificates'],
      ['admin/security/sslcertificates/create', 'preview/admin/security/sslcertificates'],
      ['admin/security/sslcertificates/edit/abc', 'preview/admin/security/sslcertificates'],
      ['admin/security/oauth2', 'preview/admin/security/oauth2'],
      ['admin/security/usertoken', 'preview/admin/security/user-tokens'],
      ['admin/security/satokens', 'preview/admin/security/service-account-tokens'],
      ['admin/security/atlassiancrowd', 'preview/admin/security/crowd'],
      ['admin/security/saml', 'preview/admin/security/saml'],
    ]],
    ['support', [
      ['admin/support/systeminformation', 'preview/admin/support/systeminformation'],
      ['admin/support/status', 'preview/admin/support/metrichealth'],
      ['admin/support/status:deadlocks', 'preview/admin/support/metrichealth'],
      ['admin/support/supportzip', 'preview/admin/support/supportzip'],
      ['admin/support/supportrequest', 'preview/admin/support/supportrequest'],
      ['admin/support/logs', 'preview/admin/support/logs'],
      ['admin/support/logs:astrum.log', 'preview/admin/support/logs/astrum.log'],
      ['admin/support/logs:tasks%2Fcleanup.log', 'preview/admin/support/logs/tasks/cleanup.log'],
      ['admin/support/recovery', 'preview/admin/support/recoverymode'],
      ['admin/support/logging', 'preview/admin/support/logging'],
      ['admin/support/logging/create', 'preview/admin/support/logging/create'],
      ['admin/support/logging/edit/org.globalart', 'preview/admin/support/logging/org.globalart'],
    ]],
    ['system', [
      ['admin/system/tasks', 'preview/admin/system/tasks'],
      ['admin/system/tasks:3f1c', 'preview/admin/system/tasks/3f1c'],
      ['admin/system/capabilities', 'preview/admin/system/capabilities'],
      ['admin/system/capabilities/create', 'preview/admin/system/capabilities/create'],
      ['admin/system/capabilities/edit/abc123', 'preview/admin/system/capabilities/abc123'],
      ['admin/system/capabilities-extjs', 'preview/admin/system/capabilities'],
      ['admin/system/capabilities-extjs:abc123', 'preview/admin/system/capabilities/abc123'],
      ['admin/system/api', 'preview/browse/api'],
      ['admin/system/emailserver', 'preview/admin/system/emailserver'],
      ['admin/system/http', 'preview/admin/system/http'],
      ['admin/system/nodes', 'preview/admin/system/nodes'],
      ['admin/system/notifications', 'preview/admin/system/notifications'],
      ['admin/system/previewui', 'preview/settings'],
    ]],
    ['user', [
      ['user', 'preview/user/account'],
      ['user/account', 'preview/user/account'],
      ['user/account/settings', 'preview/user/account'],
      ['user/accesstokens', 'preview/user/accesstokens'],
      ['user/NuGetApiToken', 'preview/user/nugetapitoken'],
      ['user/usertoken', 'preview/user/usertoken'],
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
    ['Astrum One paths', [
      ['preview/browse/welcome', 'preview/browse/welcome'],
      ['preview/admin/security/users', 'preview/admin/security/users'],
      ['/preview/browse/search?q=junit', 'preview/browse/search?q=junit'],
    ]],
  ])('%s', (_group, cases) => {
    it.each(cases)('maps %p to %p', (classic, preview) => {
      expect(heritageToPreviewPath(classic)).toBe(preview);
    });
  });

  it('round-trips repository entity paths through the Classic UI', () => {
    const original = 'preview/admin/repository/repositories/npm-proxy-central';
    expect(heritageToPreviewPath(getHeritageEquivalent(original))).toBe(original);
  });
});
