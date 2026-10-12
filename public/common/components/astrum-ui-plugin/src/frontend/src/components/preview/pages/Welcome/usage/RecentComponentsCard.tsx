import React from 'react';
import { Card, Flex, Heading, Link, Text } from '@radix-ui/themes';
import UIStrings from '../../../../../constants/UIStrings';
import { FormatIcon } from '../../../shared/Badges/FormatIcon';
import { componentName, formatAgo, formatLabel, usageLinks } from './usageFormat';
import type { UsageComponent } from './usageDashboard.types';

/** How many of the components added last the card lists, so that it stays about as tall as its neighbour. */
const SHOWN = 6;

export interface RecentComponentsCardProps {
  components: UsageComponent[];
}

/**
 * The components added last, with their repository, who uploaded them and when.
 */
export function RecentComponentsCard({ components }: RecentComponentsCardProps): JSX.Element {
  const strings = UIStrings.USAGE_DASHBOARD.RECENT;
  return (
    <Card size="3" className="as-usage-dashboard__panel as-usage-dashboard__wide" aria-label={strings.TITLE}>
      <Heading as="h2" size="4" mb="2">{strings.TITLE}</Heading>
      {components.length === 0 ? (
        <Flex align="center" justify="center" className="as-usage-dashboard__empty">
          <Text size="2" color="gray">{strings.EMPTY}</Text>
        </Flex>
      ) : (
        <ul className="as-usage-dashboard__list as-usage-dashboard__recent">
          {components.slice(0, SHOWN).map((component) => (
            <li key={`${component.format}:${component.id}`}>
              <Flex align="center" gap="3" py="2">
                <span className="as-usage-dashboard__format-tile" title={formatLabel(component.format)}>
                  <FormatIcon format={component.format} size={22} />
                </span>
                <Flex direction="column" minWidth="0" flexGrow="1">
                  <Text size="2" truncate>
                    <Link href={usageLinks.component(component)} color="gray" highContrast underline="hover"
                      weight="medium">
                      {componentName(component)}
                    </Link>
                    {component.version && <span className="as-usage-dashboard__mono">{component.version}</span>}
                  </Text>
                  <Text size="1" color="gray" truncate>
                    {[component.repository, component.createdBy].filter(Boolean).join(' · ')}
                  </Text>
                </Flex>
                <Text size="1" color="gray" className="as-usage-dashboard__nowrap" title={component.created}>
                  {formatAgo(component.created)}
                </Text>
              </Flex>
            </li>
          ))}
        </ul>
      )}
    </Card>
  );
}

export default RecentComponentsCard;
