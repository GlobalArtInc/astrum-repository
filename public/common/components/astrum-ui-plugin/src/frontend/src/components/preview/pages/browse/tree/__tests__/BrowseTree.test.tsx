

import React from 'react';
import { render, screen, waitFor, fireEvent, } from '@testing-library/react';
import '@testing-library/jest-dom';

import { BrowseTree } from '../BrowseTree';
import { BrowseTreeNode } from '../BrowseTreeNode';
import { mockRootNodes, mockFolder1Children, getMockChildren } from './mockData';
import type { TreeNodeState, BrowseNode } from '../browse-tree.types';

// Mock the relative paths that the source imports from
jest.mock('../../../../../../interface/ExtAPIUtils', () => ({
  ExtAPIUtils: {
    extAPIRequest: jest.fn(),
    checkForErrorAndExtract: jest.fn((response) => response.data.result.data),
  },
}));

jest.mock('../../../../../../constants/APIConstants', () => ({
  APIConstants: {
    EXT: {
      BROWSE: {
        ACTION: 'coreui_Browse',
        METHODS: {
          READ: 'read',
        },
      },
    },
  },
}));

// Get the mocked module
import { ExtAPIUtils } from '../../../../../../interface/ExtAPIUtils';

/**
 * Helper to create a mock API response.
 */
function mockApiResponse(data: BrowseNode[]) {
  return {
    data: {
      result: {
        success: true,
        data,
      },
    },
  };
}

/**
 * Helper to setup the ExtAPIUtils mock for a specific node.
 */
function setupApiMock() {
  ExtAPIUtils.extAPIRequest.mockImplementation(
    async (_action: string, _method: string, { data }: { data: [{ repositoryName: string; node: string }] }) => {
      const requestedNodeId = data[0].node;
      const mockChildren = getMockChildren(requestedNodeId);
      return mockApiResponse(mockChildren);
    }
  );
}

