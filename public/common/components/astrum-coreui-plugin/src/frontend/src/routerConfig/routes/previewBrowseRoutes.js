
/**
 * Preview Browse Routes - SUPER UX Migration
 *
 * Search uses the unified search page for all formats.
 */

import {UIView} from '@uirouter/react';
import {
  Permissions,
  WelcomeSuper,
  BrowsePage,
  AuditLogPage,
  RepositoryProfilePage,
  TagsPage,
  TagDetailPage,
  UploadPage,
  UploadFormPage,
  ApiPage,
  GADetailPage,
  CustomSearchPage,
  ComingSoonPage,
} from '@globalart/astrum-ui-plugin';

export const previewBrowseRoutes = [
  // =============================================================================
  // ROOT ROUTES
  // =============================================================================
  {
    name: 'preview',
    url: 'preview',
    abstract: true,
    component: UIView,
  },
  {
    name: 'preview.browse',
    url: '/browse', // Nested under preview: #/preview/browse
    abstract: true,
    component: UIView,
    data: {title: 'Browse'},
  },

  // =============================================================================
  // DASHBOARD
  // =============================================================================
  {
    name: 'preview.browse.welcome',
    url: '/welcome?tab', // Nested: #/preview/browse/welcome
    component: WelcomeSuper,
    params: {
      tab: { type: 'string', dynamic: true, value: 'overview' },
    },
    data: {
      title: 'Welcome',
      visibilityRequirements: {
        anonymousAccessOrHasUser: true,
      },
    },
  },

  // =============================================================================
  // API DOCUMENTATION - Swagger UI (promoted from Settings, no sidebar)
  // Requires authentication to view
  // =============================================================================
  {
    name: 'preview.browse.api',
    url: '/api',
    component: ApiPage,
    data: {
      title: 'API',
      // ASTRUM-54048: API requires the same permission as the Default UI (adminRoutes.js
      // ADMIN.SYSTEM.API.ROOT) and Classic UI — astrum:settings:read. There is no
      // API-specific privilege in the platform; settings:read is the authoritative gate.
      visibilityRequirements: {
        requiresUser: true,
        permissions: [Permissions.SETTINGS.READ],
      },
    },
  },

  // =============================================================================
  // AUDIT LOG - IMPLEMENTED ✅
  // Requires authentication to view
  // =============================================================================
  {
    name: 'preview.browse.audit',
    url: '/audit?repositoryName',
    component: AuditLogPage,
    params: {
      repositoryName: { type: 'string', dynamic: true, value: null },
    },
    data: {
      title: 'Audit',
      visibilityRequirements: {
        requiresUser: true,
        permissions: ['astrum:audit:read'],
        statesEnabled: [
          {
            key: 'previewAuditEnabled',
            defaultValue: true,
          },
        ],
      },
    },
  },


  // =============================================================================
  // BROWSE - IMPLEMENTED ✅
  // =============================================================================
  {
    name: 'preview.browse.browse',
    url: '?format',
    component: BrowsePage,
    params: {
      format: { type: 'string', value: null, dynamic: true, inherit: true },
    },
    data: {
      title: 'Browse',
      // Mirrors Default UI BROWSE.BROWSE.ROOT (browseRoutes.js): visible to users with
      // any repository-view or content-selector permission and at least one browseable format.
      visibilityRequirements: {
        anonymousAccessOrHasUser: true,
        permissionPrefixes: ['astrum:repository-view', 'astrum:repository-content-selector'],
        statesEnabled: [{ key: 'browseableformats', defaultValue: [] }],
      },
    },
  },
  {
    name: 'preview.browse.browse.repo',
    url: '/{repoName:[^/]+}/',
    component: BrowsePage,
    params: {
      repoName: { value: null, dynamic: true },
      format: { type: 'string', value: null, dynamic: true, inherit: true },
    },
    data: {title: 'Browse Repository'},
  },
  {
    name: 'preview.browse.browse.repo.path',
    url: '*path?tab',
    component: BrowsePage,
    params: {
      repoName: { type: 'string', value: null, dynamic: true },
      path: { value: null, raw: true, dynamic: true },
      tab: { type: 'string', value: 'summary', dynamic: true }, // Default tab
      format: { type: 'string', value: null, dynamic: true, inherit: true },
    },
    data: {title: 'Browse'},
  },
  // Tab-based routing for asset detail views
  {
    name: 'preview.browse.browse.repo.path.summary',
    url: '/summary',
    component: BrowsePage,
    params: {
      repoName: { type: 'string', value: null, dynamic: true },
      path: { type: 'string', value: null, raw: true, dynamic: true },
      format: { type: 'string', value: null, dynamic: true, inherit: true },
      tab: { type: 'string', value: 'summary', dynamic: true },
    },
    data: {title: 'Summary'},
  },
  {
    name: 'preview.browse.browse.repo.path.usage',
    url: '/usage',
    component: BrowsePage,
    params: {
      repoName: { type: 'string', value: null, dynamic: true },
      path: { type: 'string', value: null, raw: true, dynamic: true },
      format: { type: 'string', value: null, dynamic: true, inherit: true },
      tab: { type: 'string', value: 'usage', dynamic: true },
    },
    data: {title: 'Usage'},
  },
  {
    name: 'preview.browse.browse.repo.path.attributes',
    url: '/attributes',
    component: BrowsePage,
    params: {
      repoName: { type: 'string', value: null, dynamic: true },
      path: { type: 'string', value: null, raw: true, dynamic: true },
      format: { type: 'string', value: null, dynamic: true, inherit: true },
      tab: { type: 'string', value: 'attributes', dynamic: true },
    },
    data: {title: 'Attributes'},
  },
  {
    name: 'preview.browse.browse.repo.path.tags',
    url: '/tags',
    component: BrowsePage,
    params: {
      repoName: { type: 'string', value: null, dynamic: true },
      path: { type: 'string', value: null, raw: true, dynamic: true },
      format: { type: 'string', value: null, dynamic: true, inherit: true },
      tab: { type: 'string', value: 'tags', dynamic: true },
    },
    data: {title: 'Component Tags'},
  },
  {
    name: 'preview.browse.browse.repo.path.lifecycle',
    url: '/lifecycle',
    component: BrowsePage,
    params: {
      repoName: { type: 'string', value: null, dynamic: true },
      path: { type: 'string', value: null, raw: true, dynamic: true },
      format: { type: 'string', value: null, dynamic: true, inherit: true },
      tab: { type: 'string', value: 'lifecycle', dynamic: true },
    },
    data: {title: 'GlobalArt Lifecycle'},
  },

  // =============================================================================
  // REPOSITORY PROFILE (Browse Context) - IMPLEMENTED ✅
  // Uses SAME component as Settings profile, but rendered without Settings sidebar
  // This is a sibling route so it replaces BrowsePage entirely
  // =============================================================================
  {
    name: 'preview.browse.repository-profile',
    url: '/repository-profile/:repositoryName?tab',
    component: RepositoryProfilePage,
    params: {
      repositoryName: { type: 'string', dynamic: true },
      tab: { value: null, type: 'string', dynamic: true },
    },
    resolve: [
      {
        token: 'repositoryName',
        deps: ['$stateParams'],
        resolveFn: ($stateParams) => $stateParams.repositoryName,
      },
      {
        token: 'context',
        resolveFn: () => 'browse', // Tells component to show "Back to Browse"
      },
    ],
    data: { title: 'Repository Profile' },
  },

  // =============================================================================
  // UPLOAD - IMPLEMENTED ✅
  // =============================================================================
  // Base upload route - abstract container
  {
    name: 'preview.browse.upload',
    url: '/upload',
    abstract: true,
    component: UIView,
    data: {
      title: 'Upload',
      // Mirrors Default UI BROWSE.UPLOAD.ROOT (browseRoutes.js).
      visibilityRequirements: {
        permissions: [Permissions.COMPONENT.CREATE],
      },
    },
  },
  // Upload list (repository selection)
  {
    name: 'preview.browse.upload.list',
    url: '',
    component: UploadPage,
    data: {title: 'Upload'},
  },
  // Upload form for a specific repository
  {
    name: 'preview.browse.upload.form',
    url: '/:repoName',
    component: UploadFormPage,
    params: {
      repoName: { type: 'string', value: null, dynamic: true },
    },
    data: {title: 'Upload to Repository'},
  },
  // Alias for compatibility with left nav which uses browse.upload.edit -> preview.browse.upload.edit
  {
    name: 'preview.browse.upload.edit',
    redirectTo: 'preview.browse.upload.form',
  },

  // =============================================================================
  // TAGS - IMPLEMENTED ✅
  // =============================================================================
  {
    name: 'preview.browse.tags',
    url: '/tags',
    component: TagsPage,
    data: {
      title: 'Tags',
      // Mirrors Default UI BROWSE.TAGS.ROOT (browseRoutes.js). The isProEdition gate in
      // LeftNavigationMenuRadix.tsx remains (now redundant with editions: ['PRO'], but harmless).
      visibilityRequirements: {
        permissions: [Permissions.TAGS.READ],
        editions: ['PRO'],
      },
    },
  },
  {
    name: 'preview.browse.tagdetail',
    url: '/tags/:tagName',
    component: TagDetailPage,
    params: {
      tagName: { type: 'string', value: null, dynamic: true },
    },
    data: {
      title: 'Tag Details',
      // Tag detail is a sibling of preview.browse.tags (its UIRouter parent is
      // preview.browse, which is ungated), so it does not inherit the tags gate via the
      // guard's ancestor walk. Gate it explicitly so a direct URL to a tag detail
      // redirects to the dashboard instead of loading and failing with a 403 (ASTRUM-54048).
      visibilityRequirements: {
        permissions: [Permissions.TAGS.READ],
        editions: ['PRO'],
      },
    },
  },

  // =============================================================================
  // SEARCH PARENT
  // =============================================================================
  {
    name: 'preview.browse.search',
    url: '/search',
    abstract: true,
    component: UIView,
    data: {
      title: 'Search',
      visibilityRequirements: {
        permissions: ['astrum:search:read'],
      },
    },
  },
  
  // =============================================================================
  // UNIFIED SEARCH PAGE - Main entry point for ALL formats
  // =============================================================================
  {
    name: 'preview.browse.search.unified',
    url: '?q&format&sort&direction',
    component: ComingSoonPage,
    params: {
      q: { value: null, dynamic: true },
      format: { value: null, dynamic: true },
      sort: { value: null, dynamic: true },
      direction: { value: null, dynamic: true },
    },
    data: {title: 'Search Components'},
  },

  // =============================================================================
  // CUSTOM SEARCH
  // =============================================================================
  {
    name: 'preview.browse.search.custom',
    url: '/custom',
    component: CustomSearchPage,
    data: {title: 'Custom Search'},
  },

  // =============================================================================
  // COMPONENT DETAIL
  // =============================================================================
  {
    name: 'preview.browse.search.component',
    url: '/component/:gaId?version',
    component: GADetailPage,
    params: {
      // 'path' (/[^/]*/), not 'string' (/.*/): the default string pattern matches across `/`, so
      // a deep link to a tab absorbed the segment and gaId became 'npm:dummy:name/overview'.
      // Leaving `raw` off keeps encoding symmetric — the router encodes on write and decodes on
      // read, so callers must pass the plain 'format:group:name' and never pre-encode it. A
      // pre-encoded value cannot round-trip, and because gaId is not `dynamic`, the mismatch
      // costs a state re-entry that remounts the page and re-fires every request (ASTRUM-54201).
      gaId: { type: 'path' },
      // dynamic: without it, changing `version` re-enters the state and remounts the page.
      version: { type: 'string', value: null, squash: true, dynamic: true },
    },
    resolve: [
      { token: 'gaId', deps: ['$stateParams'], resolveFn: ($stateParams) => $stateParams.gaId },
      { token: 'version', deps: ['$stateParams'], resolveFn: ($stateParams) => $stateParams.version },
    ],
    data: {title: 'Component Details'},
  },
  {
    name: 'preview.browse.search.component.overview',
    url: '/overview',
    component: GADetailPage,
    resolve: [
      { token: 'gaId', deps: ['$stateParams'], resolveFn: ($stateParams) => $stateParams.gaId },
      { token: 'version', deps: ['$stateParams'], resolveFn: ($stateParams) => $stateParams.version },
    ],
    data: {title: 'Overview'},
  },
  {
    name: 'preview.browse.search.component.versions',
    url: '/versions',
    component: GADetailPage,
    resolve: [
      { token: 'gaId', deps: ['$stateParams'], resolveFn: ($stateParams) => $stateParams.gaId },
      { token: 'version', deps: ['$stateParams'], resolveFn: ($stateParams) => $stateParams.version },
    ],
    data: {title: 'Versions'},
  },
  {
    name: 'preview.browse.search.component.repos',
    url: '/repos',
    component: GADetailPage,
    resolve: [
      { token: 'gaId', deps: ['$stateParams'], resolveFn: ($stateParams) => $stateParams.gaId },
      { token: 'version', deps: ['$stateParams'], resolveFn: ($stateParams) => $stateParams.version },
    ],
    data: {title: 'Repositories'},
  },
  {
    name: 'preview.browse.search.component.files',
    url: '/files',
    component: GADetailPage,
    resolve: [
      { token: 'gaId', deps: ['$stateParams'], resolveFn: ($stateParams) => $stateParams.gaId },
      { token: 'version', deps: ['$stateParams'], resolveFn: ($stateParams) => $stateParams.version },
    ],
    data: {title: 'Files'},
  },
  {
    name: 'preview.browse.search.component.security',
    url: '/security',
    component: GADetailPage,
    resolve: [
      { token: 'gaId', deps: ['$stateParams'], resolveFn: ($stateParams) => $stateParams.gaId },
      { token: 'version', deps: ['$stateParams'], resolveFn: ($stateParams) => $stateParams.version },
    ],
    data: {title: 'Security'},
  },
];
