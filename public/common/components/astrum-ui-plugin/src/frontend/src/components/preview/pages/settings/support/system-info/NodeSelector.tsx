

import React from 'react';
import { Box, Flex, Text } from '@radix-ui/themes';
import { Server, Check } from 'lucide-react';

import { NodeSelectorProps } from './types';

import './NodeSelector.scss';

/**
 * NodeSelector - HA node selection component for multi-node clusters
 */
export function NodeSelector({
  nodes,
  selectedNode,
  onNodeSelect,
  className = '',
}: NodeSelectorProps) {
  if (nodes.length <= 1) {
    return null;
  }

  return (
    <Box className={`node-selector ${className}`.trim()}>
      <Flex align="center" gap="2" className="node-selector__header">
        <Server size={16} className="node-selector__icon" />
        <Text size="2" weight="medium">Select Node</Text>
      </Flex>
      
      <Flex gap="2" wrap="wrap" className="node-selector__nodes">
        {nodes.map((node) => {
          const isSelected = node.nodeId === selectedNode;
          const displayName = node.friendlyName || node.hostname || node.nodeId;
          
          return (
            <button
              key={node.nodeId}
              type="button"
              className={`node-selector__node ${isSelected ? 'node-selector__node--selected' : ''}`}
              onClick={() => onNodeSelect(node.nodeId)}
              aria-pressed={isSelected}
            >
              <Flex align="center" gap="2">
                {isSelected && <Check size={14} className="node-selector__check" />}
                <Text size="2">{displayName}</Text>
                {node.local && (
                  <Text size="1" className="node-selector__local-badge">
                    local
                  </Text>
                )}
              </Flex>
            </button>
          );
        })}
      </Flex>
    </Box>
  );
}

export default NodeSelector;


