

import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';
import { Theme } from '@radix-ui/themes';

import { BrowsePage } from '../BrowsePage';
import { ExtJS } from '../../../../../interface/ExtJS';

// Wrapper with Radix UI providers
const renderWithProviders = (ui) => {
  return render(
    <Theme>
      {ui}
    </Theme>
  );
};

// Mock router hooks
const mockGo = jest.fn();
const mockRouter = {
  stateService: {
    go: mockGo,
  },
};

let mockParams = {};

jest.mock('@uirouter/react', () => ({
  useRouter: () => mockRouter,
  useCurrentStateAndParams: () => ({
    params: mockParams,
  }),
}));

// Mock axios
jest.mock('axios', () => ({
  get: jest.fn().mockResolvedValue({ data: { data: [] } }),
}));

// Mock the repository info of the page header and the usage snippets
const mockRepositoryInfo = {
  name: 'maven-central',
  format: 'maven2',
  type: 'proxy',
  url: 'http://localhost/repository/maven-central',
  status: { online: true },
};

jest.mock('../useRepositoryInfo', () => ({
  ...jest.requireActual('../useRepositoryInfo'),
  useRepositoryInfo: (name) => ({
    repository: name ? { ...mockRepositoryInfo, name } : null,
    loading: false,
  }),
}));

// Mock repository list hooks
jest.mock('../repository-list/useRepositoryList', () => ({
  isIqServerEnabled: () => false,
  useRepositoryList: () => ({
    state: { loading: false, error: null, healthCheck: {}, firewallStatus: {} },
    showHealthCheckColumn: false,
    showIqPolicyViolationsColumn: false,
  }),
}));

jest.mock('../repository-list/useRepositoryListServer', () => ({
  useRepositoryListServer: () => ({
    repositories: [],
    loading: false,
    error: null,
    filterParams: {},
    setFilterParams: jest.fn(),
    totalCount: 0,
    page: 1,
    totalPages: 1,
    nextPage: jest.fn(),
    previousPage: jest.fn(),
  }),
  arrayToFilterString: jest.fn((arr) => arr?.join(',') || undefined),
}));

// Mock child components
jest.mock('../repository-list/RepositoryListTable', () => ({
  RepositoryListTable: ({ onSelect }) => (
    <div data-testid="mock-repository-list">
      <button
        data-testid="repo-maven-central"
        onClick={() => onSelect && onSelect('maven-central')}
      >
        maven-central
      </button>
      <button
        data-testid="repo-npm-hosted"
        onClick={() => onSelect && onSelect('npm-hosted')}
      >
        npm-hosted
      </button>
    </div>
  ),
}));

// Mock browse API
jest.mock('../browse.api', () => ({
  fetchAsset: jest.fn().mockResolvedValue({
    id: 'asset-123',
    name: 'test-asset',
    format: 'maven2',
    contentType: 'application/xml',
    size: 1024,
    repositoryName: 'maven-central',
    path: '/org/apache/maven/pom.xml',
    downloadUrl: 'https://server.example.com/custom-context/repository/maven-central/org/apache/maven/pom.xml',
  }),
  fetchComponent: jest.fn().mockResolvedValue({
    id: 'comp-123',
    name: 'maven',
    format: 'maven2',
    repositoryName: 'maven-central',
    group: 'org.apache',
    version: '1.0.0',
  }),
}));

// Mock shared components
const mockToast = {
  success: jest.fn(),
  error: jest.fn(),
  info: jest.fn(),
  warning: jest.fn(),
};

jest.mock('../../../shared', () => {
  const actual = jest.requireActual('../../../shared');
  return {
    ...actual,
    FilterSidebar: () => <div data-testid="mock-filter-sidebar">Filters</div>,
    PageHeader: ({ title, description }) => (
      <div data-testid="mock-page-header">
        <h2>{title}</h2>
        <p>{description}</p>
      </div>
    ),
    useToast: () => mockToast,
  };
});

jest.mock('../../../shared/MobileFilterDrawer/MobileFilterDrawer', () => ({
  MobileFilterDrawer: ({ children, isOpen }) =>
    isOpen ? <div data-testid="mock-mobile-filter-drawer">{children}</div> : null,
}));