describe('BrowseTree', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    setupApiMock();
  });

  describe('rendering', () => {
    it('renders loading state initially', () => {
      ExtAPIUtils.extAPIRequest.mockImplementation(() => new Promise(() => {})); // Never resolves
      render(<BrowseTree repositoryName="maven-releases" />);

      expect(screen.getByText(/loading/i)).toBeInTheDocument();
    });

    it('renders the tree with root nodes after loading', async () => {
      render(<BrowseTree repositoryName="maven-releases" />);

      await waitFor(() => {
        expect(screen.getByText('folder1')).toBeInTheDocument();
      });

      expect(screen.getByText('component1')).toBeInTheDocument();
      expect(screen.getByText('asset1.jar')).toBeInTheDocument();
    });

    it('shows the full name of a node on hover, as long names are cut off', async () => {
      render(<BrowseTree repositoryName="maven-releases" />);

      await waitFor(() => {
        expect(screen.getByText('asset1.jar')).toHaveAttribute('title', 'asset1.jar');
      });
    });

    it('renders a tree role element with aria-label', async () => {
      render(<BrowseTree repositoryName="maven-releases" />);

      await waitFor(() => {
        const tree = screen.getByRole('tree');
        expect(tree).toHaveAttribute('aria-label', 'Contents of maven-releases');
      });
    });

    it('renders empty state when no nodes', async () => {
      ExtAPIUtils.extAPIRequest.mockResolvedValue(mockApiResponse([]));
      render(<BrowseTree repositoryName="empty-repo" />);

      await waitFor(() => {
        expect(screen.getByText(/this repository is empty/i)).toBeInTheDocument();
      });
      expect(screen.getByTestId('empty-state')).toBeInTheDocument();
    });

    it('renders error state with retry button', async () => {
      ExtAPIUtils.extAPIRequest.mockRejectedValue(new Error('Network error'));
      render(<BrowseTree repositoryName="maven-releases" />);

      await waitFor(() => {
        expect(screen.getByText(/network error/i)).toBeInTheDocument();
      });

      expect(screen.getByRole('button', { name: /retry/i })).toBeInTheDocument();
    });
  });

  describe('icons', () => {
    it('renders folder icon for folder type', async () => {
      render(<BrowseTree repositoryName="maven-releases" />);

      await waitFor(() => {
        expect(screen.getByTestId('icon-folder')).toBeInTheDocument();
      });
    });

    it('renders package icon for component type', async () => {
      render(<BrowseTree repositoryName="maven-releases" />);

      await waitFor(() => {
        expect(screen.getByTestId('icon-component')).toBeInTheDocument();
      });
    });

    it('renders file icon for asset type', async () => {
      render(<BrowseTree repositoryName="maven-releases" />);

      await waitFor(() => {
        expect(screen.getByTestId('icon-asset')).toBeInTheDocument();
      });
    });
  });

  describe('expand/collapse', () => {
    it('expands a folder when clicking the toggle', async () => {
      render(<BrowseTree repositoryName="maven-releases" />);

      await waitFor(() => {
        expect(screen.getByText('folder1')).toBeInTheDocument();
      });

      // Click the toggle for folder1
      const toggleButton = screen.getByTestId('toggle-folder1');
      fireEvent.click(toggleButton);

      // Should load and show children
      await waitFor(() => {
        expect(screen.getByText('subfolder1')).toBeInTheDocument();
      });

      expect(screen.getByText('component2')).toBeInTheDocument();
    });

    it('collapses an expanded folder when clicking the toggle', async () => {
      render(<BrowseTree repositoryName="maven-releases" />);

      await waitFor(() => {
        expect(screen.getByText('folder1')).toBeInTheDocument();
      });

      // Expand folder1
      const toggleButton = screen.getByTestId('toggle-folder1');
      fireEvent.click(toggleButton);

      await waitFor(() => {
        expect(screen.getByText('subfolder1')).toBeInTheDocument();
      });

      // Collapse folder1
      fireEvent.click(toggleButton);

      await waitFor(() => {
        expect(screen.queryByText('subfolder1')).not.toBeInTheDocument();
      });
    });

    it('does not show toggle for leaf nodes', async () => {
      render(<BrowseTree repositoryName="maven-releases" />);

      await waitFor(() => {
        expect(screen.getByText('asset1.jar')).toBeInTheDocument();
      });

      // The toggle for asset1.jar should be hidden (opacity: 0)
      const toggle = screen.getByTestId('toggle-asset1.jar');
      expect(toggle).toHaveStyle({ opacity: '0' });
    });
  });

  describe('selection', () => {
    it('calls onSelect when a node is clicked', async () => {
      const onSelect = jest.fn();
      render(<BrowseTree repositoryName="maven-releases" onSelect={onSelect} />);

      await waitFor(() => {
        expect(screen.getByText('folder1')).toBeInTheDocument();
      });

      // Click on folder1 link
      const link = screen.getByTestId('link-folder1');
      fireEvent.click(link);

      expect(onSelect).toHaveBeenCalledWith(
        expect.objectContaining({
          id: 'folder1',
          type: 'folder',
        })
      );
    });
  });

  describe('accessibility', () => {
    it('has proper tree role', async () => {
      render(<BrowseTree repositoryName="maven-releases" />);

      await waitFor(() => {
        expect(screen.getByRole('tree')).toBeInTheDocument();
      });
    });

    it('has proper treeitem role for nodes', async () => {
      render(<BrowseTree repositoryName="maven-releases" />);

      await waitFor(() => {
        expect(screen.getAllByRole('treeitem')).toHaveLength(3);
      });
    });

    it('has aria-expanded on expandable nodes', async () => {
      render(<BrowseTree repositoryName="maven-releases" />);

      await waitFor(() => {
        expect(screen.getByText('folder1')).toBeInTheDocument();
      });

      const folder1Item = screen.getAllByRole('treeitem')[0];
      expect(folder1Item).toHaveAttribute('aria-expanded', 'false');
    });

    it('has aria-label on the tree', async () => {
      render(<BrowseTree repositoryName="maven-releases" />);

      await waitFor(() => {
        const tree = screen.getByRole('tree');
        expect(tree).toHaveAttribute('aria-label', 'Contents of maven-releases');
      });
    });
  });

  describe('error handling', () => {
    it('retries on error when clicking retry button', async () => {
      // First call fails
      ExtAPIUtils.extAPIRequest.mockRejectedValueOnce(new Error('Network error'));

      render(<BrowseTree repositoryName="maven-releases" />);

      await waitFor(() => {
        expect(screen.getByText(/network error/i)).toBeInTheDocument();
      });

      // Setup successful response for retry
      setupApiMock();

      // Click retry
      const retryButton = screen.getByRole('button', { name: /retry/i });
      fireEvent.click(retryButton);

      // Should show tree after successful retry
      await waitFor(() => {
        expect(screen.getByText('folder1')).toBeInTheDocument();
      });
    });
  });

  describe('keyboard navigation', () => {
    it('handles ArrowDown key to move focus', async () => {
      render(<BrowseTree repositoryName="maven-releases" />);

      await waitFor(() => {
        expect(screen.getByText('folder1')).toBeInTheDocument();
      });

      const firstNode = screen.getByTestId('tree-node-folder1');
      fireEvent.keyDown(firstNode, { key: 'ArrowDown' });

      // Should not throw and handle the key
      expect(firstNode).toBeInTheDocument();
    });

    it('handles ArrowUp key to move focus', async () => {
      render(<BrowseTree repositoryName="maven-releases" />);

      await waitFor(() => {
        expect(screen.getByText('folder1')).toBeInTheDocument();
      });

      const firstNode = screen.getByTestId('tree-node-folder1');
      fireEvent.keyDown(firstNode, { key: 'ArrowUp' });

      expect(firstNode).toBeInTheDocument();
    });

    it('handles ArrowRight key to expand', async () => {
      render(<BrowseTree repositoryName="maven-releases" />);

      await waitFor(() => {
        expect(screen.getByText('folder1')).toBeInTheDocument();
      });

      const firstNode = screen.getByTestId('tree-node-folder1');
      fireEvent.keyDown(firstNode, { key: 'ArrowRight' });

      // Should trigger expansion
      await waitFor(() => {
        expect(screen.getByText('subfolder1')).toBeInTheDocument();
      });
    });

    it('handles ArrowLeft key to collapse or focus parent', async () => {
      render(<BrowseTree repositoryName="maven-releases" />);

      await waitFor(() => {
        expect(screen.getByText('folder1')).toBeInTheDocument();
      });

      const firstNode = screen.getByTestId('tree-node-folder1');
      fireEvent.keyDown(firstNode, { key: 'ArrowLeft' });

      expect(firstNode).toBeInTheDocument();
    });

    it('handles Enter key to select node', async () => {
      const onSelect = jest.fn();
      render(<BrowseTree repositoryName="maven-releases" onSelect={onSelect} />);

      await waitFor(() => {
        expect(screen.getByText('folder1')).toBeInTheDocument();
      });

      const firstNode = screen.getByTestId('tree-node-folder1');
      fireEvent.keyDown(firstNode, { key: 'Enter' });

      expect(onSelect).toHaveBeenCalled();
    });

    it('handles Space key to select node', async () => {
      const onSelect = jest.fn();
      render(<BrowseTree repositoryName="maven-releases" onSelect={onSelect} />);

      await waitFor(() => {
        expect(screen.getByText('folder1')).toBeInTheDocument();
      });

      const firstNode = screen.getByTestId('tree-node-folder1');
      fireEvent.keyDown(firstNode, { key: ' ' });

      expect(onSelect).toHaveBeenCalled();
    });

    it('ignores other keys', async () => {
      render(<BrowseTree repositoryName="maven-releases" />);

      await waitFor(() => {
        expect(screen.getByText('folder1')).toBeInTheDocument();
      });

      const firstNode = screen.getByTestId('tree-node-folder1');
      // Should not throw for unknown key
      fireEvent.keyDown(firstNode, { key: 'x' });

      expect(firstNode).toBeInTheDocument();
    });
  });

  describe('custom props', () => {
    it('applies custom className to container', async () => {
      render(<BrowseTree repositoryName="maven-releases" className="custom-class" />);

      await waitFor(() => {
        const container = screen.getByTestId('browse-tree');
        expect(container).toHaveClass('browse-tree');
        expect(container).toHaveClass('custom-class');
      });
    });

    it('applies custom className with baseUrl', async () => {
      render(<BrowseTree repositoryName="maven-releases" baseUrl="custom/browse/path" className="test-class" />);

      await waitFor(() => {
        const container = screen.getByTestId('browse-tree');
        expect(container).toHaveClass('test-class');
      });
    });
  });

  // BrowseTree search functionality was removed; search is now handled by InRepositorySearch.
});

