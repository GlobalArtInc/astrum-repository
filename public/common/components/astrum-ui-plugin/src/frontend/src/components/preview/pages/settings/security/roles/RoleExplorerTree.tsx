

import React, { useState, useCallback } from 'react';
import { Box, Flex, Text, IconButton, ScrollArea, Tooltip, TextField, Separator } from '@radix-ui/themes';
import {
  Shield,
  Key,
  Filter,
  ChevronRight,
  ChevronDown,
  AlertTriangle,
  Loader2,
  Search,
  ChevronsDown,
  ChevronsUp
} from 'lucide-react';
import { SecurityTreeNode } from './useRoleTree';

import './RoleExplorerTree.scss';
import RolesStrings from '../../../../constants/pages/settings/security/RolesStrings';

interface RoleExplorerTreeProps {
  tree: SecurityTreeNode[];
  loading: boolean;
  onToggleExpand: (nodeId: string) => void;
  onExpandAll?: () => void;
  onCollapseAll?: () => void;
  onSearchChange?: (term: string) => void;
}

export function RoleExplorerTree({
  tree,
  loading,
  onToggleExpand,
  onExpandAll,
  onCollapseAll,
  onSearchChange
}: RoleExplorerTreeProps) {
  const [searchTerm, setSearchTerm] = useState('');

  const handleSearch = useCallback((term: string) => {
    setSearchTerm(term);
    onSearchChange?.(term);
  }, [onSearchChange]);

  if (loading) {
    return (
      <Flex align="center" justify="center" className="role-explorer-tree__loading">
        <Loader2 className="role-explorer-tree__spinner" size={24} />
        <Text size="2">{RolesStrings.TREE.BUILDING_SECURITY_TREE}</Text>
      </Flex>
    );
  }

  return (
    <Box className="role-explorer-tree-container">
      <Flex className="role-explorer-tree__toolbar" align="center" gap="3">
        <Box style={{ flex: 1 }}>
          <TextField.Root 
            placeholder={RolesStrings.TREE.SEARCH_TREE_NAME_PERMISSION_REPO_PLACEHOLDER} 
            size="2"
            value={searchTerm}
            onChange={(e) => handleSearch(e.target.value)}
          >
            <TextField.Slot>
              <Search size={16} />
            </TextField.Slot>
          </TextField.Root>
        </Box>

        <Separator orientation="vertical" size="1" />

        <Flex gap="2" align="center">
          <Tooltip content={RolesStrings.TREE.EXPAND_ALL}>
            <IconButton type="button" variant="ghost" size="2" onClick={onExpandAll}>
              <ChevronsDown size={16} />
            </IconButton>
          </Tooltip>
          <Tooltip content={RolesStrings.TREE.COLLAPSE_ALL}>
            <IconButton type="button" variant="ghost" size="2" onClick={onCollapseAll}>
              <ChevronsUp size={16} />
            </IconButton>
          </Tooltip>
        </Flex>

      </Flex>

      <Box className="role-explorer-tree__header">
        <Flex gap="4" p="2" className="role-explorer-tree__header-row">
          <Text size="1" weight="bold" className="role-explorer-tree__col-name">{RolesStrings.TREE.NAME}</Text>
          <Text size="1" weight="bold" className="role-explorer-tree__col-permissions">{RolesStrings.TREE.PERMISSIONS}</Text>
          <Text size="1" weight="bold" className="role-explorer-tree__col-resource">{RolesStrings.TREE.RESOURCE}</Text>
          <Text size="1" weight="bold" className="role-explorer-tree__col-description">{RolesStrings.TREE.DESCRIPTION}</Text>
        </Flex>
      </Box>

      <ScrollArea scrollbars="vertical" className="role-explorer-tree__scroll">
        {tree.length === 0 ? (
          <Box className="role-explorer-tree__empty">
            <Text size="2" color="gray">{RolesStrings.TREE.NO_MATCHING_SECURITY_NODES_FOUND}</Text>
          </Box>
        ) : (
          <Box role="tree" aria-label={RolesStrings.TREE.ROLE_SECURITY_EXPLORER_ARIA}>
            {tree.map((node) => (
              <TreeNode 
                key={node.id} 
                node={node} 
                onToggleExpand={onToggleExpand} 
                searchTerm={searchTerm}
              />
            ))}
          </Box>
        )}
      </ScrollArea>
    </Box>
  );
}

interface TreeNodeProps {
  node: SecurityTreeNode;
  onToggleExpand: (nodeId: string) => void;
  depth?: number;
  searchTerm?: string;
}

