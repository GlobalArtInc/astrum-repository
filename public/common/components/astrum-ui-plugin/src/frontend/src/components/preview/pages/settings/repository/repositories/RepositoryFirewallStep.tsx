

/**
 * Repository Creation wizard – Step 4 (proxy repos only).
 *
 * Enables Firewall (None / Audit / Quarantine / PCCS) if licensed, or shows cross-sell with
 * option to skip. PCCS is conditionally offered for npm/pypi proxies, gated by the
 * format-capabilities API ({@code GET /v1/repositories/firewall/format-capabilities}). Same
 * button treatment as Malware Defense with outline on selected.
 */
import React, { useEffect, useState } from 'react';
import { Box, Flex, Text, Card, Button, } from '@radix-ui/themes';
import { ExternalLink } from 'lucide-react';
import {
  fetchPccsSupportedFormats,
  useFirewallEnable,
} from '../../../../shared/security/useFirewallEnable';
import { ConfirmDialog } from '../../../../shared/form/ConfirmDialog';
import RepositoriesPageStrings from '../../../../constants/pages/settings/repository/RepositoriesPageStrings';
import CommonStrings from '../../../../constants/CommonStrings';

export type ProtectionLevel = 'none' | 'audit' | 'quarantine' | 'pccs';

export interface RepositoryFirewallStepProps {
  /** Required for immediate mode; omit for deferred mode */
  repositoryName?: string;
  /** When false, show cross-sell (Learn more / Contact sales) instead of Enable buttons */
  hasFirewallLicense?: boolean;
  /**
   * Repository format. Determines whether PCCS is offered (npm/pypi). Optional for callers
   * that pre-date PCCS support; when omitted, PCCS is hidden.
   */
  format?: string;
  /** In immediate mode: called after API succeeds. In deferred: not used (parent advances via Next). */
  onComplete?: () => void;
  /** Deferred mode: store choice, no API calls. Parent advances via wizard Next. */
  mode?: 'immediate' | 'deferred';
  /** Deferred mode: current choice (controlled) */
  value?: ProtectionLevel;
  /** Deferred mode: when user selects a level */
  onChoice?: (level: ProtectionLevel) => void;
}