jest.mock('../tree/BrowseTree', () => ({
  BrowseTree: ({ repositoryName, initialPath, onSelect }) => (
    <div data-testid="mock-browse-tree">
      <span data-testid="tree-repo-name">{repositoryName}</span>
      {initialPath && <span data-testid="tree-initial-path">{initialPath}</span>}
      <button
        data-testid="tree-node-component"
        onClick={() =>
          onSelect && onSelect({
            id: 'org/apache/maven',
            text: 'maven',
            type: 'component',
            leaf: false,
            componentId: 'comp-123',
          })
        }
      >
        Component Node
      </button>
      <button
        data-testid="tree-node-asset"
        onClick={() =>
          onSelect && onSelect({
            id: 'org/apache/maven/pom.xml',
            text: 'pom.xml',
            type: 'asset',
            leaf: true,
            assetId: 'asset-456',
          })
        }
      >
        Asset Node
      </button>
      <button
        data-testid="tree-node-folder"
        onClick={() =>
          onSelect && onSelect({
            id: 'org/apache',
            text: 'apache',
            type: 'folder',
            leaf: false,
          })
        }
      >
        Folder Node
      </button>
    </div>
  ),
}));

jest.mock('../detail/DetailPanel', () => ({
  DetailPanel: ({ node, repositoryName, onDeleted }) => (
    <div data-testid="mock-detail-panel">
      {node ? (
        <>
          <span data-testid="detail-node-text">{node.text}</span>
          <span data-testid="detail-node-type">{node.type}</span>
          <span data-testid="detail-repo-name">{repositoryName}</span>
          <button data-testid="detail-delete-btn" onClick={onDeleted}>
            Delete
          </button>
        </>
      ) : (
        <span data-testid="detail-empty">Select an item</span>
      )}
    </div>
  ),
}));

