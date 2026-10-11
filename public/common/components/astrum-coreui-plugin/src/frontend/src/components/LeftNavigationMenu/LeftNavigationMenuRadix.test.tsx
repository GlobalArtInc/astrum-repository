
import React from 'react';
import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';

import type { UseLeftNavigationMenuResult, UseSearchCollapsibleNavResult } from './useLeftNavigationMenu';

const mockUseSearchCollapsibleNav = jest.fn<UseSearchCollapsibleNavResult, [boolean]>();
const mockUseLeftNavigationMenu = jest.fn<UseLeftNavigationMenuResult, []>();

// Component tests mock the hook layer: presentation is verified in isolation.
jest.mock('./useLeftNavigationMenu', () => ({
  useSearchCollapsibleNav: (isCollapsed: boolean) => mockUseSearchCollapsibleNav(isCollapsed),
  useLeftNavigationMenu: () => mockUseLeftNavigationMenu(),
}));

// NavItemBox is the shared Preview UI primitive; stub it to a simple node.
// useContextAwareRouteName is used by the local Classic UI SearchNavItem leaf.
jest.mock('@globalart/astrum-ui-plugin', () => ({
  useContextAwareRouteName: (name: string) => name,
  NavItemBox: ({ name, text }: { name: string; text: string }) => <div data-testid={`nav-item-${name}`}>{text}</div>,
  PreviewUIContext: {
    Provider: ({ children }: { children: React.ReactNode }) => children,
  },
}));

// The local SearchNavItem leaf binds to the router; stub those bindings.
jest.mock('@uirouter/react', () => ({
  useIsActive: () => false,
  useSref: (name: string) => ({
    href: `#/${name}`,
    onClick: jest.fn(),
  }),
}));

jest.mock('../../routerConfig/routeNames/routeNames', () => ({
  ROUTE_NAMES: {
    BROWSE: {
      WELCOME: { ROOT: 'browse.welcome' },
      SEARCH: {
        ROOT: 'browse.search',
        UNIFIED: 'browse.search.unified',
        GENERIC: 'browse.search.generic',
        CUSTOM: 'browse.search.custom',
      },
      BROWSE: { ROOT: 'browse.browse' },
      UPLOAD: { ROOT: 'browse.upload', LIST: 'browse.upload.list' },
      TAGS: { ROOT: 'browse.tags' },
      MALWARERISK: { ROOT: 'browse.malwarerisk' },
    },
    ADMIN: {
      DIRECTORY: 'admin',
    },
  },
}));

jest.mock('@radix-ui/themes', () => ({
  ...jest.requireActual('@radix-ui/themes'),
  Box: ({ children, ...props }: { children?: React.ReactNode }) => <div {...props}>{children}</div>,
  Flex: ({ children, ...props }: { children?: React.ReactNode }) => <div {...props}>{children}</div>,
  ScrollArea: ({ children, ...props }: { children?: React.ReactNode }) => <div {...props}>{children}</div>,
  Separator: (props: Record<string, unknown>) => <hr {...props} />,
}));

jest.mock('../GlobalHeader/LanguageMenuRadix', () => () => <div data-testid="language-menu" />);
jest.mock('../GlobalHeader/HelpMenuRadix', () => () => <div data-testid="help-menu" />);

jest.mock('lucide-react', () => {
  const Icon = () => <span />;
  return {
    Repeat: Icon,
    X: Icon,
    LayoutDashboard: Icon,
    Search: Icon,
    FolderOpen: Icon,
    HardDriveUpload: Icon,
    Tags: Icon,
    Settings: Icon,
    ShieldAlert: Icon,
    FileCode: Icon,
    ChevronDown: Icon,
    ChevronUp: Icon,
  };
});

import LeftNavigationMenuRadix, { SearchCollapsibleNav } from './LeftNavigationMenuRadix';

const collapsibleResult = (
  overrides: Partial<UseSearchCollapsibleNavResult> = {}
): UseSearchCollapsibleNavResult => ({
  isExpanded: false,
  isParentActive: false,
  parentHref: '#/browse.search.generic',
  navigateToSearch: jest.fn(),
  handleParentClick: jest.fn(),
  customRouteName: 'browse.search.custom',
  visibleFormats: [],
  ...overrides,
});

