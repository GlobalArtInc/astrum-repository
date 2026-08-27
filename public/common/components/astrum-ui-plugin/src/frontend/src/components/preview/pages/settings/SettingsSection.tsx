

/**
 * SettingsSection — Section wrapper with heading for Settings Hub
 *
 * Displays:
 * - Section heading (size="5")
 * - Grid of SettingsCard components with gap="3"
 */

import React from 'react';
import { Box, Flex, Heading } from '@radix-ui/themes';
import SettingsCard from './SettingsCard';
import type { SettingsSection as SettingsSectionType } from './types';

export interface SettingsSectionProps {
  section: SettingsSectionType;
}

export default function SettingsSection({ section }: SettingsSectionProps) {
  return (
    <Box>
      <Heading as="h2" size="5" mb="3">{section.label}</Heading>
      <Flex direction="column" gap="3">
        {section.cards.map((card) => (
          <SettingsCard key={card.id} card={card} />
        ))}
      </Flex>
    </Box>
  );
}