function TreeNode({ node, onToggleExpand, depth = 0, searchTerm }: TreeNodeProps) {
  const hasChildren = node.children && node.children.length > 0;
  const isExpanded = node.expanded;

  const getIcon = () => {
    switch (node.type) {
      case 'role': return <Shield size={16} className="role-explorer-tree__icon--role" />;
      case 'privilege': return <Key size={16} className="role-explorer-tree__icon--privilege" />;
      case 'content-selector': return <Filter size={16} className="role-explorer-tree__icon--selector" />;
      default: return null;
    }
  };

  const highlightText = (text: string | undefined) => {
    if (!(text && searchTerm)) return text;
    const parts = text.split(new RegExp(`(${searchTerm})`, 'gi'));
    return parts.map((part, i) => 
      part.toLowerCase() === searchTerm.toLowerCase() 
        ? <span key={i} className="role-explorer-tree__highlight">{part}</span> 
        : part
    );
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'ArrowRight' && hasChildren && !isExpanded) {
      onToggleExpand(node.id);
    } else if (e.key === 'ArrowLeft' && hasChildren && isExpanded) {
      onToggleExpand(node.id);
    }
  };

  return (
    <Box 
      className={`role-explorer-tree__node ${node.inherited ? 'role-explorer-tree__node--inherited' : ''}`}
    >
      <Flex 
        align="flex-start" 
        gap="4" 
        className="role-explorer-tree__node-content"
        role="treeitem"
        aria-expanded={hasChildren ? isExpanded : undefined}
        tabIndex={0}
        onKeyDown={handleKeyDown}
      >
        {/* Column 1: Name + Icon */}
        <Flex className="role-explorer-tree__col-name" style={{ paddingLeft: `${depth * 20}px` }} align="center" gap="2">
          <Box className="role-explorer-tree__expander">
            {hasChildren && (
              <IconButton 
                type="button"
                variant="ghost" 
                size="1" 
                onClick={(e) => { e.stopPropagation(); onToggleExpand(node.id); }}
                aria-label={isExpanded ? RolesStrings.TREE.COLLAPSE : RolesStrings.TREE.EXPAND}
              >
                {isExpanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
              </IconButton>
            )}
          </Box>

          <Box className="role-explorer-tree__type-icon">
            {getIcon()}
          </Box>

          <Flex align="center" gap="1">
            <Text size="2" weight="medium" className="role-explorer-tree__node-name">
              {highlightText(node.name)}
            </Text>
            {node.inherited && (
              <Text size="1" color="gray" className="role-explorer-tree__inherited-label">
                {RolesStrings.TREE.INHERITED}
              </Text>
            )}
            {node.isCircular && (
              <Tooltip content={RolesStrings.TREE.CIRCULAR_ROLE_REFERENCE_DETECTED_INHERITANCE}>
                <AlertTriangle size={14} className="role-explorer-tree__icon--warning" />
              </Tooltip>
            )}
          </Flex>
        </Flex>

        {/* Column 2: Permissions (Actions) */}
        <Box className="role-explorer-tree__col-permissions">
          {node.actions && (
            <Box className="role-explorer-tree__badge role-explorer-tree__badge--actions">
              <Text size="1">{highlightText(node.actions)}</Text>
            </Box>
          )}
        </Box>

        {/* Column 3: Resource (Repository) */}
        <Box className="role-explorer-tree__col-resource">
          {node.repository && (
            <Box className="role-explorer-tree__badge role-explorer-tree__badge--repo">
              <Text size="1">{highlightText(node.repository)}</Text>
            </Box>
          )}
          {node.type === 'content-selector' && node.csel && (
            <Tooltip content={node.csel}>
              <Box className="role-explorer-tree__badge role-explorer-tree__badge--csel">
                <Text size="1">{highlightText(node.csel)}</Text>
              </Box>
            </Tooltip>
          )}
        </Box>

        {/* Column 4: Description */}
        <Box className="role-explorer-tree__col-description">
          <Text size="1" color="gray" className="role-explorer-tree__description">
            {highlightText(node.description)}
          </Text>
        </Box>
      </Flex>

      {hasChildren && isExpanded && (
        <Box className="role-explorer-tree__children">
          {node.children!.map((child) => (
            <TreeNode 
              key={child.id} 
              node={child} 
              onToggleExpand={onToggleExpand} 
              depth={depth + 1} 
              searchTerm={searchTerm}
            />
          ))}
        </Box>
      )}
    </Box>
  );
}