const menuResult = (overrides: Partial<UseLeftNavigationMenuResult> = {}): UseLeftNavigationMenuResult => ({
  isCollapsed: false,
  isPreviewUI: false,
  adminInitialRouteName: 'admin.system.api',
  isSettingsVisible: true,
  showTestHub: false,
  isProEdition: true,
  version: '3.96.4',
  logo: 'logo.svg',
  canSwitchToClassic: true,
  isDrawer: false,
  isDrawerOpen: false,
  closeDrawer: jest.fn(),
  onDrawerClick: jest.fn(),
  drawerRef: { current: null },
  ...overrides,
});

describe('SearchCollapsibleNav', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockUseSearchCollapsibleNav.mockReturnValue(collapsibleResult());
  });

  it('renders the custom entry and filtered formats when expanded', () => {
    mockUseSearchCollapsibleNav.mockReturnValue(
      collapsibleResult({
        isExpanded: true,
        visibleFormats: [
          { id: 'alpine', label: 'Alpine', routeName: 'browse.search.alpine' },
          { id: 'maven', label: 'Maven', routeName: 'browse.search.maven' },
        ],
      })
    );

    render(<SearchCollapsibleNav isCollapsed={false} />);

    expect(screen.getByText('Custom')).toBeInTheDocument();
    expect(screen.getByText('Alpine')).toBeInTheDocument();
    expect(screen.getByText('Maven')).toBeInTheDocument();
  });

  it('does not render the submenu when collapsed', () => {
    const { container } = render(<SearchCollapsibleNav isCollapsed={false} />);

    expect(container.querySelector('.search-collapsible')).not.toHaveClass('search-collapsible--expanded');
    expect(screen.queryByText('Custom')).not.toBeInTheDocument();
  });

  it('reflects expansion state from the hook', () => {
    mockUseSearchCollapsibleNav.mockReturnValue(collapsibleResult({ isExpanded: true }));

    const { container } = render(<SearchCollapsibleNav isCollapsed={false} />);

    expect(container.querySelector('.search-collapsible')).toHaveClass('search-collapsible--expanded');
  });

  it('renders a minimal link without submenu when collapsed', () => {
    const { container } = render(<SearchCollapsibleNav isCollapsed />);

    expect(container.querySelector('.search-collapsible')).not.toBeInTheDocument();
    expect(container.querySelector('.guide-nav-item')).toBeInTheDocument();
    expect(screen.queryByText('Custom')).not.toBeInTheDocument();
  });
});

describe('LeftNavigationMenuRadix', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockUseSearchCollapsibleNav.mockReturnValue(collapsibleResult());
  });

  it('shouldShowApiNavItemInPreviewUI', () => {
    mockUseLeftNavigationMenu.mockReturnValue(menuResult({ isPreviewUI: true }));

    render(<LeftNavigationMenuRadix />);

    expect(screen.getByTestId('nav-item-preview.browse.api')).toBeInTheDocument();
    expect(screen.getByText('API')).toBeInTheDocument();
  });

  it('shouldNotShowApiNavItemInClassicUI', () => {
    mockUseLeftNavigationMenu.mockReturnValue(menuResult({ isPreviewUI: false }));

    render(<LeftNavigationMenuRadix />);

    expect(screen.queryByTestId('nav-item-preview.browse.api')).not.toBeInTheDocument();
  });

  it('hides the settings entry when not visible', () => {
    mockUseLeftNavigationMenu.mockReturnValue(
      menuResult({ isPreviewUI: true, isSettingsVisible: false, adminInitialRouteName: null })
    );

    render(<LeftNavigationMenuRadix />);

    expect(screen.queryByTestId('nav-item-preview.settings')).not.toBeInTheDocument();
  });

  it('shouldNotShowTagsNavItemInCEMode', () => {
    mockUseLeftNavigationMenu.mockReturnValue(menuResult({ isProEdition: false }));

    render(<LeftNavigationMenuRadix />);

    expect(screen.queryByTestId('nav-item-browse.tags')).not.toBeInTheDocument();
  });

  it('shouldShowTagsNavItemInProEdition', () => {
    mockUseLeftNavigationMenu.mockReturnValue(menuResult({ isProEdition: true }));

    render(<LeftNavigationMenuRadix />);

    expect(screen.getByTestId('nav-item-browse.tags')).toBeInTheDocument();
    expect(screen.getByText('Tags')).toBeInTheDocument();
  });
});

