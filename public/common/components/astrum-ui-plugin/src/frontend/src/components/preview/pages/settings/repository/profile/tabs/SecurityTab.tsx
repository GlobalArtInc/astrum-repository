

import React from 'react';
import { Box, Flex, Text, Badge } from '@radix-ui/themes';
import {
  Shield,
  Heart,
  AlertTriangle,
  CheckCircle,
  ExternalLink,
} from 'lucide-react';
import type {
  RepositoryProfileData,
  HealthCheckData,
  FirewallData,
} from '../types';
import RepositoryProfileStrings from '../../../../../constants/pages/settings/repository/RepositoryProfileStrings';
import CommonStrings from '../../../../../constants/CommonStrings';

// =============================================================================
// Types
// =============================================================================

interface SecurityTabProps {
  repository: RepositoryProfileData;
  healthCheck: HealthCheckData | null;
  firewall: FirewallData | null;
}

// =============================================================================
// Component
// =============================================================================

/**
 * SecurityTab - Displays security status including Firewall and Health Check
 */
export function SecurityTab({
  repository,
  healthCheck,
  firewall,
}: SecurityTabProps): JSX.Element {
  const hasSecurityData = healthCheck || firewall;

  if (!hasSecurityData) {
    return (
      <Box className="profile-empty-state">
        <Shield size={64} className="profile-empty-state__icon" />
        <Text className="profile-empty-state__title">{RepositoryProfileStrings.SECURITY_TAB.SECURITY_FEATURES_NOT_CONFIGURED}</Text>
        <Text className="profile-empty-state__message">
          {RepositoryProfileStrings.SECURITY_TAB.REPOSITORY_HEALTH_CHECK_FIREWALL_NOT}
        </Text>
      </Box>
    );
  }

  return (
    <Box className="profile-section__grid">
      {/* Firewall Status */}
      <Box className="status-card">
        <Box className="status-card__header">
          <Flex align="center" gap="2" className="status-card__title">
            <Shield
              size={18}
              className={firewall?.enabled ? 'status-card__icon--enabled' : 'status-card__icon--disabled'}
            />
            {RepositoryProfileStrings.SECURITY_TAB.REPOSITORY_FIREWALL}
          </Flex>
          <Badge color={firewall?.enabled ? 'green' : 'gray'}>
            {firewall?.enabled ? RepositoryProfileStrings.SECURITY_TAB.ENABLED : RepositoryProfileStrings.SECURITY_TAB.DISABLED}
          </Badge>
        </Box>

        {firewall?.enabled ? (
          <Box className="status-card__content">
            <Box className="status-card__row">
              <Text className="status-card__label">{RepositoryProfileStrings.SECURITY_TAB.QUARANTINE_POLICY}</Text>
              <Text className="status-card__value">{firewall.quarantinePolicy || '—'}</Text>
            </Box>

            <Box className="status-card__row">
              <Text className="status-card__label">{RepositoryProfileStrings.SECURITY_TAB.POLICY_VIOLATIONS}</Text>
              <Text className="status-card__value">
                {firewall.policyViolations ?? 0}
                {(firewall.criticalCount ?? 0) > 0 && (
                  <Badge color="red" ml="2" size="1">
                    {RepositoryProfileStrings.SECURITY_TAB.CRITICAL(firewall.criticalCount)}
                  </Badge>
                )}
              </Text>
            </Box>

          </Box>
        ) : (
          <Box className="profile-empty-state" style={{ padding: '20px' }}>
            <Text color="gray" size="2">
              {RepositoryProfileStrings.SECURITY_TAB.FIREWALL_NOT_ENABLED_REPOSITORY_CONFIGURE}
            </Text>
          </Box>
        )}
      </Box>

      {/* Health Check Status */}
      <Box className="status-card">
        <Box className="status-card__header">
          <Flex align="center" gap="2" className="status-card__title">
            <Heart
              size={18}
              className={healthCheck?.enabled ? 'status-card__icon--enabled' : 'status-card__icon--disabled'}
            />
            {RepositoryProfileStrings.SECURITY_TAB.REPOSITORY_HEALTH_CHECK}
          </Flex>
          <Badge color={healthCheck?.enabled ? 'green' : 'gray'}>
            {healthCheck?.enabled ? (healthCheck.analyzing ? RepositoryProfileStrings.SECURITY_TAB.ANALYZING : RepositoryProfileStrings.SECURITY_TAB.ENABLED) : RepositoryProfileStrings.SECURITY_TAB.DISABLED}
          </Badge>
        </Box>

        {healthCheck?.enabled ? (
          <Box className="status-card__content">
            {healthCheck.analyzing ? (
              <Box className="status-card__row">
                <Text className="status-card__label">{CommonStrings.LABELS.STATUS}</Text>
                <Flex align="center" gap="2">
                  <Text className="status-card__value">{RepositoryProfileStrings.SECURITY_TAB.ANALYZING_COMPONENTS}</Text>
                </Flex>
              </Box>
            ) : (
              <>
                <Box className="status-card__row">
                  <Text className="status-card__label">{RepositoryProfileStrings.SECURITY_TAB.SECURITY_ISSUES}</Text>
                  <Flex align="center" gap="1">
                    {(healthCheck.securityIssueCount ?? 0) > 0 ? (
                      <>
                        <AlertTriangle size={14} color="var(--yellow-11)" />
                        <Text className="status-card__value">{healthCheck.securityIssueCount}</Text>
                      </>
                    ) : (
                      <>
                        <CheckCircle size={14} color="var(--green-11)" />
                        <Text className="status-card__value">{RepositoryProfileStrings.SECURITY_TAB.NONE}</Text>
                      </>
                    )}
                  </Flex>
                </Box>

                <Box className="status-card__row">
                  <Text className="status-card__label">{RepositoryProfileStrings.SECURITY_TAB.LICENSE_ISSUES}</Text>
                  <Flex align="center" gap="1">
                    {(healthCheck.licenseIssueCount ?? 0) > 0 ? (
                      <>
                        <AlertTriangle size={14} color="var(--yellow-11)" />
                        <Text className="status-card__value">{healthCheck.licenseIssueCount}</Text>
                      </>
                    ) : (
                      <>
                        <CheckCircle size={14} color="var(--green-11)" />
                        <Text className="status-card__value">{RepositoryProfileStrings.SECURITY_TAB.NONE}</Text>
                      </>
                    )}
                  </Flex>
                </Box>

                {healthCheck.detailUrl && (
                  <Box className="status-card__row">
                    <Text className="status-card__label">{RepositoryProfileStrings.SECURITY_TAB.REPORT}</Text>
                    <a
                      href={healthCheck.detailUrl}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="status-card__link"
                    >
                      <ExternalLink size={14} />
                      {RepositoryProfileStrings.SECURITY_TAB.VIEW_DETAILED_REPORT}
                    </a>
                  </Box>
                )}
              </>
            )}
          </Box>
        ) : (
          <Box className="profile-empty-state" style={{ padding: '20px' }}>
            <Text color="gray" size="2">
              {RepositoryProfileStrings.SECURITY_TAB.HEALTH_CHECK_NOT_ENABLED_REPOSITORY}
            </Text>
          </Box>
        )}
      </Box>
    </Box>
  );
}

export default SecurityTab;


