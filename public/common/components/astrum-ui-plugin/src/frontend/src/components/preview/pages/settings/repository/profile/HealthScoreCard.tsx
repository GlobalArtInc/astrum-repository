

import React from 'react';
import { Box, Text } from '@radix-ui/themes';
import type { HealthCheckData, RepositoryMetrics } from './types';
import RepositoryProfileStrings from '../../../../constants/pages/settings/repository/RepositoryProfileStrings';

// =============================================================================
// Types
// =============================================================================

interface HealthScoreCardProps {
  healthCheck: HealthCheckData | null;
  metrics: RepositoryMetrics | null;
  repositoryName: string;
}

// =============================================================================
// Component
// =============================================================================

/**
 * HealthScoreCard - Displays a health score and key metrics for a repository
 *
 * Follows the GlobalArt Guide "Developer Trust Score" pattern.
 */
export function HealthScoreCard({
  healthCheck,
  metrics,
  repositoryName,
}: HealthScoreCardProps): JSX.Element {
  // Calculate health score (simplified)
  // In a real implementation, this would come from the health check API
  const calculateScore = (): number | null => {
    if (!healthCheck?.enabled) return null;

    // Simple scoring: start at 100, subtract for issues
    let score = 100;
    if (healthCheck.securityIssueCount) {
      score -= healthCheck.securityIssueCount * 5;
    }
    if (healthCheck.licenseIssueCount) {
      score -= healthCheck.licenseIssueCount * 2;
    }
    return Math.max(0, Math.min(100, score));
  };

  const score = calculateScore();

  const getScoreClass = (score: number | null): string => {
    if (score === null) return 'unknown';
    if (score >= 90) return 'excellent';
    if (score >= 70) return 'good';
    if (score >= 50) return 'fair';
    return 'poor';
  };

  const scoreClass = getScoreClass(score);

  return (
    <Box className="health-score-card">
      <Text className="health-score-card__title">{RepositoryProfileStrings.HEALTH_SCORE.HEALTH_SCORE}</Text>

      <Text className={`health-score-card__score health-score-card__score--${scoreClass}`}>
        {score !== null ? score : '—'}
      </Text>

      <Text className="health-score-card__label">
        {score !== null ? RepositoryProfileStrings.HEALTH_SCORE.OUT_100 : RepositoryProfileStrings.HEALTH_SCORE.NOT_AVAILABLE}
      </Text>

      <Box className="health-score-card__divider" />

      <Box className="health-score-card__stats">
        <Box className="health-score-card__stat">
          <Text className="health-score-card__stat-label">{RepositoryProfileStrings.HEALTH_SCORE.COMPONENTS}</Text>
          <Text className="health-score-card__stat-value">
            {metrics?.componentCount?.toLocaleString() ?? '—'}
          </Text>
        </Box>

        <Box className="health-score-card__stat">
          <Text className="health-score-card__stat-label">{RepositoryProfileStrings.HEALTH_SCORE.ASSETS}</Text>
          <Text className="health-score-card__stat-value">
            {metrics?.assetCount?.toLocaleString() ?? '—'}
          </Text>
        </Box>

        {healthCheck?.securityIssueCount !== undefined && (
          <Box className="health-score-card__stat">
            <Text className="health-score-card__stat-label">{RepositoryProfileStrings.HEALTH_SCORE.SECURITY_ISSUES}</Text>
            <Text className="health-score-card__stat-value">
              {healthCheck.securityIssueCount}
            </Text>
          </Box>
        )}
      </Box>
    </Box>
  );
}

export default HealthScoreCard;


