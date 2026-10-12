

import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';
import { Theme } from '@radix-ui/themes';

import { DetailPanel, type AssetData, type ComponentData } from '../DetailPanel';
import type { BrowseNode } from '../../tree/browse-tree.types';
import { deleteAsset, deleteComponent } from '../../browse.api';

const mockFetchBrowseNodes = jest.fn();

const mockAxiosGet = jest.fn();
jest.mock('axios', () => ({
  __esModule: true,
  default: { get: (...args: unknown[]) => mockAxiosGet(...args) },
}));

const mockTrackSnippetCopy = jest.fn();
jest.mock('../../../search/details/dependencySnippets/trackSnippetCopy', () => ({
  trackSnippetCopy: (...args: unknown[]) => mockTrackSnippetCopy(...args),
}));

jest.mock('../../browse.api', () => ({
  deleteComponent: jest.fn().mockResolvedValue(['mock-id']),
  deleteAsset: jest.fn().mockResolvedValue(undefined),
  fetchBrowseNodes: (...args: unknown[]) => mockFetchBrowseNodes(...args),
}));

const mockToast = {
  showToast: jest.fn(),
  success: jest.fn(),
  error: jest.fn(),
  warning: jest.fn(),
  info: jest.fn(),
};

// Mock useToast from shared
jest.mock('../../../../shared', () => ({
  useToast: () => mockToast,
  DeepResearchLink: ({ href }: { href: string }) => <a href={href}>Deep Research</a>,
}));

// Test wrapper with Radix Theme
const renderWithTheme = (ui: React.ReactElement) => {
  return render(<Theme>{ui}</Theme>);
};

// Mock BrowseNode data
const mockComponentNode: BrowseNode = {
  id: '/org/apache/commons/commons-lang3/3.14.0',
  text: 'commons-lang3-3.14.0',
  type: 'component',
  leaf: false,
  componentId: 'component-123',
};

const mockAssetNode: BrowseNode = {
  id: '/org/apache/commons/commons-lang3/3.14.0/commons-lang3-3.14.0.jar',
  text: 'commons-lang3-3.14.0.jar',
  type: 'asset',
  leaf: true,
  componentId: 'component-123',
  assetId: 'asset-123',
};

const mockFolderNode: BrowseNode = {
  id: '/org/apache/commons',
  text: 'commons',
  type: 'folder',
  leaf: false,
};

// Mock component data
const mockComponentData: ComponentData = {
  id: 'component-123',
  repositoryName: 'maven-releases',
  format: 'maven2',
  group: 'org.apache.commons',
  name: 'commons-lang3',
  version: '3.14.0',
};

// Mock asset data
const mockAssetData: AssetData = {
  id: 'asset-123',
  name: '/org/apache/commons/commons-lang3/3.14.0/commons-lang3-3.14.0.jar',
  format: 'maven2',
  contentType: 'application/java-archive',
  size: 1572864,
  repositoryName: 'maven-releases',
  blobCreated: '2024-01-15T10:30:00Z',
  blobUpdated: '2024-01-15T10:30:00Z',
  lastDownloaded: '2024-01-20T14:25:00Z',
  // Server-provided absolute URL — includes context path regardless of deployment
  downloadUrl: 'https://server.example.com/custom-context/repository/maven-releases/org/apache/commons/commons-lang3/3.14.0/commons-lang3-3.14.0.jar',
};

const repositoryName = 'maven-releases';

