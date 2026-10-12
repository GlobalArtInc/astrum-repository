
import React from 'react';
import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';

import type { UseLeftNavigationMenuResult } from './useLeftNavigationMenu';

const mockUseLeftNavigationMenu = jest.fn<UseLeftNavigationMenuResult, []>();

// Component tests mock the hook layer: presentation is verified in isolation.
jest.mock('./useLeftNavigationMenu', () => ({
  useLeftNavigationMenu: () => mockUseLeftNavigationMenu(),
}));

// NavItemBox is the shared navigation primitive; stub it to a simple node.
jest.mock('@globalart/astrum-ui-plugin', () => ({
  NavItemBox: ({ name, text }: { name: string; text: string }) => <div data-testid={`nav-item-${name}`}>{text}</div>,
}));

jest.mock('../../routerConfig/routeNames/routeNames', () => ({
  ROUTE_NAMES: {
    BROWSE: {
      WELCOME: { ROOT: 'preview.browse.welcome' },
      SEARCH: {
        ROOT: 'preview.browse.search',
        UNIFIED: 'preview.browse.search.unified',
      },
      BROWSE: { ROOT: 'preview.browse.browse' },
      API: { ROOT: 'preview.browse.api' },
      UPLOAD: { ROOT: 'preview.browse.upload', LIST: 'preview.browse.upload.list' },
      TAGS: { ROOT: 'preview.browse.tags' },
    },
    ADMIN: {
      DIRECTORY: 'preview.admin',
      SETTINGS: 'preview.settings',
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
    FlaskConical: Icon,
  };
});

import LeftNavigationMenuRadix from './LeftNavigationMenuRadix';

const menuResult = (overrides: Partial<UseLeftNavigationMenuResult> = {}): UseLeftNavigationMenuResult => ({
  isCollapsed: false,
  adminInitialRouteName: 'preview.admin.repository',
  isSettingsVisible: true,
  showTestHub: false,
  isProEdition: true,
  version: '3.96.4',
  logo: 'logo.svg',
  isDrawer: false,
  isDrawerOpen: false,
  closeDrawer: jest.fn(),
  onDrawerClick: jest.fn(),
  drawerRef: { current: null },
  ...overrides,
});

describe('LeftNavigationMenuRadix', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('links the pages of Astrum One', () => {
    mockUseLeftNavigationMenu.mockReturnValue(menuResult());

    render(<LeftNavigationMenuRadix />);

    expect(screen.getByTestId('nav-item-preview.browse.welcome')).toBeInTheDocument();
    expect(screen.getByTestId('nav-item-preview.browse.search.unified')).toHaveTextContent('Search');
    expect(screen.getByTestId('nav-item-preview.browse.browse')).toBeInTheDocument();
    expect(screen.getByTestId('nav-item-preview.browse.upload.list')).toBeInTheDocument();
    expect(screen.getByTestId('nav-item-preview.browse.api')).toHaveTextContent('API');
    expect(screen.getByTestId('nav-item-preview.settings')).toBeInTheDocument();
  });

  it('hides the settings entry when not visible', () => {
    mockUseLeftNavigationMenu.mockReturnValue(
      menuResult({ isSettingsVisible: false, adminInitialRouteName: null })
    );

    render(<LeftNavigationMenuRadix />);

    expect(screen.queryByTestId('nav-item-preview.settings')).not.toBeInTheDocument();
  });

  it('shouldNotShowTagsNavItemInCEMode', () => {
    mockUseLeftNavigationMenu.mockReturnValue(menuResult({ isProEdition: false }));

    render(<LeftNavigationMenuRadix />);

    expect(screen.queryByTestId('nav-item-preview.browse.tags')).not.toBeInTheDocument();
  });

  it('shouldShowTagsNavItemInProEdition', () => {
    mockUseLeftNavigationMenu.mockReturnValue(menuResult({ isProEdition: true }));

    render(<LeftNavigationMenuRadix />);

    expect(screen.getByTestId('nav-item-preview.browse.tags')).toBeInTheDocument();
    expect(screen.getByText('Tags')).toBeInTheDocument();
  });
});

describe('LeftNavigationMenuRadix footer', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('labels the menu', () => {
    mockUseLeftNavigationMenu.mockReturnValue(menuResult());

    render(<LeftNavigationMenuRadix />);

    expect(screen.getByRole('navigation', { name: 'Main menu' })).toBeInTheDocument();
  });

  it('shows the version and no theme choice', () => {
    mockUseLeftNavigationMenu.mockReturnValue(menuResult());

    render(<LeftNavigationMenuRadix />);

    expect(screen.getByTestId('left-nav-version')).toHaveTextContent('Astrum Repository 3.96.4');
    expect(screen.queryByRole('radiogroup')).not.toBeInTheDocument();
    expect(screen.queryByRole('radio')).not.toBeInTheDocument();
  });

  it('leaves the footer out of the collapsed menu', () => {
    mockUseLeftNavigationMenu.mockReturnValue(menuResult({ isCollapsed: true }));

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
      mockUseLeftNavigationMenu.mockReturnValue(menuResult({ isDrawer: true, isCollapsed: true }));

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
        menuResult({ isDrawer: true, isDrawerOpen: true, closeDrawer })
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
        menuResult({ isDrawer: true, isDrawerOpen: true, onDrawerClick })
      );

      render(<LeftNavigationMenuRadix />);
      fireEvent.click(screen.getByTestId('nav-item-preview.browse.welcome'));

      expect(onDrawerClick).toHaveBeenCalled();
    });
  });
});
