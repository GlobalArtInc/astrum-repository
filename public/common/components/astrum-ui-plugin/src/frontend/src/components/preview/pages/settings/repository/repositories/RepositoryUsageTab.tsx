

import React from 'react';
import { Box, Flex, Text, Callout } from '@radix-ui/themes';
import { Info, FolderTree } from 'lucide-react';
import { LoadingState, ErrorState, EntityTable } from '../../../../shared';
import { useRepositoryTree } from './useRepositoryTree';

export interface RepositoryUsageTabProps {
  repositoryName: string;
}

/**
 * RepositoryUsageTab displays which group repositories include this repository as a member.
 */
export function RepositoryUsageTab({ repositoryName }: RepositoryUsageTabProps) {
  const { usages, loading, error, refresh } = useRepositoryTree(repositoryName);

  if (loading) return <LoadingState message="Checking group membership..." />;
  if (error) return <ErrorState message={error} onRetry={refresh} />;

  const columns = [
    {
      id: 'name',
      header: 'Group Repository',
      accessor: (name: string) => (
        <Flex align="center" gap="2">
          <FolderTree size={16} color="var(--blue-9)" />
          <Text weight="medium">{name}</Text>
        </Flex>
      ),
      sortable: true,
    }
  ];

  return (
    <Box p="4" className="repository-usage-tab">
      <Callout.Root color="blue" mb="4" size="1">
        <Callout.Icon>
          <Info size={16} />
        </Callout.Icon>
        <Callout.Text>
          Showing group repositories that include <strong>{repositoryName}</strong> as a member.
        </Callout.Text>
      </Callout.Root>

      {usages.length > 0 ? (
        <EntityTable 
          data={usages}
          columns={columns}
          getRowKey={(name) => name}
          testId="repository-usage-table"
        />
      ) : (
        <Flex 
          direction="column" 
          align="center" 
          justify="center" 
          p="6" 
          style={{ background: 'var(--gray-2)', borderRadius: 'var(--radius-3)' }}
        >
          <Text color="gray" size="2">This repository is not a member of any groups.</Text>
        </Flex>
      )}
    </Box>
  );
}

export default RepositoryUsageTab;