describe('BrowsePage', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockParams = {};
  });

  describe('Layout', () => {
    it('renders list view by default (Step 1)', () => {
      renderWithProviders(<BrowsePage />);

      expect(screen.getByTestId('browse-page')).toBeInTheDocument();
      expect(screen.getByTestId('mock-repository-list')).toBeInTheDocument();
      // Detail panel is only shown in tree view (Step 2)
      expect(screen.queryByTestId('mock-detail-panel')).not.toBeInTheDocument();
    });

    it('renders tree view with detail panel when URL has repo (Step 2)', () => {
      mockParams = { repoName: 'maven-central' };
      renderWithProviders(<BrowsePage />);

      expect(screen.getByTestId('browse-page')).toBeInTheDocument();
      expect(screen.getByTestId('mock-browse-tree')).toBeInTheDocument();
      expect(screen.getByTestId('mock-detail-panel')).toBeInTheDocument();
      expect(screen.queryByTestId('mock-repository-list')).not.toBeInTheDocument();
    });

    it('renders page header with title and count', () => {
      renderWithProviders(<BrowsePage />);

      const headings = screen.getAllByRole('heading', { name: /repositories/i });
      expect(headings.length).toBeGreaterThan(0);
    });
  });

  describe('Repository selection', () => {
    it('shows repository list in initial view', () => {
      renderWithProviders(<BrowsePage />);

      expect(screen.getByTestId('mock-repository-list')).toBeInTheDocument();
      // Tree should not be visible until a repo is selected
      expect(screen.queryByTestId('mock-browse-tree')).not.toBeInTheDocument();
    });

    it('navigates to repository browse in same tab when repo clicked', async () => {
      mockGo.mockClear();
      renderWithProviders(<BrowsePage />);

      await userEvent.click(screen.getByTestId('repo-maven-central'));

      expect(mockGo).toHaveBeenCalledWith('preview.browse.browse.repo', {
        repoName: 'maven-central',
      });
    });

    it('keeps repository list visible when repo clicked (same-tab navigation)', async () => {
      mockGo.mockClear();
      renderWithProviders(<BrowsePage />);

      await userEvent.click(screen.getByTestId('repo-maven-central'));

      expect(screen.getByTestId('mock-repository-list')).toBeInTheDocument();
    });
  });

  describe('Tree interaction', () => {
    beforeEach(() => {
      mockParams = { repoName: 'maven-central' };
    });

    it('shows tree panel when navigating with repo in URL', async () => {
      renderWithProviders(<BrowsePage />);

      expect(screen.getByTestId('mock-browse-tree')).toBeInTheDocument();
      expect(screen.getByTestId('tree-repo-name')).toHaveTextContent('maven-central');
    });

    it('updates selected node on tree item click', async () => {
      renderWithProviders(<BrowsePage />);

      await userEvent.click(screen.getByTestId('tree-node-component'));

      expect(screen.getByTestId('detail-node-text')).toHaveTextContent('maven');
      expect(screen.getByTestId('detail-node-type')).toHaveTextContent('component');
    });

  });

  describe('Detail panel', () => {
    beforeEach(() => {
      mockParams = { repoName: 'maven-central' };
    });

    it('shows empty state when no node selected (after navigating with repo in URL)', () => {
      renderWithProviders(<BrowsePage />);

      expect(screen.getByTestId('detail-empty')).toBeInTheDocument();
    });

    it('shows detail panel when node selected', async () => {
      renderWithProviders(<BrowsePage />);

      await userEvent.click(screen.getByTestId('tree-node-component'));

      expect(screen.getByTestId('detail-node-text')).toBeInTheDocument();
      expect(screen.queryByTestId('detail-empty')).not.toBeInTheDocument();
    });

    it('shows correct node type in detail panel', async () => {
      renderWithProviders(<BrowsePage />);

      await userEvent.click(screen.getByTestId('tree-node-component'));
      expect(screen.getByTestId('detail-node-type')).toHaveTextContent('component');

      await userEvent.click(screen.getByTestId('tree-node-asset'));
      expect(screen.getByTestId('detail-node-type')).toHaveTextContent('asset');

      await userEvent.click(screen.getByTestId('tree-node-folder'));
      expect(screen.getByTestId('detail-node-type')).toHaveTextContent('folder');
    });
  });

  describe('Deep linking', () => {
    it('restores repository from URL params', () => {
      mockParams = { repoName: 'maven-central' };

      renderWithProviders(<BrowsePage />);

      // Tree should be visible with the repository from URL
      expect(screen.getByTestId('mock-browse-tree')).toBeInTheDocument();
      expect(screen.getByTestId('tree-repo-name')).toHaveTextContent('maven-central');
    });

    it('restores path from URL params', () => {
      mockParams = {
        repoName: 'maven-central',
        path: encodeURIComponent('org/apache/maven'),
      };

      renderWithProviders(<BrowsePage />);

      // Tree should receive the initial path
      expect(screen.getByTestId('tree-initial-path')).toHaveTextContent('org/apache/maven');
    });

    it('expands tree to path on load (via initialPath prop)', () => {
      mockParams = {
        repoName: 'maven-central',
        path: encodeURIComponent('org/apache'),
      };

      renderWithProviders(<BrowsePage />);

      // The BrowseTree mock receives initialPath
      expect(screen.getByTestId('tree-initial-path')).toHaveTextContent('org/apache');
    });
  });

  describe('Delete handling', () => {
    beforeEach(() => {
      mockParams = { repoName: 'maven-central' };
    });

    it('clears selection after delete', async () => {
      renderWithProviders(<BrowsePage />);

      await userEvent.click(screen.getByTestId('tree-node-component'));
      expect(screen.getByTestId('detail-node-text')).toHaveTextContent('maven');

      await userEvent.click(screen.getByTestId('detail-delete-btn'));
      expect(screen.getByTestId('detail-empty')).toBeInTheDocument();
    });
  });

  describe('Copy URL', () => {
    beforeEach(() => {
      Object.defineProperty(navigator, 'clipboard', {
        value: {
          writeText: jest.fn().mockResolvedValue(undefined),
        },
        configurable: true,
      });
    });

    it('copies the repository URL to the clipboard and shows a success toast', async () => {
      mockParams = { repoName: 'maven-central' };
      renderWithProviders(<BrowsePage />);

      const copyBtn = screen.getByRole('button', { name: /copy url/i });
      await userEvent.click(copyBtn);

      expect(navigator.clipboard.writeText).toHaveBeenCalledWith('http://localhost/repository/maven-central/');
      await waitFor(() => expect(mockToast.success).toHaveBeenCalledWith('Repository URL copied'));
    });
  });

  describe('Repository header', () => {
    it('shows the format, type and status of the repository', () => {
      mockParams = { repoName: 'maven-central' };
      renderWithProviders(<BrowsePage />);

      expect(screen.getByRole('heading', { level: 1, name: 'maven-central' })).toBeInTheDocument();
      expect(screen.getByTestId('format-badge')).toHaveTextContent('maven');
      expect(screen.getByText('Proxy')).toBeInTheDocument();
      expect(screen.getByTestId('status-label')).toHaveTextContent('Online');
    });

    it('hides the repository settings without permission', () => {
      const check = jest.spyOn(ExtJS, 'checkPermission').mockReturnValue(false);
      mockParams = { repoName: 'maven-central' };
      renderWithProviders(<BrowsePage />);

      expect(screen.queryByRole('button', { name: /repository settings/i })).not.toBeInTheDocument();
      check.mockRestore();
    });

    it('opens the repository settings with a grant on this repository', async () => {
      const check = jest
        .spyOn(ExtJS, 'checkPermission')
        .mockImplementation((permission) => permission === 'astrum:repository-admin:maven2:maven-central:read');
      mockParams = { repoName: 'maven-central' };
      renderWithProviders(<BrowsePage />);

      await userEvent.click(screen.getByRole('button', { name: /repository settings/i }));

      expect(mockGo).toHaveBeenCalledWith('preview.admin.repository.repositories.detail', {
        repositoryName: 'maven-central',
      });
      check.mockRestore();
    });
  });

  describe('Multiple repositories', () => {
    it('switches between repositories via the Repositories breadcrumb', async () => {
      mockParams = { repoName: 'maven-central' };
      renderWithProviders(<BrowsePage />);

      expect(screen.getByTestId('tree-repo-name')).toHaveTextContent('maven-central');

      await userEvent.click(screen.getByRole('button', { name: /^repositories$/i }));

      // Should be back on list view
      expect(screen.getByTestId('mock-repository-list')).toBeInTheDocument();
    });
  });

  describe('Repository name in detail panel', () => {
    it('passes correct repository name to detail panel', async () => {
      mockParams = { repoName: 'maven-central' };
      renderWithProviders(<BrowsePage />);

      await userEvent.click(screen.getByTestId('tree-node-component'));
      expect(screen.getByTestId('detail-repo-name')).toHaveTextContent('maven-central');
    });
  });

  describe('URL-based navigation', () => {
    beforeEach(() => {
      mockParams = { repoName: 'maven-central' };
      mockGo.mockClear();
    });

    it('updates URL with path when selecting a tree node', async () => {
      renderWithProviders(<BrowsePage />);

      await userEvent.click(screen.getByTestId('tree-node-component'));

      expect(mockGo).toHaveBeenCalledWith('preview.browse.browse.repo.path', {
        repoName: 'maven-central',
        path: 'org/apache/maven',
        tab: 'summary',
      });
    });

    it('updates URL with path when selecting an asset node', async () => {
      renderWithProviders(<BrowsePage />);

      await userEvent.click(screen.getByTestId('tree-node-asset'));

      expect(mockGo).toHaveBeenCalledWith('preview.browse.browse.repo.path', {
        repoName: 'maven-central',
        path: 'org/apache/maven/pom.xml',
        tab: 'summary',
      });
    });

    it('updates URL with path when selecting a folder node', async () => {
      renderWithProviders(<BrowsePage />);

      await userEvent.click(screen.getByTestId('tree-node-folder'));

      expect(mockGo).toHaveBeenCalledWith('preview.browse.browse.repo.path', {
        repoName: 'maven-central',
        path: 'org/apache',
        tab: 'summary',
      });
    });

    it('preserves existing tab param when selecting a node', async () => {
      mockParams = { repoName: 'maven-central', tab: 'attributes' };
      renderWithProviders(<BrowsePage />);

      await userEvent.click(screen.getByTestId('tree-node-component'));

      expect(mockGo).toHaveBeenCalledWith('preview.browse.browse.repo.path', {
        repoName: 'maven-central',
        path: 'org/apache/maven',
        tab: 'attributes',
      });
    });
  });
});