describe('BrowseTreeNode', () => {
  const defaultProps = {
    repositoryName: 'maven-releases',
    depth: 0,
    onToggle: jest.fn(),
    onSelect: jest.fn(),
    isFocused: false,
    onKeyDown: jest.fn(),
    baseUrl: 'browse/browse/maven-releases',
  };

  const createNodeState = (overrides: Partial<TreeNodeState> = {}): TreeNodeState => ({
    node: mockRootNodes[0], // folder1
    expanded: false,
    children: null,
    loading: false,
    ...overrides,
  });

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders the node text', () => {
    render(
      <BrowseTreeNode
        {...defaultProps}
        nodeState={createNodeState()}
      />
    );

    expect(screen.getByText('folder1')).toBeInTheDocument();
  });

  it('calls onToggle when toggle is clicked', async () => {
    const onToggle = jest.fn();

    render(
      <BrowseTreeNode
        {...defaultProps}
        onToggle={onToggle}
        nodeState={createNodeState()}
      />
    );

    const toggle = screen.getByTestId('toggle-folder1');
    fireEvent.click(toggle);

    expect(onToggle).toHaveBeenCalledWith('folder1');
  });

  it('calls onSelect when link is clicked', async () => {
    const onSelect = jest.fn();

    render(
      <BrowseTreeNode
        {...defaultProps}
        onSelect={onSelect}
        nodeState={createNodeState()}
      />
    );

    const link = screen.getByTestId('link-folder1');
    fireEvent.click(link);

    expect(onSelect).toHaveBeenCalledWith(mockRootNodes[0]);
  });

  it('shows loading spinner when loading', () => {
    render(
      <BrowseTreeNode
        {...defaultProps}
        nodeState={createNodeState({ loading: true })}
      />
    );

    // The loading spinner should be present (Radix Spinner)
    const toggle = screen.getByTestId('toggle-folder1');
    expect(toggle.querySelector('.rt-Spinner')).toBeInTheDocument();
  });

  it('renders children when expanded', () => {
    const childNodeStates: TreeNodeState[] = mockFolder1Children.map((node) => ({
      node,
      expanded: false,
      children: null,
      loading: false,
    }));

    render(
      <BrowseTreeNode
        {...defaultProps}
        nodeState={createNodeState({ expanded: true, children: childNodeStates })}
      />
    );

    expect(screen.getByText('subfolder1')).toBeInTheDocument();
    expect(screen.getByText('component2')).toBeInTheDocument();
  });

  it('applies focused class when isFocused is true', () => {
    render(
      <BrowseTreeNode
        {...defaultProps}
        isFocused={true}
        nodeState={createNodeState()}
      />
    );

    const node = screen.getByTestId('tree-node-folder1');
    expect(node).toHaveClass('browse-tree__node--focused');
  });

  it('shows error message when there is an error', () => {
    render(
      <BrowseTreeNode
        {...defaultProps}
        nodeState={createNodeState({ error: 'Failed to load children' })}
      />
    );

    expect(screen.getByText('Failed to load children')).toBeInTheDocument();
  });

  it('renders a clickable node label without a navigating href', () => {
    render(
      <BrowseTreeNode
        {...defaultProps}
        nodeState={createNodeState()}
      />
    );

    const link = screen.getByTestId('link-folder1');
    expect(link).toBeInTheDocument();
    expect(link).not.toHaveAttribute('href');
  });

  it('reports its depth as the tree level', () => {
    render(
      <BrowseTreeNode
        {...defaultProps}
        depth={2}
        nodeState={createNodeState()}
      />
    );

    expect(screen.getByRole('treeitem')).toHaveAttribute('aria-level', '3');
  });

  it('marks the selected node', () => {
    render(
      <BrowseTreeNode
        {...defaultProps}
        selectedNodeId="folder1"
        nodeState={createNodeState()}
      />
    );

    expect(screen.getByRole('treeitem')).toHaveAttribute('aria-selected', 'true');
    expect(screen.getByTestId('tree-node-folder1')).toHaveClass('browse-tree__node--selected');
  });
});

