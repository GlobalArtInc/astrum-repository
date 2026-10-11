

import React from 'react';
import { Box, Flex, Text, Table, Badge, Tooltip } from '@radix-ui/themes';
import { Loader2, AlertCircle } from 'lucide-react';
import { useNodes } from './useNodes';

import './NodesList.scss';
import NodesStrings from '../../../../constants/pages/settings/system/NodesStrings';

/**
 * NodesList - Displays cluster nodes in a table.
 */
export function NodesList() {
  const { nodes, loading, error } = useNodes();

  if (loading) {
    return (
      <Flex align="center" justify="center" className="nodes-list__loading" aria-live="polite" aria-busy="true">
        <Loader2 size={24} className="nodes-list__spinner" aria-hidden="true" />
        <Text size="2">{NodesStrings.LIST.LOADING_NODES}</Text>
      </Flex>
    );
  }

  if (error) {
    return (
      <Box className="nodes-list__error">
        <AlertCircle size={20} aria-hidden="true" />
        <Text size="2">{error}</Text>
      </Box>
    );
  }

  return (
    <Box className="nodes-list">
      <Table.Root className="nodes-list__table" variant="surface">
        <Table.Header>
          <Table.Row>
            <Table.ColumnHeaderCell className="nodes-list__th">{NodesStrings.LIST.NODE_NAME}</Table.ColumnHeaderCell>
            <Table.ColumnHeaderCell className="nodes-list__th">{NodesStrings.LIST.NODE_IDENTITY}</Table.ColumnHeaderCell>
          </Table.Row>
        </Table.Header>
        <Table.Body>
          {nodes.length === 0 ? (
            <Table.Row>
              <Table.Cell colSpan={2} className="nodes-list__empty">
                {NodesStrings.LIST.NO_NODES_FOUND}
              </Table.Cell>
            </Table.Row>
          ) : (
            nodes.map((node) => (
              <Table.Row key={node.name} className="nodes-list__row">
                <Table.Cell className="nodes-list__cell">
                  <Flex align="center" gap="2">
                    <Text weight="medium">{node.displayName || node.name}</Text>
                    {node.local && (
                      <Tooltip content={NodesStrings.LIST.NODE_YOU_CURRENTLY_CONNECTED}>
                        <Badge color="gray" variant="soft" size="1">
                          {NodesStrings.LIST.CURRENT_NODE}
                        </Badge>
                      </Tooltip>
                    )}
                  </Flex>
                </Table.Cell>
                <Table.Cell className="nodes-list__cell">{node.name}</Table.Cell>
              </Table.Row>
            ))
          )}
        </Table.Body>
      </Table.Root>
    </Box>
  );
}

export default NodesList;
