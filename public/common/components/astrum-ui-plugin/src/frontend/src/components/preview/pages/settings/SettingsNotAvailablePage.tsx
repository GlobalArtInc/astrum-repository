

/**
 * SettingsNotAvailablePage —placeholder page for settings not yet migrated to the new UI
 *
 * Displays a centered message explaining that a feature is still being prepared
 * for the new UI and should be accessed through the classic administration panel.
 */

import React from 'react';
import { Flex, Heading, Text, Button } from '@radix-ui/themes';
import { Construction, ArrowLeft } from 'lucide-react';

export interface SettingsNotAvailablePageProps {
  featureName: string;
}

export default function SettingsNotAvailablePage({ featureName }: SettingsNotAvailablePageProps) {
  return (
    <Flex
      direction="column"
      align="center"
      justify="center"
      style={{ minHeight: '100vh', padding: 'var(--space-6)', background: 'var(--color-background)' }}
    >
      <Flex
        direction="column"
        align="center"
        gap="4"
        style={{ maxWidth: '500px', textAlign: 'center' }}
      >
        <Construction
          size={64}
          strokeWidth={1.5}
          style={{ color: 'var(--gray-8)', marginBottom: 'var(--space-2)' }}
        />
        <Heading size="7">Not available in preview</Heading>
        <div style={{ width: '60px', height: '3px', background: 'var(--accent-9)', borderRadius: 'var(--radius-1)', margin: 'var(--space-1) 0' }} />
        <Text size="3" color="gray">
          {featureName} is still being prepared for the Astrum One UI.
          Access it through the classic UI administration panel for now.
        </Text>
        <Button variant="solid" size="3" mt="2" asChild>
          <a href="#preview/admin/settings" style={{ color: 'var(--accent-contrast)', textDecoration: 'none' }}>
            <ArrowLeft size={16} />
            Back to Settings
          </a>
        </Button>
      </Flex>
    </Flex>
  );
}
