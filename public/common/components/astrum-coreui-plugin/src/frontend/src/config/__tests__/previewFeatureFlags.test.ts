
import {
  isDevelopmentMode,
  isMockMode,
  isFeatureEnabled,
  canPreviewWip,
  getWipPreviewUrl,
  PREVIEW_FEATURE_FLAGS,
} from '@globalart/astrum-ui-plugin/src/frontend/src/components/preview/config/featureFlags';

describe('previewFeatureFlags', () => {
  // Store original window properties
  const originalLocation = window.location;

  beforeEach(() => {
    // Reset window.location before each test
    delete window.location;
    window.location = {
      hostname: 'production.example.com',
      search: '',
      href: 'https://production.example.com/path',
    };
  });

  afterEach(() => {
    // Restore original location
    window.location = originalLocation;
  });

  describe('isDevelopmentMode', () => {
    it('returns false for production hostname without debug param', () => {
      window.location.hostname = 'astrum.company.com';
      window.location.search = '';
      
      expect(isDevelopmentMode()).toBe(false);
    });

    it('returns true for localhost', () => {
      window.location.hostname = 'localhost';
      window.location.search = '';
      
      expect(isDevelopmentMode()).toBe(true);
    });

    it('returns true for 127.0.0.1', () => {
      window.location.hostname = '127.0.0.1';
      window.location.search = '';
      
      expect(isDevelopmentMode()).toBe(true);
    });

    it('returns true when ?debug is present in URL', () => {
      window.location.hostname = 'production.example.com';
      window.location.search = '?debug';
      
      expect(isDevelopmentMode()).toBe(true);
    });

    it('returns true when debug is part of search params', () => {
      window.location.hostname = 'production.example.com';
      window.location.search = '?foo=bar&debug=true';
      
      expect(isDevelopmentMode()).toBe(true);
    });

    it('returns true for localhost with debug param', () => {
      window.location.hostname = 'localhost';
      window.location.search = '?debug';
      
      expect(isDevelopmentMode()).toBe(true);
    });
  });

  describe('isMockMode', () => {
    it('returns false in production (no debug, no mock param)', () => {
      window.location.hostname = 'production.example.com';
      window.location.search = '';

      expect(isMockMode()).toBe(false);
    });

    it('returns false in development without ?mock param', () => {
      window.location.hostname = 'localhost';
      window.location.search = '';

      expect(isMockMode()).toBe(false);
    });

    it('returns true in development with ?mock param', () => {
      window.location.hostname = 'localhost';
      window.location.search = '?mock';

      expect(isMockMode()).toBe(true);
    });

    it('returns true in development with mock=1', () => {
      window.location.hostname = 'localhost';
      window.location.search = '?mock=1';

      expect(isMockMode()).toBe(true);
    });

    it('does not activate mock mode in production even with ?mock param', () => {
      window.location.hostname = 'production.example.com';
      window.location.search = '?mock';

      expect(isMockMode()).toBe(false);
    });
  });

  describe('PREVIEW_FEATURE_FLAGS', () => {
    it('has repository features enabled per release spec', () => {
      expect(PREVIEW_FEATURE_FLAGS['repository.repositories']).toBe(true); // ASTRUM-52792
      expect(PREVIEW_FEATURE_FLAGS['repository.blobstores']).toBe(true); // ASTRUM-52793
      expect(PREVIEW_FEATURE_FLAGS['repository.selectors']).toBe(true); // ASTRUM-52782
      expect(PREVIEW_FEATURE_FLAGS['repository.cleanuppolicies']).toBe(true);
    });

    it('has security features partially enabled per release spec', () => {
      // Coming Soon
      expect(PREVIEW_FEATURE_FLAGS['security.privileges']).toBe(true); // ASTRUM-52808
      expect(PREVIEW_FEATURE_FLAGS['security.roles']).toBe(true);
      expect(PREVIEW_FEATURE_FLAGS['security.users']).toBe(true); // ASTRUM-52807
      // Enabled — ASTRUM-51085
      expect(PREVIEW_FEATURE_FLAGS['security.anonymous']).toBe(true);
      expect(PREVIEW_FEATURE_FLAGS['security.realms']).toBe(true);
      expect(PREVIEW_FEATURE_FLAGS['security.usertokens']).toBe(true); // Enabled — ASTRUM-53608
      expect(PREVIEW_FEATURE_FLAGS['security.ldap']).toBe(true); // Enabled — ASTRUM-54147
      expect(PREVIEW_FEATURE_FLAGS['security.crowd']).toBe(true); // Enabled — ASTRUM-54148
      expect(PREVIEW_FEATURE_FLAGS['security.saml']).toBe(true); // ASTRUM-52595
      expect(PREVIEW_FEATURE_FLAGS['security.oauth2']).toBe(true); // Enabled — ASTRUM-54266
      expect(PREVIEW_FEATURE_FLAGS['security.sslcertificates']).toBe(true); // Enabled — ASTRUM-54265
    });

    it('has support features enabled (Sprint 11)', () => {
      expect(PREVIEW_FEATURE_FLAGS['support.logs']).toBe(true);
      expect(PREVIEW_FEATURE_FLAGS['support.logging']).toBe(true);
      expect(PREVIEW_FEATURE_FLAGS['support.systeminfo']).toBe(true);
      expect(PREVIEW_FEATURE_FLAGS['support.supportrequest']).toBe(true);
      expect(PREVIEW_FEATURE_FLAGS['support.supportzip']).toBe(true);
      expect(PREVIEW_FEATURE_FLAGS['support.metrics']).toBe(true);
    });

    it('has system features per release spec', () => {
      // Enabled
      expect(PREVIEW_FEATURE_FLAGS['system.api']).toBe(true);
      expect(PREVIEW_FEATURE_FLAGS['system.nodes']).toBe(true);
      expect(PREVIEW_FEATURE_FLAGS['system.capabilities']).toBe(true);
      expect(PREVIEW_FEATURE_FLAGS['system.tasks']).toBe(true);
      expect(PREVIEW_FEATURE_FLAGS['system.http']).toBe(true); // ASTRUM-52594
      expect(PREVIEW_FEATURE_FLAGS['system.licensing']).toBe(true); // Enabled — ASTRUM-52900
      // Coming Soon
      expect(PREVIEW_FEATURE_FLAGS['system.upgrade']).toBe(false);
      expect(PREVIEW_FEATURE_FLAGS['system.emailserver']).toBe(true);
      // IQ Server (Lifecycle/Firewall entry page). The
      // hosted-repo-eval sub-flag was dropped as unused; enablement is gated
      // by the backend hostedRepositoryEvaluationEnabled State flag.
      expect(PREVIEW_FEATURE_FLAGS['iqserver-connected']).toBe(true);
    });

    it('has the Welcome usage metrics tab enabled by default', () => {
      expect(PREVIEW_FEATURE_FLAGS['welcome.usageMetrics']).toBe(true); // ASTRUM-54200
    });
  });

  describe('isFeatureEnabled', () => {
    it('returns true for officially enabled features', () => {
      expect(isFeatureEnabled('support.logs')).toBe(true);
      expect(isFeatureEnabled('system.nodes')).toBe(true);
      expect(isFeatureEnabled('security.ldap')).toBe(true); // Enabled — ASTRUM-54147
    });

    // These blocks need a flag that is genuinely still disabled. They used
    // security.sslcertificates until ASTRUM-54265 enabled it; system.upgrade is the
    // remaining Coming Soon flag. Whoever enables that one next has to move these
    // over again — or add a test-only flag if none is left.
    it('returns false for disabled features in production', () => {
      window.location.hostname = 'production.example.com';
      window.location.search = '';

      expect(isFeatureEnabled('system.upgrade')).toBe(false);
    });

    it('returns false for unknown features', () => {
      expect(isFeatureEnabled('unknown.feature')).toBe(false);
    });

    describe('development mode URL override', () => {
      beforeEach(() => {
        window.location.hostname = 'localhost';
      });

      it('returns true when enableWip matches feature key', () => {
        window.location.search = '?enableWip=system.upgrade';
        
        expect(isFeatureEnabled('system.upgrade')).toBe(true);
      });

      it('returns true when enableWip=all', () => {
        window.location.search = '?enableWip=all';
        
        expect(isFeatureEnabled('system.upgrade')).toBe(true);
        expect(isFeatureEnabled('system.capabilities')).toBe(true);
      });

      it('returns false when enableWip does not match', () => {
        window.location.search = '?enableWip=system.capabilities';
        
        expect(isFeatureEnabled('system.upgrade')).toBe(false);
      });
    });

    describe('production mode security', () => {
      it('ignores enableWip param in production', () => {
        window.location.hostname = 'production.example.com';
        window.location.search = '?enableWip=system.upgrade';
        
        expect(isFeatureEnabled('system.upgrade')).toBe(false);
      });

      it('ignores enableWip=all in production', () => {
        window.location.hostname = 'production.example.com';
        window.location.search = '?enableWip=all';
        
        expect(isFeatureEnabled('system.upgrade')).toBe(false);
      });
    });
  });

  describe('canPreviewWip', () => {
    it('returns false for enabled features', () => {
      window.location.hostname = 'localhost';

      expect(canPreviewWip('system.nodes')).toBe(false); // Enabled per spec
      expect(canPreviewWip('support.logs')).toBe(false); // Sprint 11 enabled
    });

    it('returns true for disabled features in development mode', () => {
      window.location.hostname = 'localhost';
      
      expect(canPreviewWip('system.upgrade')).toBe(true);
    });

    it('returns false for disabled features in production', () => {
      window.location.hostname = 'production.example.com';
      window.location.search = '';
      
      expect(canPreviewWip('system.upgrade')).toBe(false);
    });

    it('returns true when debug param present', () => {
      window.location.hostname = 'production.example.com';
      window.location.search = '?debug';
      
      expect(canPreviewWip('system.upgrade')).toBe(true);
    });
  });

  describe('getWipPreviewUrl', () => {
    it('adds enableWip param to current URL', () => {
      window.location.href = 'https://localhost:8081/#admin/settings';
      
      // Mock URL constructor
      const originalURL = global.URL;
      global.URL = class MockURL {
        constructor(href) {
          this.href = href;
          this.searchParams = new URLSearchParams();
        }
        toString() {
          const params = this.searchParams.toString();
          return params ? `${this.href.split('?')[0]}?${params}` : this.href;
        }
      };
      
      const result = getWipPreviewUrl('support.logs');
      
      expect(result).toContain('enableWip=support.logs');
      
      global.URL = originalURL;
    });
  });

});
