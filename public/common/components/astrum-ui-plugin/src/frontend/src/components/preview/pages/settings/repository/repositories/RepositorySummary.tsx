

import React from 'react';
import { Box, Flex, Text, Badge, Tooltip } from '@radix-ui/themes';
import { ExternalLink, Globe, HardDrive, FolderSync } from 'lucide-react';
import { StatusBadge, FormatBadge } from '../../../../shared';
import { TypeBadge } from './TypeBadge';
import { Repository, RepositoryFormData } from './types';
import { BlobStoreStorageCard } from './BlobStoreStorageCard';
import { RepositoryMetricsCard } from './RepositoryMetricsCard';

import './RepositoryForm.scss';
import CommonStrings from '../../../../constants/CommonStrings';
import RepositoriesPageStrings from '../../../../constants/pages/settings/repository/RepositoriesPageStrings';

interface RepositorySummaryProps {
  repository: Repository;
  formData: RepositoryFormData;
  onNavigateToTab: (tab: string) => void;
  /** Hide blob store section for Cloud distribution where storage is managed internally */
  isCloud?: boolean;
}

export function RepositorySummary({ repository, formData, onNavigateToTab, isCloud = false }: RepositorySummaryProps) {
  const isOnline = repository.status?.online ?? repository.online ?? true;
  const blobStoreName = formData.storage?.blobStoreName || '-';
  const remoteUrl = formData.proxy?.remoteUrl;
  const memberCount = formData.group?.memberNames?.length || 0;

  return (
    <Box className="repository-summary">
      <Box className="repository-summary__grid">
        {/* Status Section */}
        <SummaryItem 
          label={CommonStrings.LABELS.STATUS} 
          value={
            <Flex align="center" gap="2">
              <StatusBadge
                status={isOnline ? 'online' : 'offline'}
                label={isOnline ? RepositoriesPageStrings.SUMMARY.ONLINE : RepositoriesPageStrings.SUMMARY.OFFLINE}
                size="small"
              />
              {repository.status?.description && (
                <Text size="2" color="gray">{repository.status.description}</Text>
              )}
            </Flex>
          }
        />

        {/* Format & Type */}
        <SummaryItem 
          label={CommonStrings.LABELS.FORMAT} 
          value={<FormatBadge format={repository.format} />} 
        />
        <SummaryItem 
          label={CommonStrings.LABELS.TYPE} 
          value={<TypeBadge type={repository.type as any} />} 
        />

        {/* URL Section */}
        <SummaryItem
          label={RepositoriesPageStrings.SUMMARY.URL}
          value={
            <Flex align="center" gap="2">
              <Text size="2" className="repository-summary__url">{repository.url + '/'}</Text>
              <Tooltip content={RepositoriesPageStrings.SUMMARY.OPEN_NEW_TAB}>
                <a href={repository.url + '/'} target="_blank" rel="noopener noreferrer" className="repository-summary__link">
                  <ExternalLink size={14} />
                </a>
              </Tooltip>
            </Flex>
          }
        />

        {/* Proxy-specific: Remote URL */}
        {remoteUrl && (
          <SummaryItem
            label={RepositoriesPageStrings.SUMMARY.REMOTE_STORAGE}
            value={
              <Flex align="center" gap="2">
                <Globe size={14} className="repository-summary__icon" />
                <Text size="2" className="repository-summary__url">{remoteUrl}</Text>
              </Flex>
            }
            onClick={() => onNavigateToTab('settings')}
          />
        )}

        {/* Storage Section - hidden in Cloud distribution */}
        {!isCloud && (
          <SummaryItem
            label={RepositoriesPageStrings.SUMMARY.BLOB_STORE}
            value={
              <Flex align="center" gap="2">
                <HardDrive size={14} className="repository-summary__icon" />
                <Text size="2">{blobStoreName}</Text>
              </Flex>
            }
            onClick={() => onNavigateToTab('settings')}
          />
        )}

        {/* Group-specific: Members */}
        {repository.type === 'group' && (
          <SummaryItem 
            label={RepositoriesPageStrings.SUMMARY.GROUP_MEMBERS} 
            value={
              <Flex align="center" gap="2">
                <FolderSync size={14} className="repository-summary__icon" />
                <Text size="2">{RepositoriesPageStrings.SUMMARY.MEMBER_REPOSITORIES(memberCount)}</Text>
              </Flex>
            }
            onClick={() => onNavigateToTab('settings')}
          />
        )}

        {/* Cleanup Policies */}
        {formData.cleanup?.policyNames && formData.cleanup.policyNames.length > 0 && (
          <SummaryItem
            label={RepositoriesPageStrings.SUMMARY.CLEANUP_POLICIES}
            value={
              <Flex gap="1" wrap="wrap">
                {formData.cleanup.policyNames.map(policy => (
                  <Badge key={policy} size="1" color="blue" variant="soft">{policy}</Badge>
                ))}
              </Flex>
            }
            onClick={() => onNavigateToTab('settings')}
          />
        )}
      </Box>

      {/* Blob Store Storage Card - hidden in Cloud distribution */}
      {!isCloud && blobStoreName && blobStoreName !== '-' && (
        <BlobStoreStorageCard blobStoreName={blobStoreName} />
      )}

      {/* Repository Metrics Card */}
      <RepositoryMetricsCard repositoryName={repository.name} />
    </Box>
  );
}

interface SummaryItemProps {
  label: string;
  value: React.ReactNode;
  onClick?: () => void;
}

function SummaryItem({ label, value, onClick }: SummaryItemProps) {
  return (
    <Box 
      className={`repository-summary__item ${onClick ? 'repository-summary__item--clickable' : ''}`}
      onClick={onClick}
    >
      <Text size="1" color="gray" className="repository-summary__label">{label}</Text>
      <Box className="repository-summary__value">
        {typeof value === 'string' ? <Text size="2">{value}</Text> : value}
      </Box>
    </Box>
  );
}