export function RepositoryFirewallStep({
  repositoryName = '',
  hasFirewallLicense = true,
  format,
  onComplete,
  mode = 'immediate',
  value = 'none',
  onChoice,
}: RepositoryFirewallStepProps): JSX.Element {
  const [auditModalOpen, setAuditModalOpen] = useState(false);
  const [noneModalOpen, setNoneModalOpen] = useState(false);
  const [pccsSupported, setPccsSupported] = useState(false);
  const { enableAudit, enableQuarantine, enablePccs, loading } = useFirewallEnable(repositoryName);
  const isDeferred = mode === 'deferred';

  useEffect(() => {
    if (!format) {
      setPccsSupported(false);
      return;
    }
    let cancelled = false;
    fetchPccsSupportedFormats().then((formats) => {
      if (!cancelled) {
        setPccsSupported(formats.includes(format));
      }
    });
    return () => {
      cancelled = true;
    };
  }, [format]);

  const handleLevelChange = async (level: ProtectionLevel) => {
    if (level === 'none') {
      setNoneModalOpen(true);
      return;
    }
    if (level === 'audit') {
      setAuditModalOpen(true);
      return;
    }
    if (level === 'pccs') {
      // No confirm dialog: PCCS is the strongest mode, advancing to it is intentional.
      if (isDeferred) {
        onChoice?.('pccs');
        return;
      }
      try {
        await enablePccs(onComplete!);
      }
      catch {
        // Error shown inline
      }
      return;
    }
    if (isDeferred) {
      onChoice?.('quarantine');
      return;
    }
    try {
      await enableQuarantine(onComplete!);
    } catch {
      // Error shown inline
    }
  };

  const handleNoneConfirm = () => {
    setNoneModalOpen(false);
    if (isDeferred) {
      onChoice?.('none');
    } else {
      onComplete?.();
    }
  };

  const handleAuditConfirm = async () => {
    setAuditModalOpen(false);
    if (isDeferred) {
      onChoice?.('audit');
    } else {
      try {
        await enableAudit(onComplete!);
      } catch {
        // Error shown inline
      }
    }
  };

  const handleEnableQuarantineFromAuditModal = async () => {
    setAuditModalOpen(false);
    if (isDeferred) {
      onChoice?.('quarantine');
    } else {
      try {
        await enableQuarantine(onComplete!);
      } catch {
        // Error shown inline
      }
    }
  };

  return (
    <Box p="4" className="repository-firewall-step">
      <Card size="2">
        <Flex direction="column" gap="4">
          <Box>
            <Text size="4" weight="bold" as="div" mb="1">
              {RepositoriesPageStrings.FIREWALL_STEP.ENABLE_REPOSITORY_FIREWALL}
            </Text>
            <Text size="2" color="gray" as="div">
              {RepositoriesPageStrings.FIREWALL_STEP.PROTECT_PROXY_REPOSITORY_ENABLING_FIREWALL}
            </Text>
          </Box>

          {hasFirewallLicense ? (
            <Box>
              {isDeferred && (
                <Text size="2" color="gray" mb="2" as="div">
                  {value === 'none'
                    ? RepositoriesPageStrings.FIREWALL_STEP.CURRENT_CHOICE_SKIP_NO_FIREWALL
                    : value === 'audit'
                      ? RepositoriesPageStrings.FIREWALL_STEP.FIREWALL_ENABLED_AUDIT_MODE
                      : value === 'pccs'
                        ? RepositoriesPageStrings.FIREWALL_STEP.FIREWALL_ENABLED_PCCS_MODE
                        : RepositoriesPageStrings.FIREWALL_STEP.FIREWALL_ENABLED_QUARANTINE_MODE}
                </Text>
              )}
              <Flex gap="3" wrap="wrap">
                <Button
                  type="button"
                  variant={isDeferred && value === 'none' ? 'solid' : 'soft'}
                  color="gray"
                  size="3"
                  onClick={() => handleLevelChange('none')}
                  disabled={!isDeferred && loading}
                  aria-pressed={isDeferred && value === 'none'}
                >
                  {RepositoriesPageStrings.FIREWALL_STEP.SKIP}
                </Button>
                <Button
                  type="button"
                  variant={isDeferred && value === 'audit' ? 'solid' : 'soft'}
                  color="amber"
                  size="3"
                  onClick={() => handleLevelChange('audit')}
                  disabled={!isDeferred && loading}
                  aria-pressed={isDeferred && value === 'audit'}
                >
                  {RepositoriesPageStrings.FIREWALL_STEP.AUDIT}
                </Button>
                <Button
                  type="button"
                  variant={isDeferred && value === 'quarantine' ? 'solid' : 'soft'}
                  color="green"
                  size="3"
                  onClick={() => handleLevelChange('quarantine')}
                  disabled={!isDeferred && loading}
                  aria-pressed={isDeferred && value === 'quarantine'}
                >
                  {RepositoriesPageStrings.FIREWALL_STEP.QUARANTINE}
                </Button>
                {pccsSupported && (
                  <Button
                    type="button"
                    variant={isDeferred && value === 'pccs' ? 'solid' : 'soft'}
                    color="orange"
                    size="3"
                    onClick={() => handleLevelChange('pccs')}
                    disabled={!isDeferred && loading}
                    aria-pressed={isDeferred && value === 'pccs'}
                  >
                    {RepositoriesPageStrings.FIREWALL_STEP.PCCS}
                  </Button>
                )}
              </Flex>
            </Box>
          ) : (
            <Flex direction="column" gap="3">
              <Text size="2" color="gray">
                {RepositoriesPageStrings.FIREWALL_STEP.PROTECT_REPOSITORY_REPOSITORY_FIREWALL_REQUIRES}
              </Text>
              <Flex gap="2" wrap="wrap">
                <Button variant="ghost" size="2" asChild>
                  <a
                    href="https://links.globalart.dev/astrum-repository-firewall"
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    <ExternalLink size={14} />
                    {CommonStrings.ACTIONS.LEARN_MORE}
                  </a>
                </Button>
                <Button variant="ghost" size="2" asChild>
                  <a href="https://links.globalart.dev/contact-sales" target="_blank" rel="noopener noreferrer">
                    {RepositoriesPageStrings.FIREWALL_STEP.CONTACT_SALES}
                  </a>
                </Button>
                <Button type="button" variant="soft" size="2" onClick={() => (isDeferred ? onChoice?.('none') : onComplete?.())}>
                  {RepositoriesPageStrings.FIREWALL_STEP.NONE}
                </Button>
              </Flex>
            </Flex>
          )}
        </Flex>
      </Card>

      <ConfirmDialog
        open={noneModalOpen}
        onOpenChange={setNoneModalOpen}
        title={RepositoriesPageStrings.FIREWALL_STEP.REPOSITORY_CREATED_WITHOUT_PROTECTION}
        message={RepositoriesPageStrings.FIREWALL_STEP.REPOSITORY_NOT_PROTECTED_FIREWALL_MALICIOUS}
        confirmLabel={RepositoriesPageStrings.FIREWALL_STEP.CONTINUE_ANYWAY}
        cancelLabel={RepositoriesPageStrings.FIREWALL_STEP.GO_BACK}
        variant="warning"
        onConfirm={handleNoneConfirm}
        testId="firewall-skip-confirm"
      />

      <ConfirmDialog
        open={auditModalOpen}
        onOpenChange={setAuditModalOpen}
        title={RepositoriesPageStrings.FIREWALL_STEP.AUDIT_MODE_NOT_FULLY_PROTECTED}
        message={RepositoriesPageStrings.FIREWALL_STEP.AUDIT_MODE_MONITORS_COMPONENTS_BUT}
        confirmLabel={RepositoriesPageStrings.FIREWALL_STEP.ENABLE_AUDIT_ANYWAY}
        cancelLabel={CommonStrings.ACTIONS.CANCEL}
        variant="warning"
        onConfirm={handleAuditConfirm}
        testId="firewall-audit-confirm"
      >
        <Flex gap="2" mt="3" wrap="wrap">
          <Button
            type="button"
            variant="soft"
            color="orange"
            size="2"
            onClick={handleEnableQuarantineFromAuditModal}
            disabled={loading}
          >
            {RepositoriesPageStrings.FIREWALL_STEP.ENABLE_QUARANTINE_INSTEAD}
          </Button>
        </Flex>
      </ConfirmDialog>
    </Box>
  );
}

export default RepositoryFirewallStep;
