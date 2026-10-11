

import React from 'react';
import { Flex, Text, Heading, Badge, Tooltip, Link } from '@radix-ui/themes';
import { AlertCircle, ShieldOff } from 'lucide-react';
import SharedStrings from '../constants/SharedStrings';

const FIREWALL_DOCS_URL = 'https://help.globalart.dev/en/repository-firewall.html';

export interface FirewallNotSupportedEmptyStateProps {
  /** The repository format (e.g. 'terraform', 'helm') */
  format: string;
  /**
   * Rendering context controls sizing and layout:
   * - 'cell'  → inline AlertCircle + "Not supported" text (repo list / quick config)
   * - 'tab'   → centered full empty state with ShieldOff icon (repo profile tab)
   */
  context?: 'cell' | 'tab';
}

/**
 * FirewallNotSupportedEmptyState — unified N/A treatment for unsupported Firewall formats.
 *
 * Used in:
 * - FirewallCell (context="cell")
 * - MalwareQuickActionsTab badge (context="cell")
 * - FirewallReportTab (context="tab")
 */
export function FirewallNotSupportedEmptyState({
  format,
  context = 'cell',
}: FirewallNotSupportedEmptyStateProps): JSX.Element {
  const tooltipText = SharedStrings.FIREWALL_NOT_SUPPORTED.FORMAT_NOT_SUPPORTED;

  if (context === 'tab') {
    return (
      <Flex
        direction="column"
        align="center"
        justify="center"
        gap="3"
        style={{ padding: 'var(--space-8)', minHeight: 200 }}
        data-testid="firewall-not-supported-empty-state"
      >
        <ShieldOff size={48} color="var(--gray-8)" aria-hidden="true" />
        <Heading size="4" color="gray">
          {SharedStrings.FIREWALL_NOT_SUPPORTED.FIREWALL_NOT_AVAILABLE}
        </Heading>
        <Text size="2" color="gray" align="center" style={{ maxWidth: 400 }}>
          {SharedStrings.FIREWALL_NOT_SUPPORTED.REPOSITORY_FIREWALL_DOES_NOT_SUPPORT(format)}
        </Text>
        <Link
          href={FIREWALL_DOCS_URL}
          target="_blank"
          rel="noopener noreferrer"
          size="2"
        >
          {SharedStrings.FIREWALL_NOT_SUPPORTED.LEARN_ABOUT_SUPPORTED_FORMATS}
        </Link>
      </Flex>
    );
  }

  return (
    <Tooltip content={tooltipText}>
      <Flex
        align="center"
        justify="center"
        gap="1"
        data-testid="firewall-not-supported-badge"
        aria-label={SharedStrings.FIREWALL_NOT_SUPPORTED.FIREWALL_NOT_SUPPORTED(format)}
      >
        <AlertCircle size={14} color="var(--gray-8)" aria-hidden="true" />
        <Text size="1" color="gray">
          {SharedStrings.FIREWALL_NOT_SUPPORTED.NOT_SUPPORTED}
        </Text>
      </Flex>
    </Tooltip>
  );
}

/**
 * A standalone gray Badge variant for use in table rows (Quick Config).
 * Separate from the inline cell variant to avoid nesting interactive elements.
 */
export function FirewallNotSupportedBadge({ format }: { format: string }): JSX.Element {
  const tooltipText = SharedStrings.FIREWALL_NOT_SUPPORTED.FORMAT_NOT_SUPPORTED;

  return (
    <Tooltip content={tooltipText}>
      <Badge
        variant="soft"
        color="gray"
        size="1"
        data-testid="firewall-not-supported-badge"
        aria-label={SharedStrings.FIREWALL_NOT_SUPPORTED.FIREWALL_NOT_SUPPORTED(format)}
      >
        {SharedStrings.FIREWALL_NOT_SUPPORTED.NOT_SUPPORTED}
      </Badge>
    </Tooltip>
  );
}

export default FirewallNotSupportedEmptyState;