describe('LeftNavigationMenuRadix footer', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockUseSearchCollapsibleNav.mockReturnValue(collapsibleResult());
  });

  it('labels the menu', () => {
    mockUseLeftNavigationMenu.mockReturnValue(menuResult());

    render(<LeftNavigationMenuRadix />);

    expect(screen.getByRole('navigation', { name: 'Main menu' })).toBeInTheDocument();
  });

  it.each([
    ['Astrum One', true],
    ['the Classic UI', false],
  ])('shows the version and no theme choice in %s', (_ui, isPreviewUI) => {
    mockUseLeftNavigationMenu.mockReturnValue(menuResult({ isPreviewUI }));

    render(<LeftNavigationMenuRadix />);

    expect(screen.getByTestId('left-nav-version')).toHaveTextContent('Astrum Repository 3.96.4');
    expect(screen.queryByRole('radiogroup')).not.toBeInTheDocument();
    expect(screen.queryByRole('radio')).not.toBeInTheDocument();
  });

  it('leaves the footer out of the collapsed menu', () => {
    mockUseLeftNavigationMenu.mockReturnValue(menuResult({ isPreviewUI: true, isCollapsed: true }));

    render(<LeftNavigationMenuRadix />);

    expect(screen.queryByTestId('left-nav-footer')).not.toBeInTheDocument();
  });

  it('renders no footer while the version is unknown', () => {
    mockUseLeftNavigationMenu.mockReturnValue(menuResult({ version: null }));

    render(<LeftNavigationMenuRadix />);

    expect(screen.queryByTestId('left-nav-footer')).not.toBeInTheDocument();
  });

  describe('as a drawer on phones and tablets', () => {
    it('stays closed and keeps its rows expanded even when the side menu was collapsed', () => {
      mockUseLeftNavigationMenu.mockReturnValue(menuResult({ isPreviewUI: true, isDrawer: true, isCollapsed: true }));

      render(<LeftNavigationMenuRadix />);

      const nav = screen.getByTestId('left-nav');
      expect(nav).toHaveClass('guide-sidebar--drawer');
      expect(nav).not.toHaveClass('guide-sidebar--drawer-open');
      expect(nav).not.toHaveClass('guide-sidebar--collapsed');
      expect(screen.queryByTestId('left-nav-scrim')).not.toBeInTheDocument();
    });

    it('shows the scrim, a close button and the controls the header has no room for when open', () => {
      const closeDrawer = jest.fn();
      mockUseLeftNavigationMenu.mockReturnValue(
        menuResult({ isPreviewUI: true, isDrawer: true, isDrawerOpen: true, closeDrawer })
      );

      render(<LeftNavigationMenuRadix />);

      expect(screen.getByTestId('left-nav')).toHaveClass('guide-sidebar--drawer-open');
      expect(screen.getByTestId('language-menu')).toBeInTheDocument();
      expect(screen.getByTestId('help-menu')).toBeInTheDocument();
      expect(screen.getByText('Astrum Repository 3.96.4')).toBeInTheDocument();
      expect(screen.queryByTestId('left-nav-footer')).not.toBeInTheDocument();

      fireEvent.click(screen.getByTestId('left-nav-scrim'));
      fireEvent.click(screen.getByRole('button', { name: 'Close menu' }));
      expect(closeDrawer).toHaveBeenCalledTimes(2);
    });

    it('closes after a pick', () => {
      const onDrawerClick = jest.fn();
      mockUseLeftNavigationMenu.mockReturnValue(
        menuResult({ isPreviewUI: true, isDrawer: true, isDrawerOpen: true, onDrawerClick })
      );

      render(<LeftNavigationMenuRadix />);
      fireEvent.click(screen.getByTestId('nav-item-browse.welcome'));

      expect(onDrawerClick).toHaveBeenCalled();
    });

    it('asks the header to switch to the Classic UI and closes', () => {
      const closeDrawer = jest.fn();
      const onSwitch = jest.fn();
      window.addEventListener('as-switch-to-classic', onSwitch);
      mockUseLeftNavigationMenu.mockReturnValue(
        menuResult({ isPreviewUI: true, isDrawer: true, isDrawerOpen: true, closeDrawer })
      );

      render(<LeftNavigationMenuRadix />);
      fireEvent.click(screen.getByTestId('left-nav-switch-classic'));

      expect(closeDrawer).toHaveBeenCalled();
      expect(onSwitch).toHaveBeenCalledTimes(1);
      window.removeEventListener('as-switch-to-classic', onSwitch);
    });

    it('does not offer the Classic UI when the administrator turned it off', () => {
      mockUseLeftNavigationMenu.mockReturnValue(
        menuResult({ isPreviewUI: true, isDrawer: true, isDrawerOpen: true, canSwitchToClassic: false })
      );

      render(<LeftNavigationMenuRadix />);

      expect(screen.queryByTestId('left-nav-switch-classic')).not.toBeInTheDocument();
    });
  });
});
