

import React from 'react';
import { Box, Card, Flex, Text, Heading, Button } from '@radix-ui/themes';
import { Server, Loader2, AlertCircle, CheckCircle, Archive } from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';

import { NodeInfo, SUPPORT_ZIP_API } from './types';
import SupportZipStrings from '../../../../constants/pages/settings/support/SupportZipStrings';
import CommonStrings from '../../../../constants/CommonStrings';
import { pageLanguage } from '../../../../../../interface/plural';

interface SupportZipNodeCardProps {
  node: NodeInfo;
  isBlobStoreConfigured: boolean;
  onGenerate: (node: NodeInfo) => void;
  disabled?: boolean;
}

function formatTimestamp(value?: number): string | null {
  if (!value) return null;
  try {
    return new Date(value).toLocaleString(pageLanguage());
  } catch {
    return null;
  }
}

export function SupportZipNodeCard({
  node,
  isBlobStoreConfigured,
  onGenerate,
  disabled = false,
}: SupportZipNodeCardProps): React.ReactElement {
  const isNodeActive = node.status !== 'NODE_UNAVAILABLE';
  const zipCreating = node.status === 'CREATING';
  const generateDisabled = disabled || !isNodeActive || !isBlobStoreConfigured || zipCreating;
  const lastUpdated = formatTimestamp(node.lastUpdated);

  const renderHeader = () => {
    const StatusIcon = isNodeActive ? CheckCircle : AlertCircle;
    const iconColor = isNodeActive ? 'var(--green-9)' : 'var(--red-9)';
    return (
      <Flex align="center" gap="2">
        <Server size={18} aria-hidden="true" />
        <Heading as="h4" size="3" weight="medium" style={{ flex: 1 }}>
          {node.hostname || node.nodeId}
        </Heading>
        <StatusIcon size={16} aria-hidden="true" style={{ color: iconColor }} />
      </Flex>
    );
  };

  const renderBody = () => {
    if (!isNodeActive) {
      return (
        <Text size="2" color="gray">
          {SupportZipStrings.NODE_CARD.NODE_UNAVAILABLE_CANNOT_CREATE_ZIP}
        </Text>
      );
    }
    if (!isBlobStoreConfigured) {
      return (
        <Text size="2" color="gray">
          {SupportZipStrings.NODE_CARD.NO_BLOB_STORE_CONFIGURED_CLUSTER}
        </Text>
      );
    }
    switch (node.status) {
      case 'CREATING':
        return (
          <Flex align="center" gap="2" aria-live="polite" aria-busy="true">
            <Loader2 size={16} aria-hidden="true" />
            <Text size="2">{SupportZipStrings.NODE_CARD.CREATING_ZIP}</Text>
          </Flex>
        );
      case 'COMPLETED':
        return (
          <Flex direction="column" gap="1">
            <a
              href={ExtJS.urlOf(SUPPORT_ZIP_API.DOWNLOAD(node.blobRef ?? ''))}
              download
              aria-label={SupportZipStrings.NODE_CARD.DOWNLOAD_ZIP(node.hostname || node.nodeId)}
              data-testid={`support-zip-node-card-download-${node.nodeId}`}
            >
              {SupportZipStrings.NODE_CARD.DOWNLOAD_ZIP_2}
            </a>
            {lastUpdated && (
              <Text size="1" color="gray">
                {SupportZipStrings.NODE_CARD.GENERATED(lastUpdated)}
              </Text>
            )}
          </Flex>
        );
      case 'FAILED':
        return (
          <Flex align="center" gap="2">
            <AlertCircle size={16} aria-hidden="true" style={{ color: 'var(--red-9)' }} />
            <Text size="2" color="red">
              {SupportZipStrings.NODE_CARD.GENERATION_FAILED}
            </Text>
          </Flex>
        );
      default:
        return (
          <Text size="2" color="gray">
            {SupportZipStrings.NODE_CARD.NO_ZIP_CREATED_YET}
          </Text>
        );
    }
  };

  const buttonLabel = node.status === 'FAILED' ? CommonStrings.ACTIONS.RETRY : SupportZipStrings.NODE_CARD.GENERATE_NEW_ZIP;

  return (
    <Card data-testid={`support-zip-node-card-${node.nodeId}`}>
      <Flex direction="column" gap="3">
        {renderHeader()}
        <Box>{renderBody()}</Box>
        <Flex justify="end">
          <Button
            type="button"
            variant="solid"
            disabled={generateDisabled}
            onClick={() => onGenerate(node)}
            data-testid={`support-zip-node-card-generate-${node.nodeId}`}
            data-analytics-id="as-support-zip-create-node"
          >
            <Archive size={16} aria-hidden="true" />
            {buttonLabel}
          </Button>
        </Flex>
      </Flex>
    </Card>
  );
}

export default SupportZipNodeCard;
