

/**
 * Repository Creation wizard – Step 5: Enable RHC (Repository Health Check).
 * Proxy repos only. Same pattern as Firewall: Enable or None (skip).
 */
import React, { useState } from 'react';
import { Box, Flex, Text, Card, Switch, Callout } from '@radix-ui/themes';
import { AlertCircle } from 'lucide-react';
import { useRepositoriesApi } from './useRepositoriesApi';

import './RepositoryRHCStep.scss';

export interface RepositoryRHCStepProps {
  /** Required for immediate mode; omit for deferred mode */
  repositoryName?: string;
  /** In immediate mode: called after API succeeds. In deferred: not used (Create Repository triggers form submit). */
  onComplete?: () => void;
  /** Deferred mode: store choice, no API calls. Applied when Create Repository is clicked. */
  mode?: 'immediate' | 'deferred';
  /** Deferred mode: current choice */
  value?: 'enable' | 'none';
  /** Deferred mode: when user selects Enable or None */
  onChoice?: (choice: 'enable' | 'none') => void;
  /** Whether the Health Check system capability is enabled on this instance. Defaults to true. */
  capabilityEnabled?: boolean;
}

export function RepositoryRHCStep({
  repositoryName = '',
  onComplete,
  mode = 'immediate',
  value = 'none',
  onChoice,
  capabilityEnabled = true,
}: RepositoryRHCStepProps): JSX.Element {
  const { enableHealthCheck } = useRepositoriesApi();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const isDeferred = mode === 'deferred';

  const handleToggle = async (checked: boolean) => {
    if (isDeferred) {
      onChoice?.(checked ? 'enable' : 'none');
      return;
    }

    if (!checked) {
      // User disabled - just complete without enabling
      onComplete?.();
      return;
    }

    // User enabled - call API
    setLoading(true);
    setError(null);
    try {
      await enableHealthCheck(repositoryName);
      onComplete?.();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to enable Health Check');
    } finally {
      setLoading(false);
    }
  };

  return (
    <Box p="4" className="repository-rhc-step">
      <Card size="2">
        <Flex direction="column" gap="4">
          <Box>
            <Text size="4" weight="bold" as="div" mb="1">
              Repository Health Check
            </Text>
            <Text size="2" color="gray" as="div">
              Repository Health Check analyzes components for security vulnerabilities and license issues. You can
              enable it now or configure it later from the repository profile.
            </Text>
          </Box>

          {!capabilityEnabled && (
            <Callout.Root color="yellow" data-testid="rhc-capability-disabled-callout">
              <Callout.Icon>
                <AlertCircle size={16} />
              </Callout.Icon>
              <Callout.Text>
                Repository Health Check is not available — the system capability is not enabled.{' '}
                <a href="#preview/admin/system/capabilities">Go to System Capabilities →</a>
              </Callout.Text>
            </Callout.Root>
          )}

          {error && (
            <Text size="2" color="red">
              {error}
            </Text>
          )}

          <Flex asChild gap="3" align="center">
            <label>
              <Switch
                size="3"
                checked={value === 'enable'}
                onCheckedChange={handleToggle}
                disabled={(!isDeferred && loading) || !capabilityEnabled}
              />
              <Text size="3" weight="medium" color={capabilityEnabled ? undefined : 'gray'}>
                Enable Repository Health Check
              </Text>
            </label>
          </Flex>
        </Flex>
      </Card>
    </Box>
  );
}
