

import React from 'react';
import { Box, Flex, Text, } from '@radix-ui/themes';
import { Loader2, AlertTriangle, Trash2 } from 'lucide-react';

import {
  SettingsForm,
  SettingsFormSection,
  SettingsButton,
  SettingsAlert,
} from '../../../../shared/form';
import {
  SslCertificatesDetailProps,
  formatDate,
  isCertificateExpired,
  isCertificateExpiring,
} from './types';

import './SslCertificatesDetail.scss';
import SslCertificatesStrings from '../../../../constants/pages/settings/security/SslCertificatesStrings';
import CommonStrings from '../../../../constants/CommonStrings';

/**
 * SslCertificatesDetail - Detailed view of a single SSL certificate
 */
export function SslCertificatesDetail({
  certificate,
  loading,
  canDelete,
  onDelete,
  onCancel,
  error,
  showTrustWarning = false,
}: SslCertificatesDetailProps) {
  // Loading state
  if (loading) {
    return (
      <Flex align="center" justify="center" className="ssl-certificates-detail__loading">
        <Loader2 size={24} className="ssl-certificates-detail__spinner" />
        <Text size="2">{SslCertificatesStrings.DETAIL.LOADING_CERTIFICATE_DETAILS}</Text>
      </Flex>
    );
  }

  // No certificate found
  if (!certificate) {
    return (
      <Box className="ssl-certificates-detail__not-found">
        <AlertTriangle size={24} />
        <Text size="2">{SslCertificatesStrings.DETAIL.CERTIFICATE_NOT_FOUND}</Text>
      </Box>
    );
  }

  const isExpired = isCertificateExpired(certificate);
  const isExpiring = !isExpired && isCertificateExpiring(certificate);
  const issuedDate = formatDate(certificate.issuedOn);
  const expiresDate = formatDate(certificate.expiresOn);

  const deleteButton = canDelete ? (
    <SettingsButton variant="danger" onClick={onDelete} icon={Trash2}>
      {SslCertificatesStrings.DETAIL.DELETE_CERTIFICATE}
    </SettingsButton>
  ) : undefined;

  return (
    <Box className="ssl-certificates-detail">
      <SettingsForm
        showHeader={false}
        showActions={true}
        onCancel={onCancel}
        cancelLabel={CommonStrings.ACTIONS.BACK_TO_LIST}
        footerExtra={deleteButton}
        noDirtyTracking
      >
        {/* Error Alert */}
        {error && (
          <Box className="ssl-certificates-detail__alerts">
            <SettingsAlert type="error" onClose={() => {}}>
              {error}
            </SettingsAlert>
          </Box>
        )}

        {/* Expiration Warnings */}
        {isExpired && (
          <Box className="ssl-certificates-detail__alerts">
            <SettingsAlert type="error">
              <Flex align="center" gap="2">
                <AlertTriangle size={16} />
                <Text size="2">{SslCertificatesStrings.DETAIL.CERTIFICATE_HAS_EXPIRED}</Text>
              </Flex>
            </SettingsAlert>
          </Box>
        )}
        {isExpiring && (
          <Box className="ssl-certificates-detail__alerts">
            <SettingsAlert type="warning">
              <Flex align="center" gap="2">
                <AlertTriangle size={16} />
                <Text size="2">{SslCertificatesStrings.DETAIL.CERTIFICATE_EXPIRING_SOON}</Text>
              </Flex>
            </SettingsAlert>
          </Box>
        )}

        {/* Certificate Information */}
        <SettingsFormSection title={SslCertificatesStrings.DETAIL.CERTIFICATE_INFORMATION} defaultOpen>
          <Box className="ssl-certificates-detail__info">
            <Flex className="ssl-certificates-detail__row">
              <Text size="2" weight="medium" className="ssl-certificates-detail__label">{SslCertificatesStrings.DETAIL.FINGERPRINT}</Text>
              <Text size="1" className="ssl-certificates-detail__fingerprint">{certificate.fingerprint}</Text>
            </Flex>
            <Flex className="ssl-certificates-detail__row">
              <Text size="2" weight="medium" className="ssl-certificates-detail__label">{SslCertificatesStrings.DETAIL.VALID_UNTIL}</Text>
              <Text size="2">{expiresDate || 'N/A'}</Text>
            </Flex>
            <Flex className="ssl-certificates-detail__row">
              <Text size="2" weight="medium" className="ssl-certificates-detail__label">{SslCertificatesStrings.DETAIL.ISSUED_ON}</Text>
              <Text size="2">{issuedDate || 'N/A'}</Text>
            </Flex>
          </Box>
        </SettingsFormSection>

        {/* Subject Information */}
        <SettingsFormSection title={SslCertificatesStrings.DETAIL.SUBJECT} defaultOpen>
          <Box className="ssl-certificates-detail__info">
            <Flex className="ssl-certificates-detail__row">
              <Text size="2" weight="medium" className="ssl-certificates-detail__label">{SslCertificatesStrings.DETAIL.COMMON_NAME}</Text>
              <Text size="2">{certificate.subjectCommonName || 'N/A'}</Text>
            </Flex>
            <Flex className="ssl-certificates-detail__row">
              <Text size="2" weight="medium" className="ssl-certificates-detail__label">{SslCertificatesStrings.DETAIL.ORGANIZATION}</Text>
              <Text size="2">{certificate.subjectOrganization || 'N/A'}</Text>
            </Flex>
            {certificate.subjectOrganizationalUnit && (
              <Flex className="ssl-certificates-detail__row">
                <Text size="2" weight="medium" className="ssl-certificates-detail__label">{SslCertificatesStrings.DETAIL.ORGANIZATIONAL_UNIT}</Text>
                <Text size="2">{certificate.subjectOrganizationalUnit}</Text>
              </Flex>
            )}
          </Box>
        </SettingsFormSection>

        {/* Issuer Information */}
        <SettingsFormSection title={SslCertificatesStrings.DETAIL.ISSUER} defaultOpen>
          <Box className="ssl-certificates-detail__info">
            <Flex className="ssl-certificates-detail__row">
              <Text size="2" weight="medium" className="ssl-certificates-detail__label">{SslCertificatesStrings.DETAIL.COMMON_NAME}</Text>
              <Text size="2">{certificate.issuerCommonName || 'N/A'}</Text>
            </Flex>
            <Flex className="ssl-certificates-detail__row">
              <Text size="2" weight="medium" className="ssl-certificates-detail__label">{SslCertificatesStrings.DETAIL.ORGANIZATION}</Text>
              <Text size="2">{certificate.issuerOrganization || 'N/A'}</Text>
            </Flex>
            {certificate.issuerOrganizationalUnit && (
              <Flex className="ssl-certificates-detail__row">
                <Text size="2" weight="medium" className="ssl-certificates-detail__label">{SslCertificatesStrings.DETAIL.ORGANIZATIONAL_UNIT}</Text>
                <Text size="2">{certificate.issuerOrganizationalUnit}</Text>
              </Flex>
            )}
          </Box>
        </SettingsFormSection>

        {/* Warning */}
        {showTrustWarning && (
          <Box className="ssl-certificates-detail__warning">
            <SettingsAlert type="warning">
              <Text size="2">
                {SslCertificatesStrings.DETAIL.WARNING_ADDING_CERTIFICATE_TRUST_STORE}
              </Text>
            </SettingsAlert>
          </Box>
        )}
      </SettingsForm>
    </Box>
  );
}

export default SslCertificatesDetail;
