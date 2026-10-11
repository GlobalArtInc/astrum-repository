

/**
 * SettingsCard — one entry of the Settings Hub
 *
 * Phones show a list row: name, description and a chevron. Wider screens show a tile: the icon of the
 * page in a soft accent square, the name and a description of at most two lines.
 *
 * The entire card is the link to the settings page.
 */

import React from 'react';
import { Badge, Box, Card, Flex, Text } from '@radix-ui/themes';
import { ChevronRight, Settings } from 'lucide-react';
import { useIsPhone } from '../../../../hooks/useMediaQuery';
import { isFeatureEnabled } from '../../config/featureFlags';
import type { SettingCard } from './types';
import { SETTINGS_ICONS } from './settingsIcons';
import SettingsStrings from '../../constants/pages/settings/SettingsStrings';
import './SettingsCard.scss';

export interface SettingsCardProps {
  card: SettingCard;
}

export default function SettingsCard({ card }: SettingsCardProps) {
  const href = card.fullHash ?? `#preview/admin/${card.path}`;
  const isComingSoon = card.featureKey ? !isFeatureEnabled(card.featureKey) : false;
  const isPhone = useIsPhone();
  const Icon = SETTINGS_ICONS[card.id] ?? Settings;

  const title = (
    <Flex align="center" gap="2" wrap="wrap">
      <Text size="2" weight="medium">{card.label}</Text>
      {isComingSoon && (
        <Badge color="blue" size="1" variant="soft">
          {SettingsStrings.CARD.COMING_SOON}
        </Badge>
      )}
    </Flex>
  );

  if (isPhone) {
    return (
      <Card asChild className="settings-card">
        <a href={href}>
          <Box p="3">
            <Flex align="start" justify="between" gap="4">
              <Box style={{ flex: 1, minWidth: 0 }}>
                {title}
                <Text as="p" size="2" color="gray" style={{ margin: 0 }}>
                  {card.description}
                </Text>
              </Box>
              <ChevronRight size={18} color="var(--gray-11)" aria-hidden="true" style={{ flexShrink: 0, alignSelf: 'center' }} />
            </Flex>
          </Box>
        </a>
      </Card>
    );
  }

  return (
    <Card asChild className="settings-card settings-card--tile">
      <a href={href}>
        <Flex direction="column" gap="3" p="2">
          <span className="settings-card__icon" aria-hidden="true">
            <Icon size={18} />
          </span>
          <Box>
            {title}
            <Text as="p" size="2" color="gray" className="settings-card__description">
              {card.description}
            </Text>
          </Box>
        </Flex>
      </a>
    </Card>
  );
}
