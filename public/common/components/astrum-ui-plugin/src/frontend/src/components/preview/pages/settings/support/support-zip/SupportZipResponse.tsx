

import React from 'react';
import { Box, Flex, Text, Heading } from '@radix-ui/themes';
import { CheckCircle2, Download, AlertTriangle } from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';

import { SettingsButton, SettingsAlert } from '../../../../shared/form';
import { SupportZipResponse as SupportZipResponseType } from './types';
import { useSupportZipApi } from './useSupportZipApi';

import './SupportZipResponse.scss';
import SupportZipStrings from '../../../../constants/pages/settings/support/SupportZipStrings';
import CommonStrings from '../../../../constants/CommonStrings';

interface SupportZipResponseProps {
  response: SupportZipResponseType;
  nodeId?: string;
}

/**
 * SupportZipResponse - Displays the result of creating a support ZIP
 */
export function SupportZipResponse({ response, nodeId }: SupportZipResponseProps) {
  const { getDownloadUrl } = useSupportZipApi();

  const handleDownload = () => {
    const url = ExtJS.urlOf(getDownloadUrl(response.name));
    ExtJS.downloadUrl(url);
  };

  return (
    <Box className="support-zip-response" data-testid="support-zip-response">
      <Flex align="center" gap="3" mb="4">
        <CheckCircle2 size={24} className="support-zip-response__icon" />
        <Heading as="h3" size="4" weight="medium">
          {SupportZipStrings.RESPONSE.SUPPORT_ZIP_CREATED}
        </Heading>
      </Flex>

      {response.truncated && (
        <Box mb="4">
          <SettingsAlert type="warning" data-testid="support-zip-truncated-warning">
            <Flex align="center" gap="2">
              <AlertTriangle size={16} />
              <Text>{SupportZipStrings.RESPONSE.SUPPORT_ZIP_WAS_TRUNCATED_DUE}</Text>
            </Flex>
          </SettingsAlert>
        </Box>
      )}

      <Box className="support-zip-response__details">
        <Text size="2" color="gray" mb="2">
          {SupportZipStrings.RESPONSE.SUPPORT_ZIP_HAS_BEEN_CREATED}
        </Text>

        <Box className="support-zip-response__info" mt="4">
          {nodeId && (
            <Flex className="support-zip-response__row">
              <Text size="2" weight="medium" className="support-zip-response__label">
                {SupportZipStrings.RESPONSE.NODE_LABEL}
              </Text>
              <Text size="2" className="support-zip-response__value" data-testid="support-zip-response-node">
                {nodeId}
              </Text>
            </Flex>
          )}
          <Flex className="support-zip-response__row">
            <Text size="2" weight="medium" className="support-zip-response__label">
              {SupportZipStrings.RESPONSE.NAME_LABEL}
            </Text>
            <Text size="2" className="support-zip-response__value" data-testid="support-zip-response-name">
              {response.name}
            </Text>
          </Flex>
          <Flex className="support-zip-response__row">
            <Text size="2" weight="medium" className="support-zip-response__label">
              {SupportZipStrings.RESPONSE.SIZE_LABEL}
            </Text>
            <Text size="2" className="support-zip-response__value" data-testid="support-zip-response-size">
              {response.size}
            </Text>
          </Flex>
          <Flex className="support-zip-response__row">
            <Text size="2" weight="medium" className="support-zip-response__label">
              {SupportZipStrings.RESPONSE.PATH_LABEL}
            </Text>
            <Text size="2" className="support-zip-response__value support-zip-response__value--path" data-testid="support-zip-response-path">
              {response.file}
            </Text>
          </Flex>
        </Box>

        <Box mt="4">
          <SettingsButton
            variant="primary"
            onClick={handleDownload}
            icon={Download}
            data-testid="support-zip-download-button"
          >
            {CommonStrings.ACTIONS.DOWNLOAD}
          </SettingsButton>
        </Box>
      </Box>
    </Box>
  );
}

export default SupportZipResponse;


