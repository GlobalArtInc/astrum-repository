

/**
 * SettingsCard — Small variant card for Settings Hub
 *
 * Displays a layout with:
 * - Name (bold text) with optional Coming Soon badge
 * - Description (gray text, wraps naturally)
 * - View button
 *
 * The entire card is clickable and navigates to the settings page.
 */

import React from 'react';
import { Badge, Box, Flex, Text, Button, Card } from '@radix-ui/themes';
import { isFeatureEnabled } from '../../config/featureFlags';
import type { SettingCard } from './types';

export interface SettingsCardProps {
  card: SettingCard;
}

export default function SettingsCard({ card }: SettingsCardProps) {
  const href = card.fullHash ?? `#preview/admin/${card.path}`;
  const isComingSoon = card.featureKey ? !isFeatureEnabled(card.featureKey) : false;

  return (
    <Card asChild style={{ cursor: 'pointer' }}>
      <a href={href}>
        <Box p="3">
          <Flex align="start" justify="between" gap="4">
            <Box style={{ flex: 1, minWidth: 0 }}>
              <Flex align="center" gap="2">
                <Text size="2" weight="medium">{card.label}</Text>
                {isComingSoon && (
                  <Badge color="blue" size="1" variant="soft">
                    Coming Soon
                  </Badge>
                )}
              </Flex>
              <Text as="p" size="2" color="gray" style={{ margin: 0 }}>
                {card.description}
              </Text>
            </Box>
            <Button variant="surface" size="1">View</Button>
          </Flex>
        </Box>
      </a>
    </Card>
  );
}
