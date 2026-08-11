
import React from 'react';
import { render, screen } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import {
  UIRouterReact,
  UIRouter,
  servicesPlugin,
  memoryLocationPlugin,
} from '@uirouter/react';

import { NavItemBox } from '../NavItemBox';
import { PreviewUIContext } from '../PreviewUIContext';

// ASTRUM-54212: End-to-end guard that the "API" left-nav item is visible for a user
// holding only astrum:settings:read. This exercises the REAL chain:
//   NavItemBox -> useRouteVisibility -> useContextAwareRouteName -> router.stateRegistry
//   -> useIsVisible -> isVisible(route.data.visibilityRequirements)
// against a real uirouter registry seeded with the real preview.browse.api route data,
// rather than the fully-mocked unit tests. The Settings hub tile and this nav item both
// gate on astrum:settings:read via the same isVisible(), so this proves parity between them.

/** Mirrors the real preview.browse.api route data (previewBrowseRoutes.js). */
const API_VISIBILITY_REQUIREMENTS = {
  requiresUser: true,
  permissions: ['astrum:settings:read'],
};

function buildRouterWithApiRoute() {
  const router = new UIRouterReact();
  router.plugin(servicesPlugin);
  router.plugin(memoryLocationPlugin);

  router.stateRegistry.register({ name: 'preview', abstract: true });
  router.stateRegistry.register({ name: 'preview.browse', abstract: true });
  router.stateRegistry.register({
    name: 'preview.browse.api',
    url: '/api',
    data: {
      title: 'API',
      visibilityRequirements: API_VISIBILITY_REQUIREMENTS,
    },
  });

  // Note: the <UIRouter> component calls router.start() itself. Visibility resolution
  // only reads stateRegistry.get(), so no active transition/initial rule is needed.
  return router;
}

/** Grants exactly the supplied permissions (Shiro-agnostic: exact-match for the test). */
function grantPermissions(granted: string[]) {
  const set = new Set(granted);
  (global as any).NX.Permissions.check.mockImplementation((perm: string) => set.has(perm));
  (global as any).NX.Permissions.permissions = Object.fromEntries(granted.map((p) => [p, true]));
}

const renderApiNavItem = (router: UIRouterReact) =>
  render(
    <Theme>
      <UIRouter router={router}>
        <PreviewUIContext.Provider value={true}>
          <NavItemBox name="preview.browse.api" text="API" icon={null} isCollapsed={false} />
        </PreviewUIContext.Provider>
      </UIRouter>
    </Theme>
  );

describe('NavItemBox — preview.browse.api visibility (integration)', () => {
  let router: UIRouterReact;

  beforeEach(() => {
    router = buildRouterWithApiRoute();
    // Logged-in user (Security.hasUser() -> true satisfies requiresUser).
    (global as any).NX.Security.hasUser.mockReturnValue(true);
  });

  it('renders the API item for a logged-in user with only astrum:settings:read', () => {
    grantPermissions(['astrum:settings:read']);

    renderApiNavItem(router);

    expect(screen.getByText('API')).toBeInTheDocument();
  });

  it('hides the API item for a logged-in user without astrum:settings:read', () => {
    grantPermissions(['astrum:component:read']);

    renderApiNavItem(router);

    expect(screen.queryByText('API')).not.toBeInTheDocument();
  });
});
