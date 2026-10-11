

import React, { useEffect, useState } from 'react';
import {
  Badge,
  Box,
  Button,
  Callout,
  Card,
  Code,
  Dialog,
  Flex,
  Heading,
  IconButton,
  Separator,
  Spinner,
  Text,
  Tooltip,
} from '@radix-ui/themes';
import {
  Check,
  CheckCircle,
  Copy,
  Key,
  KeyRound,
} from 'lucide-react';
import { ConfirmDialog } from '../../shared/form';
import { useUserToken } from './useUserToken';
import UserPagesStrings from '../../constants/pages/user/UserPagesStrings';
import CommonStrings from '../../constants/CommonStrings';
import { pageLanguage } from '../../../../interface/plural';

function CopyField({ label, value }: { label: string; value: string }) {
  const [copied, setCopied] = useState(false);
  const handleCopy = () => {
    navigator.clipboard.writeText(value);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };
  return (
    <Box>
      <Text size="1" color="gray" style={{ display: 'block', marginBottom: '4px' }}>
        {label}
      </Text>
      <Flex align="center" gap="2">
        <Code
          variant="soft"
          size="2"
          style={{ flex: 1, wordBreak: 'break-all' }}
          data-testid={`copy-field-${label.toLowerCase().replace(/\s+/g, '-')}`}
        >
          {value}
        </Code>
        <Tooltip content={copied ? UserPagesStrings.USER_TOKEN.COPIED : UserPagesStrings.USER_TOKEN.COPY_CLIPBOARD}>
          <IconButton
            variant="ghost"
            size="2"
            onClick={handleCopy}
            aria-label={UserPagesStrings.USER_TOKEN.COPY(label)}
            data-testid={`copy-btn-${label.toLowerCase().replace(/\s+/g, '-')}`}
          >
            {copied ? <Check size={14} color="var(--green-9)" /> : <Copy size={14} />}
          </IconButton>
        </Tooltip>
      </Flex>
    </Box>
  );
}

const REVEAL_TIMEOUT_SECONDS = 60;

interface TokenRevealDialogProps {
  nameCode: string;
  passCode: string;
  onClose: () => void;
}

function TokenRevealDialog({ nameCode, passCode, onClose }: TokenRevealDialogProps) {
  const [seconds, setSeconds] = useState(REVEAL_TIMEOUT_SECONDS);

  useEffect(() => {
    const interval = setInterval(() => {
      setSeconds((s) => {
        if (s <= 1) {
          clearInterval(interval);
          onClose();
          return 0;
        }
        return s - 1;
      });
    }, 1000);
    return () => clearInterval(interval);
  }, [onClose]);

  const mavenSnippet = `<server>\n  <id>\${server}</id>\n  <username>${nameCode}</username>\n  <password>${passCode}</password>\n</server>`;
  const base64Value = btoa(`${nameCode}:${passCode}`);

  return (
    <Dialog.Root open onOpenChange={(open) => { if (!open) onClose(); }}>
      <Dialog.Content maxWidth="680px" data-testid="token-reveal-dialog">
        <Dialog.Title>{UserPagesStrings.USER_TOKEN.USER_TOKEN}</Dialog.Title>

        <Callout.Root color="amber" mb="4">
          <Callout.Text>
            {UserPagesStrings.USER_TOKEN.KEEP_TOKEN_SECRET_TREAT_LIKE(seconds)}
          </Callout.Text>
        </Callout.Root>

        <Flex direction="column" gap="4">
          <CopyField label={UserPagesStrings.USER_TOKEN.USER_CODE_USERNAME} value={nameCode} />
          <CopyField label={UserPagesStrings.USER_TOKEN.PASSCODE_PASSWORD} value={passCode} />

          <Separator size="4" />

          <Box>
            <Text size="1" color="gray" weight="medium" style={{ display: 'block', marginBottom: '6px' }}>
              {UserPagesStrings.USER_TOKEN.MAVEN_GRADLE_SETTINGS_XML}
            </Text>
            <Box
              style={{
                background: 'var(--gray-a3)',
                border: '1px solid var(--gray-a5)',
                borderRadius: 'var(--radius-2)',
                padding: 'var(--space-3)',
                fontFamily: 'var(--font-mono, monospace)',
                fontSize: '13px',
                whiteSpace: 'pre',
                overflowX: 'auto',
              }}
              data-testid="maven-snippet"
            >
              {mavenSnippet}
            </Box>
            <CopyField label={UserPagesStrings.USER_TOKEN.COPY_MAVEN_SNIPPET} value={mavenSnippet} />
          </Box>

          <Separator size="4" />

          <CopyField label={UserPagesStrings.USER_TOKEN.BASE64_ENCODED} value={base64Value} />

          <Text size="1" color="gray" data-testid="countdown-text">
            {UserPagesStrings.USER_TOKEN.AUTO_HIDES_SECONDS(seconds)}
          </Text>
        </Flex>

        <Flex justify="end" mt="4">
          <Dialog.Close>
            <Button variant="soft" color="gray" onClick={onClose}>
              {CommonStrings.ACTIONS.CLOSE}
            </Button>
          </Dialog.Close>
        </Flex>
      </Dialog.Content>
    </Dialog.Root>
  );
}

