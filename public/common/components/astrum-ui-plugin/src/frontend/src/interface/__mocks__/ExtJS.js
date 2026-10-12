

/**
 * Shared mock for ExtJS interface used across tests in astrum-ui-plugin
 * This mock provides default implementations for commonly used ExtJS methods
 * Individual tests can override specific methods as needed using jest.spyOn() or direct assignment
 */
export default class ExtJS {
  static showSuccessMessage = jest.fn();
  static showErrorMessage = jest.fn();
  static urlOf = jest.fn((path) => `https://testurl${path}`);
  static absolutePath = jest.fn((path) => `/absolute${path}`);
  static proLicenseUrl = jest.fn(() => 'https://testurl/PRO-LICENSE.html');
  static ceLicenseUrl = jest.fn(() => 'https://testurl/CE-LICENSE.html');

  static setDirtyStatus = jest.fn((key, isDirty) => {
    window.dirty = window.dirty || [];
    if (isDirty && window.dirty.indexOf(key) === -1) {
      window.dirty.push(key);
    } else if (!isDirty) {
      window.dirty = window.dirty.filter(it => it !== key);
    }
  });

  static requestConfirmation = jest.fn(() => Promise.resolve());
  static fetchAuthenticationToken = jest.fn(() => Promise.resolve('mock-token'));
  static requestSession = jest.fn(() => Promise.resolve({status: 200}));
  static normalizeAjaxResponse = jest.fn((response) => response);
  static requestAuthenticationToken = jest.fn(() => Promise.resolve('mock-auth-token'));
  static downloadUrl = jest.fn();

  static state = jest.fn(() => ExtJS._stateInstance);

  // Consistent state instance for when() calls - WITH USERTOKEN CAPABILITY
  static _stateInstance = {
    getValue: jest.fn((key) => {
      if (key === 'capabilityActiveTypes') return ['usertoken'];
      if (key === 'anonymousUsername') return 'anonymous';
      return undefined;
    }),
    getEdition: jest.fn(() => 'OSS'),
    getUser: jest.fn(() => ({ id: 'test-user', authenticated: true }))
  };

  static formatDate = jest.fn((_date, _format) => '2024-01-01');
  static isProEdition = jest.fn(() => true);
  static checkPermission = jest.fn(() => true);

  static useUser = jest.fn(() => ({
    id: 'test-user',
    authenticated: true,
    administrator: false,
    authenticatedRealms: []
  }));

  static useStatus = jest.fn(() => ({
    version: '3.0.0',
    edition: 'OSS'
  }));

  static useLicense = jest.fn(() => ({
    daysToExpiry: 30
  }));

  static useVisiblityWithChanges = jest.fn(() => true);
  static useState = jest.fn((getValue) => getValue());
  static usePermission = jest.fn((getValue) => getValue());

  static hasUser = jest.fn(() => true);
  static isAuthStateResolved = jest.fn(() => true);
  static whenAuthStateResolved = jest.fn(() => Promise.resolve());
  static onStateChange = jest.fn(() => () => {});
  static onPermissionsChange = jest.fn(() => () => {});
  static signOut = jest.fn();
  static waitForExtJs = jest.fn((callback) => callback());
  static arePermissionsReady = jest.fn(() => true);
  static waitForPermissions = jest.fn().mockResolvedValue(undefined);
}
