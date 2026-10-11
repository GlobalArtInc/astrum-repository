import React from 'react';
import { Badge, Flex } from '@radix-ui/themes';
import { Circle } from 'lucide-react';

import { FORMAT_LABELS, TYPE_LABELS, RepositoryType } from './types';
import RepositoriesPageStrings from '../../../../constants/pages/settings/repository/RepositoriesPageStrings';

export interface RepositoryChipsProps {
  format: string;
  type: string;
  online: boolean;
}

/**
 * The status, type and format of a repository as chips, shown under the repository name in the page headers.
 */
export function RepositoryChips({ format, type, online }: RepositoryChipsProps): JSX.Element {
  const dotColor = online ? 'var(--green-9)' : 'var(--gray-9)';
  return (
    <>
      <Badge variant="soft" color="gray" size="2" data-testid="repository-chip-status">
        <Flex align="center" gap="1">
          <Circle size={8} fill={dotColor} color={dotColor} aria-hidden="true" />
          {online ? RepositoriesPageStrings.PAGE.ONLINE : RepositoriesPageStrings.PAGE.OFFLINE}
        </Flex>
      </Badge>
      <Badge variant="soft" color="gray" size="2" data-testid="repository-chip-type">
        {TYPE_LABELS[type as RepositoryType] || type}
      </Badge>
      <Badge variant="soft" color="gray" size="2" data-testid="repository-chip-format">
        {FORMAT_LABELS[format] || format}
      </Badge>
    </>
  );
}

export default RepositoryChips;