export function UserTokenPage() {
  const {
    pageState,
    expirationTimeTimestamp,
    revealedToken,
    actionLoading,
    handleAccess,
    handleGenerate,
    handleReset,
    handleCloseReveal,
    handleRetry,
  } = useUserToken();

  const [showResetConfirm, setShowResetConfirm] = useState(false);

  const formatEpochMs = (epochMs: string) => {
    try {
      return new Date(Number(epochMs)).toLocaleDateString(pageLanguage(), {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
      });
    } catch {
      return epochMs;
    }
  };

  const expiryBadge = (epochMs: string) => {
    if (!epochMs || Number.isNaN(Number(epochMs))) return null;
    const diff = Number(epochMs) - Date.now();
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));
    if (diff < 0) return <Badge color="red" variant="soft">{UserPagesStrings.USER_TOKEN.EXPIRED}</Badge>;
    if (days < 7) return <Badge color="amber" variant="soft">{UserPagesStrings.USER_TOKEN.EXPIRES_D(days)}</Badge>;
    return null;
  };

  const handleResetConfirm = () => {
    setShowResetConfirm(false);
    handleReset();
  };

  if (pageState === 'loading') {
    return (
      <Flex direction="column" align="center" justify="center" gap="3" style={{ minHeight: 200 }} data-testid="user-token-page-loading">
        <Spinner size="3" />
        <Text size="2" color="gray">{UserPagesStrings.USER_TOKEN.LOADING_USER_TOKEN_STATUS}</Text>
      </Flex>
    );
  }

  return (
    <Box style={{ maxWidth: 640, width: '100%', padding: 'var(--space-6)' }} data-testid="user-token-page">
      <Flex align="center" gap="3" mb="4">
        <Key size={24} color="var(--accent-9)" />
        <Box>
          <Heading size="5">{UserPagesStrings.USER_TOKEN.USER_TOKEN}</Heading>
          <Text size="2" color="gray">{UserPagesStrings.USER_TOKEN.PERSONAL_ACCESS_TOKEN_API_AUTHENTICATION}</Text>
        </Box>
      </Flex>

      {pageState === 'disabled' && (
        <Callout.Root color="blue" data-testid="tokens-disabled-callout">
          <Callout.Text>
            {UserPagesStrings.USER_TOKEN.USER_TOKENS_NOT_ENABLED_INSTANCE}
          </Callout.Text>
        </Callout.Root>
      )}

      {pageState === 'error' && (
        <Callout.Root color="red" data-testid="error-state">
          <Callout.Text>
            {UserPagesStrings.USER_TOKEN.FAILED_LOAD_USER_TOKEN_STATUS}
          </Callout.Text>
          <Box mt="3">
            <Button variant="soft" color="red" onClick={handleRetry} data-testid="retry-btn">
              {CommonStrings.ACTIONS.RETRY}
            </Button>
          </Box>
        </Callout.Root>
      )}

      {(pageState === 'no-token' || pageState === 'has-token' || pageState === 'expired-token') && (
        <>
          <Card mb="4" data-testid="token-status-card">
            <Heading size="3" mb="3">{UserPagesStrings.USER_TOKEN.TOKEN_STATUS}</Heading>

            {pageState === 'no-token' && (
              <Flex direction="column" align="center" gap="3" py="5" data-testid="no-token-state">
                <KeyRound size={48} color="var(--gray-8)" />
                <Heading size="3" color="gray">{UserPagesStrings.USER_TOKEN.NO_TOKEN_GENERATED}</Heading>
                <Text size="2" color="gray" align="center">
                  {UserPagesStrings.USER_TOKEN.YOU_DONT_HAVE_USER_TOKEN}
                </Text>
                <Button
                  variant="solid"
                  color="blue"
                  onClick={handleGenerate}
                  disabled={actionLoading}
                  data-testid="generate-token-btn"
                >
                  {actionLoading && <Spinner size="1" />}
                  {UserPagesStrings.USER_TOKEN.GENERATE_TOKEN}
                </Button>
              </Flex>
            )}

            {pageState === 'has-token' && (
              <Flex direction="column" gap="3" data-testid="has-token-state">
                <Flex align="center" gap="2">
                  <CheckCircle size={20} color="var(--green-9)" />
                  <Text weight="medium" color="green">{UserPagesStrings.USER_TOKEN.TOKEN_ACTIVE}</Text>
                </Flex>
                {expirationTimeTimestamp && (
                  <Flex direction="column" gap="1">
                    <Flex gap="2" align="center" data-testid="expires-row">
                      <Text size="2" color="gray" style={{ minWidth: 70 }}>{UserPagesStrings.USER_TOKEN.EXPIRES}</Text>
                      <Text size="2">{formatEpochMs(expirationTimeTimestamp)}</Text>
                      {expiryBadge(expirationTimeTimestamp)}
                    </Flex>
                  </Flex>
                )}
                <Flex gap="2" mt="1">
                  <Button
                    variant="soft"
                    color="blue"
                    onClick={handleAccess}
                    disabled={actionLoading}
                    data-testid="access-token-btn"
                  >
                    {actionLoading && <Spinner size="1" />}
                    {UserPagesStrings.USER_TOKEN.ACCESS_TOKEN}
                  </Button>
                  <Button
                    variant="soft"
                    color="red"
                    onClick={() => setShowResetConfirm(true)}
                    disabled={actionLoading}
                    data-testid="reset-token-btn"
                  >
                    {UserPagesStrings.USER_TOKEN.RESET_TOKEN}
                  </Button>
                </Flex>
              </Flex>
            )}

            {pageState === 'expired-token' && (
              <Flex direction="column" gap="3" data-testid="expired-token-state">
                <Callout.Root color="red">
                  <Callout.Text>
                    {UserPagesStrings.USER_TOKEN.USER_TOKEN_HAS_EXPIRED_GENERATE}
                  </Callout.Text>
                </Callout.Root>
                <Flex gap="2">
                  <Button
                    variant="solid"
                    color="blue"
                    onClick={handleGenerate}
                    disabled={actionLoading}
                    data-testid="generate-token-btn"
                  >
                    {actionLoading && <Spinner size="1" />}
                    {UserPagesStrings.USER_TOKEN.GENERATE_TOKEN}
                  </Button>
                  <Button
                    variant="soft"
                    color="red"
                    onClick={() => setShowResetConfirm(true)}
                    disabled={actionLoading}
                    data-testid="reset-token-btn"
                  >
                    {UserPagesStrings.USER_TOKEN.RESET_TOKEN}
                  </Button>
                </Flex>
              </Flex>
            )}
          </Card>

          <Card data-testid="what-is-token-card">
            <Heading size="3" mb="2">{UserPagesStrings.USER_TOKEN.WHAT_USER_TOKEN}</Heading>
            <Text size="2" color="gray">
              {UserPagesStrings.USER_TOKEN.USER_TOKEN_LETS_YOU_AUTHENTICATE}
            </Text>
          </Card>
        </>
      )}

      {revealedToken && (
        <TokenRevealDialog
          nameCode={revealedToken.nameCode}
          passCode={revealedToken.passCode}
          onClose={handleCloseReveal}
        />
      )}

      <ConfirmDialog
        open={showResetConfirm}
        onOpenChange={setShowResetConfirm}
        title={UserPagesStrings.USER_TOKEN.RESET_USER_TOKEN}
        message={UserPagesStrings.USER_TOKEN.PERMANENTLY_DELETE_CURRENT_TOKEN_ANY}
        confirmLabel={UserPagesStrings.USER_TOKEN.RESET_TOKEN}
        variant="danger"
        onConfirm={handleResetConfirm}
        testId="reset-token-dialog"
      />
    </Box>
  );
}

export default UserTokenPage;
