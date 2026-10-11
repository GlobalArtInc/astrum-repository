

import React, { useCallback, useRef, useEffect } from 'react';
import { Box, Flex, Text, Spinner } from '@radix-ui/themes';
import { ChevronRight, ChevronDown, Folder, FolderOpen, Package, File, Tag } from 'lucide-react';

import UIStrings from '../../../constants/UIStrings';
import type { BrowseTreeNodeProps, BrowseNodeType } from './browse-tree.types';
import './BrowseTree.scss';

/**
 * Get the appropriate icon for a node type.
 * Folder: FolderOpen when expanded or selected, else Folder.
 * Docker shows images without an icon and tags with a tag.
 */
function NodeIcon({
  type,
  expanded,
  isSelected,
  leaf,
  format,
}: {
  type: BrowseNodeType;
  expanded?: boolean;
  isSelected?: boolean;
  leaf?: boolean;
  format?: string;
}): JSX.Element | null {
  const iconProps = { size: 16, className: 'browse-tree__node-icon', 'aria-hidden': true };

  if (format === 'docker') {
    return leaf ? <Tag {...iconProps} size={14} data-testid="icon-tag" /> : null;
  }

  switch (type) {
    case 'folder':
      return (expanded || isSelected) ? (
        <FolderOpen {...iconProps} data-testid="icon-folder-open" />
      ) : (
        <Folder {...iconProps} data-testid="icon-folder" />
      );
    case 'component':
      return <Package {...iconProps} data-testid="icon-component" />;
    default:
      return <File {...iconProps} data-testid="icon-asset" />;
  }
}

/**
 * Individual node in the browse tree.
 *
 * Handles:
 * - Expand/collapse for non-leaf nodes
 * - Lazy loading indicator
 * - Node selection and navigation
 * - Keyboard accessibility
 */
export function BrowseTreeNode({
  nodeState,
  repositoryName,
  depth,
  onToggle,
  onSelect,
  isFocused,
  selectedNodeId,
  onKeyDown,
  baseUrl,
  format,
}: BrowseTreeNodeProps): JSX.Element {
  const strings = UIStrings.BROWSE.TREE;
  const { node, expanded, children, loading, error } = nodeState;
  const nodeRef = useRef<HTMLDivElement>(null);

  // Focus management
  useEffect(() => {
    if (isFocused && nodeRef.current) {
      nodeRef.current.focus();
    }
  }, [isFocused]);

  /**
   * Handle click on the expand/collapse chevron.
   */
  const handleToggleClick = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (!node.leaf) {
        onToggle(node.id);
      }
    },
    [node.id, node.leaf, onToggle]
  );

  /**
   * Handle click on the node content.
   * Uses a non-navigating element to prevent the browser from
   * following an href and corrupting the URL hash (see 9mw3).
   */
  const handleNodeClick = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (onSelect) {
        onSelect(node);
      }
      // An image of the Docker view is opened by clicking it, like a disclosure.
      if (format === 'docker' && !node.leaf && !expanded) {
        onToggle(node.id);
      }
    },
    [node, onSelect, format, expanded, onToggle]
  );

  /**
   * Handle keyboard events.
   */
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      onKeyDown(e, node.id);
    },
    [node.id, onKeyDown]
  );

  const isSelected = selectedNodeId === node.id;
  const isMono = format === 'docker' && node.leaf;

  return (
    <div
      className="browse-tree__node-container"
      data-node-id={node.id}
      role="treeitem"
      aria-expanded={node.leaf ? undefined : expanded}
      aria-selected={isSelected}
      aria-level={depth + 1}
    >
      <Flex
        ref={nodeRef}
        className={`browse-tree__node ${isFocused ? 'browse-tree__node--focused' : ''} ${isSelected ? 'browse-tree__node--selected' : ''}`}
        align="center"
        gap="2"
        tabIndex={isFocused ? 0 : -1}
        onKeyDown={handleKeyDown}
        onClick={node.leaf ? handleNodeClick : undefined}
        data-testid={`tree-node-${node.id}`}
      >
        {/* Expand/Collapse Toggle */}
        <Box
          className="browse-tree__toggle"
          onClick={handleToggleClick}
          style={{
            cursor: node.leaf ? 'default' : 'pointer',
            display: node.leaf && format === 'docker' ? 'none' : undefined,
            opacity: node.leaf ? 0 : 1,
          }}
          aria-label={expanded ? strings.COLLAPSE : strings.EXPAND}
          aria-hidden={node.leaf}
          data-testid={`toggle-${node.id}`}
        >
          {!node.leaf && (
            loading ? (
              <Spinner size="1" />
            ) : expanded ? (
              <ChevronDown size={16} />
            ) : (
              <ChevronRight size={16} />
            )
          )}
        </Box>

        {/* Node Icon — clickable, same as label */}
        <Box
          onClick={handleNodeClick}
          style={{ cursor: 'pointer', display: 'flex', alignItems: 'center' }}
        >
          <NodeIcon
            type={node.type}
            expanded={expanded}
            isSelected={isSelected}
            leaf={node.leaf}
            format={format}
          />
        </Box>

        {/* Node Label — intentionally NOT an <a href> to prevent
            the browser from following a hash link and corrupting the
            URL with duplicate path segments (bug 9mw3). Navigation is
            handled programmatically by BrowsePage.handleSelectNode. */}
        <Text
          size="2"
          onClick={handleNodeClick}
          className={`browse-tree__node-link ${isMono ? 'browse-tree__node-link--mono' : ''}`}
          tabIndex={-1}
          data-testid={`link-${node.id}`}
        >
          {node.text}
        </Text>
      </Flex>

      {/* Error message */}
      {error && (
        <Box className="browse-tree__error" pl="6">
          <Text size="1" color="red">
            {error}
          </Text>
        </Box>
      )}

      {/* Children (if expanded) */}
      {expanded && children && children.length > 0 && (
        <div role="group" className="browse-tree__children">
          {children.map((childState) => (
            <BrowseTreeNode
              key={childState.node.id}
              nodeState={childState}
              repositoryName={repositoryName}
              depth={depth + 1}
              onToggle={onToggle}
              onSelect={onSelect}
              isFocused={false}
              selectedNodeId={selectedNodeId}
              onKeyDown={onKeyDown}
              baseUrl={baseUrl}
              format={format}
            />
          ))}
        </div>
      )}
    </div>
  );
}

export default BrowseTreeNode;

