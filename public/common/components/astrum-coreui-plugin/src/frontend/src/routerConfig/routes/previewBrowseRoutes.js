
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
  UnifiedSearchPage,
} from '@globalart/astrum-ui-plugin';
import PreviewRouteTitles from './previewRouteTitles';

export const previewBrowseRoutes = [
  // =============================================================================
  // ROOT ROUTES
  // =============================================================================
  // Astrum One owns the root of the hash: its first-level URLs have no leading slash, as in #browse/welcome.
  // The 'preview' state name stays; the URL rules of createRouter send the old #preview/... URLs here.
  {
    name: 'preview',
    url: '',
    abstract: true,
    component: UIView,
  },
  {
    name: 'preview.browse',
    url: 'browse', // #browse
    abstract: true,
    component: UIView,
    data: {title: PreviewRouteTitles.BROWSE},
  },

  // =============================================================================
  // DASHBOARD
  // =============================================================================
  {
    name: 'preview.browse.welcome',
    url: '/welcome?tab', // #browse/welcome
    component: WelcomeSuper,
    params: {
      tab: { type: 'string', dynamic: true, value: 'overview' },
    },
    data: {
      title: PreviewRouteTitles.WELCOME,
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
      title: PreviewRouteTitles.API,
      // ASTRUM-54048: API requires the same permission as the retired Classic page —
      // astrum:settings:read. There is no
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
      title: PreviewRouteTitles.AUDIT,
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
      title: PreviewRouteTitles.BROWSE,
      // Mirrors the retired Classic page: visible to users with
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
    data: {title: PreviewRouteTitles.BROWSE_REPOSITORY},
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
    data: {title: PreviewRouteTitles.BROWSE},
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
    data: {title: PreviewRouteTitles.SUMMARY},
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
    data: {title: PreviewRouteTitles.USAGE},
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
    data: {title: PreviewRouteTitles.ATTRIBUTES},
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
    data: {title: PreviewRouteTitles.COMPONENT_TAGS},
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
    data: {title: PreviewRouteTitles.GLOBALART_LIFECYCLE},
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
    data: { title: PreviewRouteTitles.REPOSITORY_PROFILE },
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
      title: PreviewRouteTitles.UPLOAD,
      // Mirrors the retired Classic page.
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
    data: {title: PreviewRouteTitles.UPLOAD},
  },
  // Upload form for a specific repository
  {
    name: 'preview.browse.upload.form',
    url: '/:repoName',
    component: UploadFormPage,
    params: {
      repoName: { type: 'string', value: null, dynamic: true },
    },
    data: {title: PreviewRouteTitles.UPLOAD_TO_REPOSITORY},
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
      title: PreviewRouteTitles.TAGS,
      // Mirrors the retired Classic page. The isProEdition gate in
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
      title: PreviewRouteTitles.TAG_DETAILS,
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
      title: PreviewRouteTitles.SEARCH,
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
    url: '?q&format&repository&sort&direction',
    component: UnifiedSearchPage,
    params: {
      q: { value: null, dynamic: true },
      format: { value: null, dynamic: true },
      repository: { value: null, dynamic: true },
      sort: { value: null, dynamic: true },
      direction: { value: null, dynamic: true },
    },
    data: {title: PreviewRouteTitles.SEARCH_COMPONENTS},
  },

  // =============================================================================
  // CUSTOM SEARCH
  // =============================================================================
  {
    name: 'preview.browse.search.custom',
    url: '/custom',
    component: CustomSearchPage,
    data: {title: PreviewRouteTitles.CUSTOM_SEARCH},
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
    data: {title: PreviewRouteTitles.COMPONENT_DETAILS},
  },
  {
    name: 'preview.browse.search.component.overview',
    url: '/overview',
    component: GADetailPage,
    resolve: [
      { token: 'gaId', deps: ['$stateParams'], resolveFn: ($stateParams) => $stateParams.gaId },
      { token: 'version', deps: ['$stateParams'], resolveFn: ($stateParams) => $stateParams.version },
    ],
    data: {title: PreviewRouteTitles.OVERVIEW},
  },
  {
    name: 'preview.browse.search.component.versions',
    url: '/versions',
    component: GADetailPage,
    resolve: [
      { token: 'gaId', deps: ['$stateParams'], resolveFn: ($stateParams) => $stateParams.gaId },
      { token: 'version', deps: ['$stateParams'], resolveFn: ($stateParams) => $stateParams.version },
    ],
    data: {title: PreviewRouteTitles.VERSIONS},
  },
  {
    name: 'preview.browse.search.component.repos',
    url: '/repos',
    component: GADetailPage,
    resolve: [
      { token: 'gaId', deps: ['$stateParams'], resolveFn: ($stateParams) => $stateParams.gaId },
      { token: 'version', deps: ['$stateParams'], resolveFn: ($stateParams) => $stateParams.version },
    ],
    data: {title: PreviewRouteTitles.REPOSITORIES},
  },
  {
    name: 'preview.browse.search.component.files',
    url: '/files',
    component: GADetailPage,
    resolve: [
      { token: 'gaId', deps: ['$stateParams'], resolveFn: ($stateParams) => $stateParams.gaId },
      { token: 'version', deps: ['$stateParams'], resolveFn: ($stateParams) => $stateParams.version },
    ],
    data: {title: PreviewRouteTitles.FILES},
  },
  {
    name: 'preview.browse.search.component.security',
    url: '/security',
    component: GADetailPage,
    resolve: [
      { token: 'gaId', deps: ['$stateParams'], resolveFn: ($stateParams) => $stateParams.gaId },
      { token: 'version', deps: ['$stateParams'], resolveFn: ($stateParams) => $stateParams.version },
    ],
    data: {title: PreviewRouteTitles.SECURITY},
  },
];
