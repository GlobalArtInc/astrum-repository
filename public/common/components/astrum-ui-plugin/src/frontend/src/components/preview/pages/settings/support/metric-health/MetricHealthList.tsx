

import React, { useState, useMemo } from 'react';
import { Box, Flex, Text } from '@radix-ui/themes';
import { Search, CheckCircle, XCircle, HelpCircle } from 'lucide-react';

import { MetricHealthListProps, formatCheckName, getHealthStatus, sortHealthChecks } from './types';

import './MetricHealthList.scss';
import MetricHealthStrings from '../../../../constants/pages/settings/support/MetricHealthStrings';

/**
 * MetricHealthList - List of health checks with status indicators
 */
export function MetricHealthList({
  checks,
  selectedCheck,
  onSelectCheck,
  className = '',
}: MetricHealthListProps) {
  const [filter, setFilter] = useState('');

  // Sort and filter checks
  const filteredChecks = useMemo(() => {
    const sorted = sortHealthChecks(checks);
    
    if (!filter.trim()) {
      return sorted;
    }

    const lowerFilter = filter.toLowerCase();
    return sorted.filter(
      (check) =>
        check.name.toLowerCase().includes(lowerFilter) ||
        formatCheckName(check.name).toLowerCase().includes(lowerFilter)
    );
  }, [checks, filter]);

  // Count by status
  const statusCounts = useMemo(() => {
    return checks.reduce(
      (acc, check) => {
        const status = getHealthStatus(check.result);
        acc[status]++;
        return acc;
      },
      { healthy: 0, unhealthy: 0, unknown: 0 }
    );
  }, [checks]);

  const getStatusIcon = (check: typeof checks[0]) => {
    const status = getHealthStatus(check.result);
    switch (status) {
      case 'healthy':
        return <CheckCircle size={16} className="metric-health-list__icon metric-health-list__icon--healthy" />;
      case 'unhealthy':
        return <XCircle size={16} className="metric-health-list__icon metric-health-list__icon--unhealthy" />;
      default:
        return <HelpCircle size={16} className="metric-health-list__icon metric-health-list__icon--unknown" />;
    }
  };

  return (
    <Box className={`metric-health-list ${className}`.trim()}>
      {/* Summary */}
      <Flex className="metric-health-list__summary" gap="4">
        <Flex align="center" gap="1">
          <CheckCircle size={14} className="metric-health-list__icon--healthy" />
          <Text size="1">{MetricHealthStrings.LIST.HEALTHY_COUNT(statusCounts.healthy)}</Text>
        </Flex>
        <Flex align="center" gap="1">
          <XCircle size={14} className="metric-health-list__icon--unhealthy" />
          <Text size="1">{MetricHealthStrings.LIST.UNHEALTHY_COUNT(statusCounts.unhealthy)}</Text>
        </Flex>
        {statusCounts.unknown > 0 && (
          <Flex align="center" gap="1">
            <HelpCircle size={14} className="metric-health-list__icon--unknown" />
            <Text size="1">{MetricHealthStrings.LIST.UNKNOWN_COUNT(statusCounts.unknown)}</Text>
          </Flex>
        )}
      </Flex>

      {/* Search filter */}
      <Box className="metric-health-list__filter">
        <Flex align="center" gap="2">
          <Search size={14} className="metric-health-list__filter-icon" />
          <input
            type="text"
            className="metric-health-list__filter-input"
            placeholder={MetricHealthStrings.LIST.FILTER_HEALTH_CHECKS_PLACEHOLDER}
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
          />
        </Flex>
      </Box>

      {/* List */}
      <Box className="metric-health-list__items">
        {filteredChecks.length === 0 ? (
          <Text size="2" className="metric-health-list__empty">
            {filter ? MetricHealthStrings.LIST.NO_MATCHING_HEALTH_CHECKS_FOUND : MetricHealthStrings.LIST.NO_HEALTH_CHECKS_AVAILABLE}
          </Text>
        ) : (
          filteredChecks.map((check) => (
            <button
              key={check.name}
              type="button"
              className={`metric-health-list__item ${
                selectedCheck === check.name ? 'metric-health-list__item--selected' : ''
              }`}
              onClick={() => onSelectCheck(check.name)}
              aria-selected={selectedCheck === check.name}
            >
              <Flex align="center" gap="2">
                {getStatusIcon(check)}
                <Text size="2" className="metric-health-list__name">
                  {formatCheckName(check.name)}
                </Text>
              </Flex>
            </button>
          ))
        )}
      </Box>
    </Box>
  );
}

export default MetricHealthList;


