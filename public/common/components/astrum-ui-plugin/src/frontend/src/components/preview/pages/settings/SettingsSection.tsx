

/**
 * SettingsSection — Section wrapper with heading for Settings Hub
 *
 * Displays:
 * - Section heading (size="5")
 * - SettingsCard components: a list on phones, tiles in two to four columns on wider screens
 */

import React from 'react';
import { Box, Grid, Heading } from '@radix-ui/themes';
import SettingsCard from './SettingsCard';
import type { SettingsSection as SettingsSectionType } from './types';

export interface SettingsSectionProps {
  section: SettingsSectionType;
}

export default function SettingsSection({ section }: SettingsSectionProps) {
  return (
    <Box>
      <Heading as="h2" size="5" mb="3">{section.label}</Heading>
      <Grid columns={{ initial: '1', xs: '2', md: '3', xl: '4' }} gap="3">
        {section.cards.map((card) => (
          <SettingsCard key={card.id} card={card} />
        ))}
      </Grid>
    </Box>
  );
}