describe('BrowseTree filter', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    setupApiMock();
  });

  it('shows only the loaded nodes that match and reports how many match', async () => {
    const onFilterResult = jest.fn();
    const { rerender } = render(<BrowseTree repositoryName="maven-releases" onFilterResult={onFilterResult} />);
    await waitFor(() => expect(screen.getByRole('tree')).toBeInTheDocument());
    const [first, second] = mockRootNodes;

    rerender(<BrowseTree repositoryName="maven-releases" filter={first.text} onFilterResult={onFilterResult} />);

    expect(screen.getByTestId(`tree-node-${first.id}`)).toBeInTheDocument();
    expect(screen.queryByTestId(`tree-node-${second.id}`)).not.toBeInTheDocument();
    expect(onFilterResult).toHaveBeenLastCalledWith(expect.any(Number));
  });

  it('says so when nothing loaded matches', async () => {
    const { rerender } = render(<BrowseTree repositoryName="maven-releases" />);
    await waitFor(() => expect(screen.getByRole('tree')).toBeInTheDocument());

    rerender(<BrowseTree repositoryName="maven-releases" filter="no-such-node" />);

    expect(screen.getByTestId('browse-tree-no-matches')).toHaveTextContent('Nothing loaded matches the filter');
  });
});

describe('BrowseTree Docker view', () => {
  const node = (id: string, text: string, type: BrowseNode['type'], leaf: boolean): BrowseNode => ({ id, text, type, leaf });
  const DOCKER_TREE: Record<string, BrowseNode[]> = {
    '/': [node('v2', 'v2', 'folder', false)],
    v2: [node('v2/-', '-', 'folder', false), node('v2/herald-api', 'herald-api', 'folder', false), node('v2/redis', 'redis', 'folder', false)],
    'v2/herald-api': [node('v2/herald-api/manifests', 'manifests', 'folder', false)],
    'v2/herald-api/manifests': [
      node('v2/herald-api/manifests/0.1.0', '0.1.0', 'asset', true),
      node('v2/herald-api/manifests/0.2.0', '0.2.0', 'asset', true),
      node('v2/herald-api/manifests/sha256%3Aabc', 'sha256:abc', 'asset', true),
    ],
    'v2/redis': [node('v2/redis/manifests', 'manifests', 'folder', false)],
    'v2/redis/manifests': [node('v2/redis/manifests/sha256%3Adef', 'sha256:def', 'asset', true)],
  };

  beforeEach(() => {
    jest.clearAllMocks();
    Element.prototype.scrollIntoView = jest.fn();
    ExtAPIUtils.extAPIRequest.mockImplementation(
      async (_action: string, _method: string, { data }: { data: [{ repositoryName: string; node: string }] }) =>
        mockApiResponse(DOCKER_TREE[data[0].node] ?? []),
    );
  });

  it('lists the images at the root, without the registry folder and the blob store', async () => {
    render(<BrowseTree repositoryName="docker-hosted" format="docker" />);

    await waitFor(() => expect(screen.getByTestId('tree-node-v2/herald-api')).toBeInTheDocument());
    expect(screen.getByTestId('tree-node-v2/redis')).toBeInTheDocument();
    expect(screen.queryByTestId('tree-node-v2')).not.toBeInTheDocument();
    expect(screen.queryByTestId('tree-node-v2/-')).not.toBeInTheDocument();
  });

  it('opens an image on click and lists its tags, with its digests in the manifests folder below them', async () => {
    const onSelect = jest.fn();
    render(<BrowseTree repositoryName="docker-hosted" format="docker" onSelect={onSelect} />);
    await waitFor(() => expect(screen.getByTestId('link-v2/herald-api')).toBeInTheDocument());

    fireEvent.click(screen.getByTestId('link-v2/herald-api'));

    await waitFor(() => expect(screen.getByTestId('tree-node-v2/herald-api/manifests/0.1.0')).toBeInTheDocument());
    expect(screen.getByTestId('tree-node-v2/herald-api/manifests/0.2.0')).toBeInTheDocument();
    expect(screen.getByTestId('tree-node-v2/herald-api/manifests')).toBeInTheDocument();
    expect(screen.queryByTestId('tree-node-v2/herald-api/manifests/sha256%3Aabc')).not.toBeInTheDocument();
    expect(screen.getAllByTestId('icon-tag')).toHaveLength(2);
    expect(onSelect).toHaveBeenCalledWith(expect.objectContaining({ id: 'v2/herald-api' }));

    fireEvent.click(screen.getByTestId('link-v2/herald-api/manifests'));

    await waitFor(() => expect(screen.getByTestId('tree-node-v2/herald-api/manifests/sha256%3Aabc')).toBeInTheDocument());
    expect(screen.getAllByTestId('tree-node-v2/herald-api/manifests/0.1.0')).toHaveLength(1);
  });

  it('keeps the digests of an image without tags', async () => {
    render(<BrowseTree repositoryName="docker-hosted" format="docker" />);
    await waitFor(() => expect(screen.getByTestId('link-v2/redis')).toBeInTheDocument());

    fireEvent.click(screen.getByTestId('link-v2/redis'));

    await waitFor(() => expect(screen.getByTestId('tree-node-v2/redis/manifests/sha256%3Adef')).toBeInTheDocument());
  });

  it('opens a deep link to a tag through the segments the view leaves out', async () => {
    const onSelect = jest.fn();
    render(
      <BrowseTree
        repositoryName="docker-hosted"
        format="docker"
        initialPath="v2/herald-api/manifests/0.2.0"
        onSelect={onSelect}
      />,
    );

    await waitFor(() => expect(onSelect).toHaveBeenCalledWith(expect.objectContaining({ id: 'v2/herald-api/manifests/0.2.0' })));
    expect(screen.getByRole('treeitem', { selected: true })).toHaveTextContent('0.2.0');
  });

  it('opens a deep link to a manifest stored under a digest through the manifests folder', async () => {
    const onSelect = jest.fn();
    render(
      <BrowseTree
        repositoryName="docker-hosted"
        format="docker"
        initialPath="v2/herald-api/manifests/sha256:abc"
        onSelect={onSelect}
      />,
    );

    await waitFor(() => expect(onSelect).toHaveBeenCalledWith(
      expect.objectContaining({ id: 'v2/herald-api/manifests/sha256%3Aabc' }),
    ));
    expect(screen.getByRole('treeitem', { selected: true })).toHaveTextContent('sha256:abc');
  });
});