describe('DetailPanel', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    (window as any).__astrumToast = mockToast;
    mockAxiosGet.mockResolvedValue({ data: { manifests: [] } });
    mockFetchBrowseNodes.mockResolvedValue([
      { id: 'child1', text: 'child1', type: 'folder', leaf: false },
      { id: 'child2', text: 'child2', type: 'component', leaf: false },
      { id: 'child3', text: 'child3.jar', type: 'asset', leaf: true },
    ]);
  });

  afterEach(() => {
    delete (window as any).__astrumToast;
  });

  describe('empty state', () => {
    it('shows empty state when node is null', () => {
      renderWithTheme(<DetailPanel node={null} repositoryName={repositoryName} />);

      expect(screen.getByText('Nothing selected')).toBeInTheDocument();
      expect(screen.getByText(/Pick an item in the tree/)).toBeInTheDocument();
    });
  });

  describe('loading state', () => {
    it('shows loading spinner when loading is true', () => {
      renderWithTheme(
        <DetailPanel node={mockComponentNode} repositoryName={repositoryName} loading={true} />
      );

      expect(screen.getByText('Loading details…')).toBeInTheDocument();
    });
  });

  describe('error state', () => {
    it('shows error message when error is provided', () => {
      renderWithTheme(
        <DetailPanel
          node={mockComponentNode}
          repositoryName={repositoryName}
          error="Failed to load"
        />
      );

      expect(screen.getByText('Could not load the details: Failed to load')).toBeInTheDocument();
    });
  });

  describe('component details', () => {
    it('renders component details for component node', () => {
      renderWithTheme(
        <DetailPanel
          node={mockComponentNode}
          repositoryName={repositoryName}
          componentData={mockComponentData}
        />
      );

      expect(screen.getByRole('heading', { name: /^commons-lang3\s?:3\.14\.0$/ })).toBeInTheDocument();
    });

    it('shows component repository', () => {
      renderWithTheme(
        <DetailPanel
          node={mockComponentNode}
          repositoryName={repositoryName}
          componentData={mockComponentData}
        />
      );

      expect(screen.getByText(repositoryName)).toBeInTheDocument();
    });

    it('shows component format', () => {
      renderWithTheme(
        <DetailPanel
          node={mockComponentNode}
          repositoryName={repositoryName}
          componentData={mockComponentData}
        />
      );

      expect(screen.getByText('Maven', { selector: 'dd' })).toBeInTheDocument();
    });

    it('shows delete button when canDelete is true', () => {
      renderWithTheme(
        <DetailPanel
          node={mockComponentNode}
          repositoryName={repositoryName}
          componentData={mockComponentData}
          canDelete={true}
        />
      );

      expect(screen.getByRole('button', { name: /delete component/i })).toBeInTheDocument();
    });

    it('hides delete button when canDelete is false', () => {
      renderWithTheme(
        <DetailPanel
          node={mockComponentNode}
          repositoryName={repositoryName}
          componentData={mockComponentData}
          canDelete={false}
        />
      );

      expect(screen.queryByRole('button', { name: /delete component/i })).not.toBeInTheDocument();
    });
  });

  describe('asset details', () => {
    it('renders asset details for asset node', () => {
      renderWithTheme(
        <DetailPanel
          node={mockAssetNode}
          repositoryName={repositoryName}
          assetData={mockAssetData}
        />
      );

      expect(screen.getByRole('heading', { name: mockAssetNode.text })).toBeInTheDocument();
    });

    it('shows asset format', () => {
      renderWithTheme(
        <DetailPanel
          node={mockAssetNode}
          repositoryName={repositoryName}
          assetData={mockAssetData}
        />
      );

      expect(screen.getByText('Maven', { selector: 'dd' })).toBeInTheDocument();
    });

    it('shows asset content type', () => {
      renderWithTheme(
        <DetailPanel
          node={mockAssetNode}
          repositoryName={repositoryName}
          assetData={mockAssetData}
        />
      );

      expect(screen.getByText('application/java-archive')).toBeInTheDocument();
    });

    it('shows download button for assets', () => {
      renderWithTheme(
        <DetailPanel
          node={mockAssetNode}
          repositoryName={repositoryName}
          assetData={mockAssetData}
        />
      );

      expect(screen.getByRole('button', { name: /download/i })).toBeInTheDocument();
    });

    it('shows delete button when canDelete is true', () => {
      renderWithTheme(
        <DetailPanel
          node={mockAssetNode}
          repositoryName={repositoryName}
          assetData={mockAssetData}
          canDelete={true}
        />
      );

      expect(screen.getByRole('button', { name: /delete file/i })).toBeInTheDocument();
    });
  });

  describe('download counts', () => {
    it('shows how often an asset was downloaded', () => {
      renderWithTheme(
        <DetailPanel
          node={mockAssetNode}
          repositoryName={repositoryName}
          assetData={{ ...mockAssetData, downloadCount: 1234 }}
        />
      );

      expect(screen.getByText('Downloads')).toBeInTheDocument();
      expect(screen.getByText((1234).toLocaleString())).toBeInTheDocument();
    });

    it('shows zero for an asset that was never downloaded', () => {
      renderWithTheme(
        <DetailPanel
          node={mockAssetNode}
          repositoryName={repositoryName}
          assetData={{ ...mockAssetData, downloadCount: 0 }}
        />
      );

      const label = screen.getByText('Downloads');
      expect(label.parentElement).toHaveTextContent('Downloads0');
    });

    it('shows a dash when the downloads of an asset are not known', () => {
      renderWithTheme(
        <DetailPanel node={mockAssetNode} repositoryName={repositoryName} assetData={mockAssetData} />
      );

      expect(screen.getByText('Downloads').parentElement).toHaveTextContent('Downloads—');
    });

    it('shows the downloads of all assets of a component', () => {
      renderWithTheme(
        <DetailPanel
          node={mockComponentNode}
          repositoryName={repositoryName}
          componentData={{ ...mockComponentData, downloadCount: 42 }}
        />
      );

      expect(screen.getByText('Downloads').parentElement).toHaveTextContent('Downloads42');
    });

    it('shows the downloads of a package root asset', () => {
      renderWithTheme(
        <DetailPanel
          node={{ ...mockAssetNode, leaf: false }}
          repositoryName={repositoryName}
          assetData={{ ...mockAssetData, downloadCount: 5 }}
        />
      );

      expect(screen.getByText('Downloads').parentElement).toHaveTextContent('Downloads5');
    });
  });

  describe('folder details', () => {
    it('renders folder details for folder node', () => {
      renderWithTheme(
        <DetailPanel node={mockFolderNode} repositoryName={repositoryName} />
      );

      expect(screen.getByRole('heading', { name: mockFolderNode.text })).toBeInTheDocument();
    });

    it('shows folder path', () => {
      renderWithTheme(
        <DetailPanel node={mockFolderNode} repositoryName={repositoryName} />
      );

      expect(screen.getAllByText(mockFolderNode.id).length).toBeGreaterThan(0);
    });

    it('shows repository name', () => {
      renderWithTheme(
        <DetailPanel node={mockFolderNode} repositoryName={repositoryName} />
      );

      expect(screen.getByText(repositoryName)).toBeInTheDocument();
    });

    it('shows delete button when canDelete is true', () => {
      renderWithTheme(
        <DetailPanel node={mockFolderNode} repositoryName={repositoryName} canDelete={true} />
      );

      expect(screen.getByRole('button', { name: /delete folder/i })).toBeInTheDocument();
    });

    it('shows delete warning when canDelete is true', () => {
      renderWithTheme(
        <DetailPanel node={mockFolderNode} repositoryName={repositoryName} canDelete={true} />
      );

      expect(screen.getByText(/deleting a folder deletes everything in it/i)).toBeInTheDocument();
    });

    it('fetches and displays children count (bug 7mv6)', async () => {
      renderWithTheme(
        <DetailPanel node={mockFolderNode} repositoryName={repositoryName} />
      );

      await waitFor(() => {
        expect(screen.getByText('3')).toBeInTheDocument();
      });

      expect(mockFetchBrowseNodes).toHaveBeenCalledWith({
        repositoryName,
        node: mockFolderNode.id,
      });
    });

    it('shows folder depth', () => {
      renderWithTheme(
        <DetailPanel node={mockFolderNode} repositoryName={repositoryName} />
      );

      expect(screen.getByText('Depth')).toBeInTheDocument();
    });

    it('shows children type breakdown', async () => {
      renderWithTheme(
        <DetailPanel node={mockFolderNode} repositoryName={repositoryName} />
      );

      await waitFor(() => {
        expect(screen.getByText('Items')).toBeInTheDocument();
        expect(screen.getByText('Folders')).toBeInTheDocument();
        expect(screen.getByText('Components')).toBeInTheDocument();
        expect(screen.getByText('Files')).toBeInTheDocument();
      });
    });
  });

  describe('delete functionality', () => {
    it('opens delete dialog when delete button is clicked', async () => {
      renderWithTheme(
        <DetailPanel
          node={mockComponentNode}
          repositoryName={repositoryName}
          componentData={mockComponentData}
          canDelete={true}
        />
      );

      const deleteButton = screen.getByRole('button', { name: /delete component/i });
      await userEvent.click(deleteButton);

      // Dialog should be open
      expect(screen.getByRole('alertdialog')).toBeInTheDocument();
    });

    it('calls onDeleted when delete is confirmed', async () => {
      const onDeleted = jest.fn();
      renderWithTheme(
        <DetailPanel
          node={mockComponentNode}
          repositoryName={repositoryName}
          componentData={mockComponentData}
          canDelete={true}
          onDeleted={onDeleted}
        />
      );

      // Open dialog
      const deleteButton = screen.getByRole('button', { name: /delete component/i });
      await userEvent.click(deleteButton);

      // Confirm delete
      const confirmButton = screen.getByRole('button', { name: /^Delete$/i });
      await userEvent.click(confirmButton);

      // Wait for the simulated delete operation
      await waitFor(() => {
        expect(onDeleted).toHaveBeenCalled();
      });
    });

    it('closes dialog when cancel is clicked', async () => {
      renderWithTheme(
        <DetailPanel
          node={mockComponentNode}
          repositoryName={repositoryName}
          componentData={mockComponentData}
          canDelete={true}
        />
      );

      // Open dialog
      const deleteButton = screen.getByRole('button', { name: /delete component/i });
      await userEvent.click(deleteButton);

      // Cancel
      const cancelButton = screen.getByRole('button', { name: /cancel/i });
      await userEvent.click(cancelButton);

      // Dialog should be closed
      await waitFor(() => {
        expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
      });
    });

    it('closes dialog, shows error toast, and refreshes tree when asset delete fails (cloud 500)', async () => {
      const onDeleted = jest.fn();
      (deleteAsset as jest.Mock).mockRejectedValueOnce(
        new Error('MissingFacetException: No facet of type AptHostedMetadataFacet attached to repository apt-proxy-debian')
      );

      renderWithTheme(
        <DetailPanel
          node={mockAssetNode}
          repositoryName={repositoryName}
          assetData={mockAssetData}
          canDelete={true}
          onDeleted={onDeleted}
        />
      );

      const deleteButton = screen.getByRole('button', { name: /delete file/i });
      await userEvent.click(deleteButton);
      const confirmButton = screen.getByRole('button', { name: /^Delete$/i });
      await userEvent.click(confirmButton);

      await waitFor(() => {
        expect(mockToast.error).toHaveBeenCalledWith(
          expect.stringContaining('MissingFacetException')
        );
      });

      expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();

      await waitFor(() => {
        expect(
          screen.getByText(/Could not delete: MissingFacetException/)
        ).toBeInTheDocument();
      });

      expect(onDeleted).toHaveBeenCalled();
    });

    it('shows success toast when delete succeeds', async () => {
      const onDeleted = jest.fn();
      (deleteAsset as jest.Mock).mockResolvedValueOnce(undefined);

      renderWithTheme(
        <DetailPanel
          node={mockAssetNode}
          repositoryName={repositoryName}
          assetData={mockAssetData}
          canDelete={true}
          onDeleted={onDeleted}
        />
      );

      const deleteButton = screen.getByRole('button', { name: /delete file/i });
      await userEvent.click(deleteButton);
      const confirmButton = screen.getByRole('button', { name: /^Delete$/i });
      await userEvent.click(confirmButton);

      await waitFor(() => {
        expect(mockToast.success).toHaveBeenCalledWith('Deleted');
      });

      expect(onDeleted).toHaveBeenCalled();
    });

    it('deletes ONLY the asset (never the component) for an asset node, even when componentData is loaded', async () => {
      // Regression (ASTRUM-54172): the asset detail panel also receives componentData (to show component
      // info). The confirm handler must key off node.type and call deleteAsset — NOT fall through to the
      // componentData branch and cascade-delete the whole component. For multi-asset components (e.g. a
      // Terraform provider version = one zip per OS/arch) that wrongly removed every sibling platform.
      const onDeleted = jest.fn();
      (deleteAsset as jest.Mock).mockResolvedValueOnce(undefined);

      renderWithTheme(
        <DetailPanel
          node={mockAssetNode}
          repositoryName={repositoryName}
          assetData={mockAssetData}
          componentData={mockComponentData}
          canDelete={true}
          onDeleted={onDeleted}
        />
      );

      const deleteButton = screen.getByRole('button', { name: /delete file/i });
      await userEvent.click(deleteButton);
      const confirmButton = screen.getByRole('button', { name: /^Delete$/i });
      await userEvent.click(confirmButton);

      await waitFor(() => {
        expect(deleteAsset).toHaveBeenCalledWith(mockAssetData.id, repositoryName);
      });
      expect(deleteComponent).not.toHaveBeenCalled();
    });

    it('shows error toast and refreshes tree when component delete fails', async () => {
      const onDeleted = jest.fn();
      (deleteComponent as jest.Mock).mockRejectedValueOnce(
        new Error('Server error (500). Please try again later.')
      );

      renderWithTheme(
        <DetailPanel
          node={mockComponentNode}
          repositoryName={repositoryName}
          componentData={mockComponentData}
          canDelete={true}
          onDeleted={onDeleted}
        />
      );

      const deleteButton = screen.getByRole('button', { name: /delete component/i });
      await userEvent.click(deleteButton);
      const confirmButton = screen.getByRole('button', { name: /^Delete$/i });
      await userEvent.click(confirmButton);

      await waitFor(() => {
        expect(mockToast.error).toHaveBeenCalledWith(
          expect.stringContaining('Server error')
        );
      });

      expect(onDeleted).toHaveBeenCalled();
    });

    it('handles delete timeout by racing with a timeout promise', async () => {
      const onDeleted = jest.fn();
      const neverResolves = new Promise<void>(() => {});
      (deleteAsset as jest.Mock).mockReturnValueOnce(neverResolves);

      renderWithTheme(
        <DetailPanel
          node={mockAssetNode}
          repositoryName={repositoryName}
          assetData={mockAssetData}
          canDelete={true}
          onDeleted={onDeleted}
        />
      );

      const deleteButton = screen.getByRole('button', { name: /delete file/i });
      await userEvent.click(deleteButton);

      expect(screen.getByRole('alertdialog')).toBeInTheDocument();
    });
  });

  describe('download functionality', () => {
    const mockWindowOpen = jest.fn();
    const originalWindowOpen = window.open;

    beforeEach(() => {
      window.open = mockWindowOpen;
      mockWindowOpen.mockClear();
    });

    afterEach(() => {
      window.open = originalWindowOpen;
    });

    it.each([
      ['root context path (/)',    'https://astrum.example.com/repository/maven-releases/org/apache/commons/commons-lang3/3.14.0/commons-lang3-3.14.0.jar'],
      ['custom context path (/astrum)', 'https://astrum.example.com/astrum/repository/maven-releases/org/apache/commons/commons-lang3/3.14.0/commons-lang3-3.14.0.jar'],
      ['custom context path (/astrum)',  'https://astrum.example.com/astrum/repository/maven-releases/org/apache/commons/commons-lang3/3.14.0/commons-lang3-3.14.0.jar'],
    ])('uses server-provided downloadUrl unchanged for %s', async (_label, serverDownloadUrl) => {
      const assetData = { ...mockAssetData, downloadUrl: serverDownloadUrl };
      renderWithTheme(
        <DetailPanel
          node={mockAssetNode}
          repositoryName={repositoryName}
          assetData={assetData}
        />
      );

      const downloadButton = screen.getByRole('button', { name: /download/i });
      await userEvent.click(downloadButton);

      expect(mockWindowOpen).toHaveBeenCalledWith(serverDownloadUrl, '_blank');
    });

    it('falls back to client-generated URL when downloadUrl is absent', async () => {
      const { downloadUrl: _omitted, ...assetDataWithoutDownloadUrl } = mockAssetData;
      renderWithTheme(
        <DetailPanel
          node={mockAssetNode}
          repositoryName={repositoryName}
          assetData={assetDataWithoutDownloadUrl as typeof mockAssetData}
        />
      );

      const downloadButton = screen.getByRole('button', { name: /download/i });
      await userEvent.click(downloadButton);

      expect(mockWindowOpen).toHaveBeenCalledWith(
        '/repository/maven-releases/org/apache/commons/commons-lang3/3.14.0/commons-lang3-3.14.0.jar',
        '_blank'
      );
    });
  });

  describe('docker attributes (ASTRUM-51972)', () => {
    const dockerAssetNode: BrowseNode = {
      id: '/v2/nginx/blobs/sha256:abc',
      text: 'nginx:latest',
      type: 'asset',
      leaf: true,
      assetId: 'docker-asset-1',
    };

    const dockerAssetData: AssetData = {
      id: 'docker-asset-1',
      name: '/v2/nginx/blobs/sha256:abc',
      format: 'docker',
      contentType: 'application/vnd.docker.image.rootfs.diff.tar.gzip',
      size: 10_000,
      repositoryName: 'docker-hosted',
      blobCreated: '2024-06-01T10:00:00Z',
      blobUpdated: '2024-06-02T10:00:00Z',
      lastDownloaded: null,
      attributes: {
        docker: {
          os: 'linux',
          arch: 'amd64',
          created: '2024-05-01T12:00:00Z',
          author: 'Nginx Team',
          workingDir: '/etc/nginx',
          totalSize: 52_428_800,
          exposedPorts: ['80/tcp', '443/tcp'],
          cmd: ['nginx', '-g', 'daemon off;'],
          entrypoint: ['/docker-entrypoint.sh'],
        },
      },
    };

    it('renders docker metadata fields in the attributes', () => {
      renderWithTheme(
        <DetailPanel
          node={dockerAssetNode}
          repositoryName="docker-hosted"
          assetData={dockerAssetData}
        />,
      );

      expect(screen.getByText('OS')).toBeInTheDocument();
      expect(screen.getByText('linux')).toBeInTheDocument();
      expect(screen.getByText('Architecture')).toBeInTheDocument();
      expect(screen.getByText('amd64')).toBeInTheDocument();
      expect(screen.getByText('Author')).toBeInTheDocument();
      expect(screen.getByText('Nginx Team')).toBeInTheDocument();
      expect(screen.getByText('Working directory')).toBeInTheDocument();
      expect(screen.getByText('/etc/nginx')).toBeInTheDocument();
      expect(screen.getByText('Exposed ports')).toBeInTheDocument();
      expect(screen.getByText('80/tcp, 443/tcp')).toBeInTheDocument();
      expect(screen.getByText('Entrypoint')).toBeInTheDocument();
      expect(screen.getByText('/docker-entrypoint.sh')).toBeInTheDocument();
      expect(screen.getByText('Command')).toBeInTheDocument();
      expect(screen.getByText('nginx, -g, daemon off;')).toBeInTheDocument();
    });

    it('omits docker metadata rows when their values are absent', () => {
      const sparseAssetData: AssetData = {
        ...dockerAssetData,
        attributes: { docker: { os: 'linux' } },
      };

      renderWithTheme(
        <DetailPanel
          node={dockerAssetNode}
          repositoryName="docker-hosted"
          assetData={sparseAssetData}
        />,
      );

      expect(screen.getByText('OS')).toBeInTheDocument();
      expect(screen.queryByText('Architecture')).not.toBeInTheDocument();
      expect(screen.queryByText('Exposed ports')).not.toBeInTheDocument();
      expect(screen.queryByText('Command')).not.toBeInTheDocument();
    });

    it('does not render docker metadata rows for non-docker assets', () => {
      renderWithTheme(
        <DetailPanel
          node={mockAssetNode}
          repositoryName={repositoryName}
          assetData={mockAssetData}
        />,
      );

      expect(screen.queryByText('OS')).not.toBeInTheDocument();
      expect(screen.queryByText('Architecture')).not.toBeInTheDocument();
    });
  });

  describe('head of the item', () => {
    it('shows the component name with its version in the accent colour', () => {
      renderWithTheme(
        <DetailPanel
          node={mockAssetNode}
          repositoryName={repositoryName}
          assetData={mockAssetData}
          componentData={mockComponentData}
        />,
      );

      const heading = screen.getByRole('heading', { name: /^commons-lang3\s?:3\.14\.0$/ });
      expect(heading.querySelector('.detail-panel__title-version')).toHaveTextContent(':3.14.0');
      expect(screen.getByTestId('detail-path')).toHaveTextContent(mockAssetData.name);
    });

    it('names a Docker tag by image and tag, and a manifest stored under a digest by image and digest', () => {
      const tagNode: BrowseNode = { id: 'v2/herald-api/manifests/0.1.0', text: '0.1.0', type: 'asset', leaf: true, assetId: 'a1' };
      const { unmount } = renderWithTheme(
        <DetailPanel
          node={tagNode}
          repositoryName="docker-hosted"
          assetData={{ ...mockAssetData, format: 'docker', name: '/v2/herald-api/manifests/0.1.0', path: '/v2/herald-api/manifests/0.1.0' }}
          componentData={{ ...mockComponentData, format: 'docker', group: null, name: 'herald-api', version: '0.1.0' }}
        />,
      );
      expect(screen.getByRole('heading', { name: /^herald-api\s?:0\.1\.0$/ })).toBeInTheDocument();
      unmount();

      const digest = 'sha256:2afc9a932abb547de79b8723d4a6bf1c38e87767aaa4ee08205c692d50221ba1';
      renderWithTheme(
        <DetailPanel
          node={{ ...tagNode, id: `v2/herald-api/manifests/${encodeURIComponent(digest)}`, text: digest }}
          repositoryName="docker-hosted"
          assetData={{ ...mockAssetData, format: 'docker', name: `/v2/herald-api/manifests/${digest}`, path: `/v2/herald-api/manifests/${digest}` }}
        />,
      );
      expect(screen.getByRole('heading', { name: /^herald-api\s?@sha256:2afc9a932abb$/ })).toBeInTheDocument();
    });

    it('copies the path', async () => {
      const writeText = jest.fn().mockResolvedValue(undefined);
      Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true });
      renderWithTheme(
        <DetailPanel node={mockAssetNode} repositoryName={repositoryName} assetData={mockAssetData} />,
      );

      await userEvent.click(screen.getByRole('button', { name: 'Copy path' }));

      expect(writeText).toHaveBeenCalledWith(mockAssetData.name);
      await waitFor(() => expect(screen.getByRole('button', { name: 'Copy path' })).toHaveAttribute('data-copied', 'true'));
    });

    it('shows the path of a folder with a copy button', () => {
      renderWithTheme(<DetailPanel node={mockFolderNode} repositoryName={repositoryName} />);

      expect(screen.getByTestId('detail-path')).toHaveTextContent('/org/apache/commons');
      expect(screen.getByRole('button', { name: 'Copy path' })).toBeInTheDocument();
    });
  });

  describe('usage', () => {
    const repository = {
      name: repositoryName,
      format: 'maven2',
      type: 'hosted' as const,
      url: 'https://astrum.example.com/repository/maven-releases',
      status: { online: true },
    };

    it('offers the clients of the format with the URL of the repository', async () => {
      renderWithTheme(
        <DetailPanel
          node={mockAssetNode}
          repositoryName={repositoryName}
          repository={repository}
          assetData={{ ...mockAssetData, attributes: { maven2: { extension: 'jar' } } }}
          componentData={mockComponentData}
        />,
      );

      const tabs = screen.getAllByRole('tab').map((tab) => tab.textContent);
      expect(tabs).toEqual(['Maven', 'Gradle Kotlin', 'Gradle Groovy', 'settings.xml', 'mvn deploy']);
      expect(screen.getByRole('tabpanel')).toHaveTextContent('<artifactId>commons-lang3</artifactId>');

      await userEvent.click(screen.getByRole('tab', { name: 'Gradle Kotlin' }));

      expect(screen.getByRole('tabpanel')).toHaveTextContent('url = uri("https://astrum.example.com/repository/maven-releases/")');
      expect(screen.getByRole('tabpanel')).toHaveTextContent('implementation("org.apache.commons:commons-lang3:3.14.0")');
    });

    it('reports a copied snippet to the snippet analytics', async () => {
      Object.defineProperty(navigator, 'clipboard', { value: { writeText: jest.fn().mockResolvedValue(undefined) }, configurable: true });
      renderWithTheme(
        <DetailPanel
          node={mockComponentNode}
          repositoryName={repositoryName}
          repository={repository}
          componentData={mockComponentData}
        />,
      );

      await userEvent.click(screen.getByRole('button', { name: 'Copy the Maven snippet' }));

      await waitFor(() => expect(mockTrackSnippetCopy).toHaveBeenCalledWith('maven2', 'Maven'));
    });

    it('builds the Docker snippets from the registry of the repository', () => {
      const tagNode: BrowseNode = { id: 'v2/herald-api/manifests/0.1.0', text: '0.1.0', type: 'asset', leaf: true, assetId: 'a1' };
      renderWithTheme(
        <DetailPanel
          node={tagNode}
          repositoryName="docker-hosted"
          repository={{ ...repository, name: 'docker-hosted', format: 'docker', url: 'https://astrum.example.com/repository/docker-hosted' }}
          assetData={{
            ...mockAssetData,
            format: 'docker',
            name: '/v2/herald-api/manifests/0.1.0',
            path: '/v2/herald-api/manifests/0.1.0',
            registryUrl: 'astrum.example.com/docker-hosted',
            checksum: { sha256: '2afc9a93', sha1: 'aa11' },
          }}
          componentData={{ ...mockComponentData, format: 'docker', group: null, name: 'herald-api', version: '0.1.0' }}
        />,
      );

      expect(screen.getAllByRole('tab').map((tab) => tab.textContent)).toEqual([
        'docker pull', 'docker login', 'Dockerfile', 'Compose', 'Kubernetes', 'oras discover', 'docker push',
      ]);
      expect(screen.getByRole('tabpanel')).toHaveTextContent('docker pull astrum.example.com/docker-hosted/herald-api:0.1.0');
      expect(screen.getByText('Digest').parentElement).toHaveTextContent('sha256:2afc9a93');
      expect(screen.getByText('SHA-1').parentElement).toHaveTextContent('aa11');
    });

    it('names a Helm chart, offers helm and lists the artifacts attached to it', async () => {
      const digest = 'sha256:85e57ba7a6537efc05d47ce917a61246f127dc49e29160396b4f3e980a197958';
      const signature = 'sha256:02dde9ac789b0f02f41817894ba8949e53a7438a70aa25c9d9dd6ed0507dcedd';
      mockAxiosGet.mockResolvedValue({
        data: {
          manifests: [{
            mediaType: 'application/vnd.oci.image.manifest.v1+json',
            digest: signature,
            size: 728,
            artifactType: 'application/vnd.cncf.notary.signature',
            annotations: { 'org.opencontainers.image.created': '2026-10-12T12:02:23Z' },
          }],
        },
      });
      const onOpenPath = jest.fn();
      const tagNode: BrowseNode = { id: 'v2/charts/mychart/manifests/0.1.0', text: '0.1.0', type: 'asset', leaf: true, assetId: 'a1' };
      renderWithTheme(
        <DetailPanel
          node={tagNode}
          repositoryName="docker-hosted"
          repository={{ ...repository, name: 'docker-hosted', format: 'docker', url: 'https://astrum.example.com/repository/docker-hosted' }}
          assetData={{
            ...mockAssetData,
            format: 'docker',
            name: '/v2/charts/mychart/manifests/0.1.0',
            path: '/v2/charts/mychart/manifests/0.1.0',
            registryUrl: 'astrum.example.com/docker-hosted',
            attributes: { docker: { digest, artifactKind: 'helm', artifactType: 'application/vnd.cncf.helm.config.v1+json' } },
          }}
          componentData={{ ...mockComponentData, format: 'docker', group: null, name: 'charts/mychart', version: '0.1.0' }}
          onOpenPath={onOpenPath}
        />,
      );

      expect(screen.getAllByRole('tab').map((tab) => tab.textContent)).toEqual([
        'helm pull', 'helm install', 'helm registry login', 'helm push',
      ]);
      expect(screen.getAllByText('Kind')[0].parentElement).toHaveTextContent('Helm chart');
      expect(mockAxiosGet).toHaveBeenCalledWith(
        `/repository/docker-hosted/v2/charts/mychart/referrers/${digest}`,
        expect.anything(),
      );
      const open = await screen.findByRole('button', { name: `Open ${signature}` });
      expect(open).toHaveTextContent('Signature');
      expect(open).toHaveTextContent('application/vnd.cncf.notary.signature');
      await userEvent.click(open);
      expect(onOpenPath).toHaveBeenCalledWith(`v2/charts/mychart/manifests/${signature}`);
    });

    it('links an attached artifact to the manifest it belongs to', async () => {
      const subject = 'sha256:6f48b8ceff217c8be876966ae98d1f15a251baaf81eaefc0c463f2784a5d2538';
      const sbom = 'sha256:63fc8afdfdb7883b3819e411a4837c32ec61158c26133692edb5d7d8a2143d48';
      const onOpenPath = jest.fn();
      const node: BrowseNode = { id: `v2/demo/app/manifests/${sbom}`, text: sbom, type: 'asset', leaf: true, assetId: 'a3' };
      renderWithTheme(
        <DetailPanel
          node={node}
          repositoryName="docker-hosted"
          repository={{ ...repository, name: 'docker-hosted', format: 'docker', url: 'https://astrum.example.com/repository/docker-hosted' }}
          assetData={{
            ...mockAssetData,
            format: 'docker',
            name: `/v2/demo/app/manifests/${sbom}`,
            path: `/v2/demo/app/manifests/${sbom}`,
            attributes: { docker: { digest: sbom, subject, artifactKind: 'sbom', artifactType: 'application/spdx+json' } },
          }}
          onOpenPath={onOpenPath}
        />,
      );

      expect(screen.getAllByRole('tab').map((tab) => tab.textContent)).toEqual([
        'oras pull', 'oras manifest fetch', 'oras discover', 'oras login', 'oras attach',
      ]);
      expect(screen.getAllByText('Kind')[0].parentElement).toHaveTextContent('SBOM');
      await userEvent.click(screen.getByRole('button', { name: subject }));
      expect(onOpenPath).toHaveBeenCalledWith(`v2/demo/app/manifests/${subject}`);
    });

    it('says when nothing is attached to a manifest', async () => {
      mockAxiosGet.mockResolvedValue({ data: { schemaVersion: 2, manifests: [] } });
      const digest = 'sha256:6f48b8ceff217c8be876966ae98d1f15a251baaf81eaefc0c463f2784a5d2538';
      const node: BrowseNode = { id: `v2/demo/app/manifests/${digest}`, text: digest, type: 'asset', leaf: true, assetId: 'a2' };
      renderWithTheme(
        <DetailPanel
          node={node}
          repositoryName="docker-hosted"
          repository={{ ...repository, name: 'docker-hosted', format: 'docker', url: 'https://astrum.example.com/repository/docker-hosted' }}
          assetData={{ ...mockAssetData, format: 'docker', name: `/v2/demo/app/manifests/${digest}`, path: `/v2/demo/app/manifests/${digest}` }}
        />,
      );

      expect(await screen.findByText('Nothing is attached to this manifest')).toBeInTheDocument();
      expect(mockAxiosGet).toHaveBeenCalledWith(`/repository/docker-hosted/v2/demo/app/referrers/${digest}`, expect.anything());
    });

    it('shows no usage for a folder', () => {
      renderWithTheme(<DetailPanel node={mockFolderNode} repositoryName={repositoryName} repository={repository} />);

      expect(screen.queryByTestId('usage-block')).not.toBeInTheDocument();
    });
  });

  describe('details', () => {
    it('lists the details of a file in a definition list', () => {
      renderWithTheme(
        <DetailPanel
          node={mockAssetNode}
          repositoryName={repositoryName}
          assetData={{ ...mockAssetData, createdBy: 'admin', checksum: { sha1: 'abc123' } }}
          componentData={mockComponentData}
        />,
      );

      const details = screen.getByTestId('detail-details');
      expect(details.querySelector('dl')).not.toBeNull();
      for (const label of ['Repository', 'Format', 'Group', 'Component', 'Version', 'Uploaded by', 'Content type', 'Size', 'Created', 'Last downloaded', 'Downloads', 'SHA-1']) {
        expect(screen.getByText(label)).toBeInTheDocument();
      }
      expect(screen.getByText('admin')).toBeInTheDocument();
      expect(screen.getByText('abc123')).toHaveClass('detail-panel__value--mono');
    });
  });
});
