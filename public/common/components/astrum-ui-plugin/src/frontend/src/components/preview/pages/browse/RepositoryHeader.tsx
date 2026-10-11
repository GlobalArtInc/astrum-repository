import React, { useCallback } from 'react';
import { Badge, Box, Button, Flex, Heading, Skeleton, Text, Tooltip } from '@radix-ui/themes';
import { Copy, Settings } from 'lucide-react';

import { ExtJS } from '../../../../interface/ExtJS';
import UIStrings from '../../constants/UIStrings';
import { useToast } from '../../shared';
import { copyToClipboard } from '../../shared/Usage';
import { FormatBadge, StatusLabel } from './shared';
import { repositoryUrlOf, type RepositoryInfo } from './useRepositoryInfo';

import './RepositoryHeader.scss';

const TYPE_KEYS: Record<string, 'HOSTED' | 'PROXY' | 'GROUP'> = {
  hosted: 'HOSTED',
  proxy: 'PROXY',
  group: 'GROUP',
};

/**
 * Whether the user may open the settings of the repository. A grant on this one repository counts as
 * well as one on every repository.
 */
export function canOpenRepositorySettings(repository: Pick<RepositoryInfo, 'name' | 'format'>): boolean {
  return (
    ExtJS.checkPermission(`astrum:repository-admin:${repository.format}:${repository.name}:read`) ||
    ExtJS.checkPermission('astrum:repository-admin:*:*:read')
  );
}

export interface RepositoryHeaderProps {
  /** Name of the open repository */
  repositoryName: string;
  /** What is known about it; null while it loads */
  repository: RepositoryInfo | null;
  /** Back to the list of repositories */
  onBackToList: () => void;
  /** Open the settings of the repository */
  onOpenSettings: (repositoryName: string) => void;
  /** Extra controls shown before the actions (e.g. the in-repository search) */
  children?: React.ReactNode;
  /** The repository does not exist or may not be browsed: no details and no actions */
  missing?: boolean;
}

/**
 * Head of a repository page: the breadcrumb back to the repositories, the name with its format, type
 * and status, and the actions "Copy URL" and, with permission, "Repository settings".
 */
export function RepositoryHeader({
  repositoryName,
  repository,
  onBackToList,
  onOpenSettings,
  children,
  missing = false,
}: RepositoryHeaderProps): JSX.Element {
  const strings = UIStrings.BROWSE.REPOSITORY;
  const toast = useToast();
  const url = repositoryUrlOf(repositoryName, repository?.url);
  const typeKey = repository ? TYPE_KEYS[repository.type] : undefined;
  const showSettings = repository ? canOpenRepositorySettings(repository) : false;

  const handleCopyUrl = useCallback(async () => {
    if (await copyToClipboard(url)) {
      toast.success(strings.URL_COPIED);
    } else {
      toast.error(UIStrings.BROWSE.LIST.URL_COPY_FAILED);
    }
  }, [url, toast, strings]);

  return (
    <Flex direction="column" gap="3" className="repository-header" data-testid="repository-header">
      <nav aria-label={strings.BREADCRUMB_LABEL} className="repository-header__breadcrumb">
        <Text size="2" color="gray">
          <button type="button" className="repository-header__crumb" onClick={onBackToList}>
            {strings.BREADCRUMB}
          </button>
          <span aria-hidden="true" className="repository-header__separator">/</span>
          <span aria-current="page">{repositoryName}</span>
        </Text>
      </nav>
      <Flex align="center" justify="between" gap="4" wrap="wrap">
        <Flex align="center" gap="3" wrap="wrap" minWidth="0">
          <Heading as="h1" size="7" className="repository-header__title">{repositoryName}</Heading>
          {repository ? (
            <Flex align="center" gap="2" wrap="wrap">
              <FormatBadge format={repository.format} size="2" radius="full" />
              {typeKey && (
                <Badge color="gray" variant="soft" size="2" radius="full" className="repository-header__type">
                  {UIStrings.BROWSE.TYPES[typeKey]}
                </Badge>
              )}
              <StatusLabel
                online={repository.status?.online ?? true}
                description={repository.status?.description || repository.status?.reason}
              />
            </Flex>
          ) : !missing && (
            <Skeleton width="160px" height="22px" />
          )}
        </Flex>
        {!missing && (
        <Flex align="center" gap="2" wrap={{ initial: 'wrap', md: 'nowrap' }} className="repository-header__actions">
          {children && <Box className="repository-header__extra">{children}</Box>}
          <Tooltip content={url}>
            <Button variant="surface" color="gray" highContrast size="2" onClick={handleCopyUrl}>
              <Copy size={14} aria-hidden />
              {strings.COPY_URL}
            </Button>
          </Tooltip>
          {showSettings && (
            <Button
              variant="surface"
              color="gray"
              highContrast
              size="2"
              onClick={() => onOpenSettings(repositoryName)}
            >
              <Settings size={14} aria-hidden />
              {strings.SETTINGS}
            </Button>
          )}
        </Flex>
        )}
      </Flex>
    </Flex>
  );
}

export default RepositoryHeader;
