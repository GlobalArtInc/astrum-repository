

import React from 'react';
import { Box, Flex, Text, Heading, Badge } from '@radix-ui/themes';
import { CheckCircle, XCircle, HelpCircle, Clock } from 'lucide-react';

import { MetricHealthDetailProps, formatCheckName, getHealthStatus } from './types';

import './MetricHealthDetail.scss';
import MetricHealthStrings from '../../../../constants/pages/settings/support/MetricHealthStrings';
import { pageLanguage } from '../../../../../../interface/plural';

/**
 * MetricHealthDetail - Detailed view of a single health check
 */
export function MetricHealthDetail({
  check,
  className = '',
}: MetricHealthDetailProps) {
  if (!check) {
    return (
      <Box className={`metric-health-detail ${className}`.trim()}>
        <Flex align="center" justify="center" className="metric-health-detail__empty">
          <Text size="2" className="metric-health-detail__empty-text">
            {MetricHealthStrings.DETAIL.SELECT_HEALTH_CHECK_VIEW_DETAILS}
          </Text>
        </Flex>
      </Box>
    );
  }

  const status = getHealthStatus(check.result);
  const { result } = check;

  const getStatusIcon = () => {
    switch (status) {
      case 'healthy':
        return <CheckCircle size={24} className="metric-health-detail__status-icon metric-health-detail__status-icon--healthy" />;
      case 'unhealthy':
        return <XCircle size={24} className="metric-health-detail__status-icon metric-health-detail__status-icon--unhealthy" />;
      default:
        return <HelpCircle size={24} className="metric-health-detail__status-icon metric-health-detail__status-icon--unknown" />;
    }
  };

  const getStatusBadge = () => {
    switch (status) {
      case 'healthy':
        return <Badge color="green" size="2">{MetricHealthStrings.DETAIL.HEALTHY}</Badge>;
      case 'unhealthy':
        return <Badge color="red" size="2">{MetricHealthStrings.DETAIL.UNHEALTHY}</Badge>;
      default:
        return <Badge color="gray" size="2">{MetricHealthStrings.DETAIL.UNKNOWN}</Badge>;
    }
  };

  return (
    <Box className={`metric-health-detail ${className}`.trim()}>
      {/* Header */}
      <Flex className="metric-health-detail__header" align="center" gap="3">
        {getStatusIcon()}
        <Box>
          <Heading as="h3" size="4" weight="medium" className="metric-health-detail__title">
            {formatCheckName(check.name)}
          </Heading>
          <Text size="1" className="metric-health-detail__subtitle">
            {check.name}
          </Text>
        </Box>
        <Box style={{ marginLeft: 'auto' }}>
          {getStatusBadge()}
        </Box>
      </Flex>

      {/* Details */}
      <Box className="metric-health-detail__content">
        {/* Message */}
        {result.message && (
          <Box className="metric-health-detail__section">
            <Text size="2" weight="medium" className="metric-health-detail__label">
              {MetricHealthStrings.DETAIL.MESSAGE}
            </Text>
            <Box className="metric-health-detail__message">
              <Text size="2">{result.message}</Text>
            </Box>
          </Box>
        )}

        {/* Error */}
        {result.error && (
          <Box className="metric-health-detail__section">
            <Text size="2" weight="medium" className="metric-health-detail__label metric-health-detail__label--error">
              {MetricHealthStrings.DETAIL.ERROR}
            </Text>
            <Box className="metric-health-detail__error">
              {result.error.message && (
                <Text size="2" className="metric-health-detail__error-message">
                  {result.error.message}
                </Text>
              )}
              {result.error.stack && (
                <pre className="metric-health-detail__stack">
                  {result.error.stack}
                </pre>
              )}
            </Box>
          </Box>
        )}

        {/* Duration */}
        {result.duration !== undefined && (
          <Box className="metric-health-detail__section">
            <Flex align="center" gap="2">
              <Clock size={14} className="metric-health-detail__duration-icon" />
              <Text size="2" className="metric-health-detail__duration">
                {MetricHealthStrings.DETAIL.DURATION_MS(result.duration)}
              </Text>
            </Flex>
          </Box>
        )}

        {/* Timestamp */}
        {result.timestamp && (
          <Box className="metric-health-detail__section">
            <Text size="2" className="metric-health-detail__timestamp">
              {MetricHealthStrings.DETAIL.LAST_CHECKED(new Date(result.timestamp).toLocaleString(pageLanguage()))}
            </Text>
          </Box>
        )}

        {/* No additional details */}
        {!((result.message || result.error ) || result.duration ) && (
          <Text size="2" className="metric-health-detail__no-details">
            {MetricHealthStrings.DETAIL.NO_ADDITIONAL_DETAILS_AVAILABLE_HEALTH}
          </Text>
        )}
      </Box>
    </Box>
  );
}

export default MetricHealthDetail;


