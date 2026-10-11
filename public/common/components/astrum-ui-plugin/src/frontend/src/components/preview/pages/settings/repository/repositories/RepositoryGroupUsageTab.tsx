

import React from 'react';
import { Box, Flex, Text, Callout, Tooltip, IconButton } from '@radix-ui/themes';
import { Info, FolderTree, ExternalLink } from 'lucide-react';
import { LoadingState, ErrorState, EntityTable } from '../../../../shared';
import { useRepositoryTree } from './useRepositoryTree';
import RepositoriesPageStrings from '../../../../constants/pages/settings/repository/RepositoriesPageStrings';

export interface RepositoryGroupUsageTabProps {
  repositoryName: string;
}

/**
 * RepositoryGroupUsageTab displays which group repositories include this repository as a member.
 */
export function RepositoryGroupUsageTab({ repositoryName }: RepositoryGroupUsageTabProps) {
  const { usages, loading, error, refresh } = useRepositoryTree(repositoryName);

  if (loading) return <LoadingState message={RepositoriesPageStrings.GROUP_USAGE.CHECKING_GROUP_MEMBERSHIP} />;
  if (error) return <ErrorState message={error} onRetry={refresh} />;

  const columns = [
    {
      id: 'name',
      header: RepositoriesPageStrings.GROUP_USAGE.GROUP_REPOSITORY,
      accessor: (name: string) => (
        <Flex align="center" justify="between" style={{ width: '100%' }}>
          <Flex align="center" gap="2">
            <FolderTree size={16} color="var(--blue-9)" />
            <Text weight="medium">{name}</Text>
          </Flex>
          <Tooltip content={RepositoriesPageStrings.GROUP_USAGE.VIEW_STRUCTURE}>
            <IconButton 
              variant="ghost" 
              size="1" 
              onClick={() => window.location.hash = `admin/repository/repositories/${encodeURIComponent(name)}/profile?tab=structure`}
            >
              <ExternalLink size={14} />
            </IconButton>
          </Tooltip>
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
          {RepositoriesPageStrings.GROUP_USAGE.SHOWING_GROUPS(<strong>{repositoryName}</strong>)}
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
          <Text color="gray" size="2">{RepositoriesPageStrings.GROUP_USAGE.REPOSITORY_NOT_MEMBER_ANY_GROUPS}</Text>
        </Flex>
      )}
    </Box>
  );
}

export default RepositoryGroupUsageTab;
