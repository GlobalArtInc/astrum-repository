import React from 'react';
import { Button, Card, Flex, Heading, Text } from '@radix-ui/themes';
import UIStrings from '../../../../../constants/UIStrings';
import plural from '../../../../../interface/plural';
import { UsageBar } from './UsageBar';
import { formatBytes, formatDays, formatNumber, usageLinks } from './usageFormat';
import type { UnusedComponents } from './usageDashboard.types';

export interface UnusedCardProps {
  unused: UnusedComponents;
  unusedDays: number;
  canConfigureCleanup: boolean;
}

/**
 * The components nobody downloaded for a while and the repositories holding most of them, pointing at cleanup
 * policies.
 */
export function UnusedCard({ unused, unusedDays, canConfigureCleanup }: UnusedCardProps): JSX.Element {
  const strings = UIStrings.USAGE_DASHBOARD.UNUSED;
  const period = formatDays(unusedDays);
  const most = unused.repositories[0]?.bytes || 1;

  return (
    <Card size="3" className="as-usage-dashboard__panel as-usage-dashboard__narrow" aria-label={strings.TITLE(period)}>
      <Flex direction="column" gap="4" height="100%">
        <div>
          <Heading as="h2" size="4">{strings.TITLE(period)}</Heading>
          <Flex align="baseline" gap="2" wrap="wrap" mt="2">
            <span className="as-usage-dashboard__big-number">{formatNumber(unused.components)}</span>
            <Text size="2" color="gray">
              {plural(strings.COMPONENTS, unused.components)}
              {' · '}
              <Text color="orange" weight="bold">{formatBytes(unused.bytes)}</Text>
            </Text>
          </Flex>
          {unused.neverDownloadedComponents > 0 && (
            <Text as="div" size="1" color="gray" mt="1">
              {strings.NEVER_DOWNLOADED(formatNumber(unused.neverDownloadedComponents),
                formatBytes(unused.neverDownloadedBytes))}
            </Text>
          )}
        </div>
        {unused.repositories.length === 0 ? (
          <Text size="2" color="gray">{strings.EMPTY(period)}</Text>
        ) : (
          <Flex direction="column" gap="3" asChild>
            <ul className="as-usage-dashboard__list">
              {unused.repositories.map((repository) => (
                <li key={repository.name}>
                  <Flex direction="column" gap="2">
                    <Flex justify="between" gap="2">
                      <Text size="2" weight="medium" truncate>{repository.name}</Text>
                      <Text size="2" color="gray" className="as-usage-dashboard__count">
                        {plural(strings.COUNT, repository.components, formatNumber(repository.components))}
                        {' · '}{formatBytes(repository.bytes)}
                      </Text>
                    </Flex>
                    <UsageBar share={repository.bytes / most} color="orange" />
                  </Flex>
                </li>
              ))}
            </ul>
          </Flex>
        )}
        {canConfigureCleanup && (
          <Flex mt="auto">
            <Button asChild size="2">
              <a href={usageLinks.cleanupPolicies()} className="as-usage-dashboard__cleanup">{strings.CLEANUP}</a>
            </Button>
          </Flex>
        )}
      </Flex>
    </Card>
  );
}

export default UnusedCard;
