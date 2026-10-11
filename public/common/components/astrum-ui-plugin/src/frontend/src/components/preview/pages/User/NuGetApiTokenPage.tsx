

import React, { useState } from 'react';
import {
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
import { Check, Copy, Package } from 'lucide-react';
import { ExtJS } from '../../../../interface/ExtJS';
import { ConfirmDialog } from '../../shared/form';
import { UsageBlock } from '../../shared/Usage';
import { useNuGetApiToken } from './useNuGetApiToken';
import UserPagesStrings from '../../constants/pages/user/UserPagesStrings';
import CommonStrings from '../../constants/CommonStrings';

function CopyField({ label, value }: { label: string; value: string }) {
  const [copied, setCopied] = useState(false);
  const handleCopy = () => {
    navigator.clipboard.writeText(value);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };
  return (
    <Box>
      {label && (
        <Text size="1" color="gray" style={{ display: 'block', marginBottom: '4px' }}>
          {label}
        </Text>
      )}
      <Flex align="center" gap="2">
        <Code
          variant="soft"
          size="2"
          style={{ flex: 1, wordBreak: 'break-all' }}
          data-testid={`copy-field-${label.toLowerCase().replace(/\s+/g, '-')}`}
        >
          {value}
        </Code>
        <Tooltip content={copied ? UserPagesStrings.NUGET_API_KEY.COPIED : UserPagesStrings.NUGET_API_KEY.COPY_CLIPBOARD}>
          <IconButton
            variant="ghost"
            size="2"
            onClick={handleCopy}
            aria-label={UserPagesStrings.NUGET_API_KEY.COPY_LABEL(label || UserPagesStrings.NUGET_API_KEY.VALUE_FALLBACK)}
            data-testid={`copy-btn-${label.toLowerCase().replace(/\s+/g, '-')}`}
          >
            {copied ? <Check size={14} color="var(--green-9)" /> : <Copy size={14} />}
          </IconButton>
        </Tooltip>
      </Flex>
    </Box>
  );
}

interface NuGetRevealDialogProps {
  apiKey: string;
  userId: string;
  onClose: () => void;
}

function NuGetRevealDialog({ apiKey, userId, onClose }: NuGetRevealDialogProps) {
  const repoUrl = `${window.location.origin}/repository/{your-nuget-repo}/index.json`;
  const dotnetCmd = `dotnet nuget add source "Astrum" \\\n  --source ${repoUrl} \\\n  --username ${userId} \\\n  --password ${apiKey}`;

  return (
    <Dialog.Root open onOpenChange={(open) => { if (!open) onClose(); }}>
      <Dialog.Content maxWidth="520px" data-testid="nuget-reveal-dialog">
        <Dialog.Title>{UserPagesStrings.NUGET_API_KEY.NUGET_API_KEY}</Dialog.Title>

        <Callout.Root color="amber" mb="4">
          <Callout.Text>
            {UserPagesStrings.NUGET_API_KEY.KEEP_KEY_SECRET_STORE_NUGET}
          </Callout.Text>
        </Callout.Root>

        <Flex direction="column" gap="4">
          <CopyField label={UserPagesStrings.NUGET_API_KEY.API_KEY} value={apiKey} />

          <Separator size="4" />

          <Box data-testid="dotnet-command">
            <UsageBlock
              title={UserPagesStrings.NUGET_API_KEY.DOTNET_NUGET_ADD_SOURCE_COMMAND}
              snippets={[{ id: 'dotnet', label: 'dotnet', snippetText: dotnetCmd }]}
            />
          </Box>
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

const API_KEY_PLACEHOLDER = '{API_KEY}';

export function NuGetApiTokenPage() {
  const user = ExtJS.useUser();
  const userId = user?.id ?? user?.userId ?? '{userId}';

  const {
    apiKey,
    showReveal,
    accessLoading,
    resetLoading,
    handleAccess,
    handleReset,
    handleCloseReveal,
  } = useNuGetApiToken();

  const [showResetConfirm, setShowResetConfirm] = useState(false);

  const repoUrl = `${window.location.origin}/repository/{your-nuget-repo}/index.json`;
  const displayKey = apiKey ?? API_KEY_PLACEHOLDER;

  const dotnetCmdPreview = `dotnet nuget add source "Astrum" \\\n  --source ${repoUrl} \\\n  --username ${userId} \\\n  --password ${displayKey}`;
  const nugetCmdPreview = `nuget.exe setapikey ${displayKey} \\\n  -Source ${repoUrl}`;
  const chocoCmdPreview = `choco source add --name="Astrum" \\\n  --source="${repoUrl}" \\\n  --user="${userId}" \\\n  --password="${displayKey}"`;

  const handleResetConfirm = () => {
    setShowResetConfirm(false);
    handleReset();
  };

  return (
    <Box style={{ maxWidth: 640, width: '100%', padding: 'var(--space-6)' }} data-testid="nuget-api-token-page">
      <Flex align="center" gap="3" mb="4">
        <Package size={24} color="var(--accent-9)" />
        <Box>
          <Heading size="5">{UserPagesStrings.NUGET_API_KEY.NUGET_API_KEY}</Heading>
          <Text size="2" color="gray">{UserPagesStrings.NUGET_API_KEY.AUTHENTICATION_KEY_NUGET_PACKAGE_FEEDS}</Text>
        </Box>
      </Flex>

      <Card mb="4" data-testid="nuget-key-card">
        <Heading size="3" mb="2">{UserPagesStrings.NUGET_API_KEY.API_KEY}</Heading>
        <Text size="2" color="gray" mb="3" style={{ display: 'block' }}>
          {UserPagesStrings.NUGET_API_KEY.ACCESS_NUGET_API_KEY_AUTHENTICATE}
        </Text>
        <Flex gap="2">
          <Button
            variant="solid"
            color="blue"
            onClick={handleAccess}
            disabled={accessLoading || resetLoading}
            data-testid="access-api-key-btn"
          >
            {accessLoading && <Spinner size="1" />}
            {UserPagesStrings.NUGET_API_KEY.ACCESS_API_KEY}
          </Button>
          <Button
            variant="soft"
            color="red"
            onClick={() => setShowResetConfirm(true)}
            disabled={accessLoading || resetLoading}
            data-testid="reset-api-key-btn"
          >
            {resetLoading && <Spinner size="1" />}
            {UserPagesStrings.NUGET_API_KEY.RESET_API_KEY}
          </Button>
        </Flex>
      </Card>

      <Card data-testid="usage-instructions-card">
        <UsageBlock
          title={UserPagesStrings.NUGET_API_KEY.USAGE_INSTRUCTIONS}
          snippets={[
            { id: 'dotnet', label: UserPagesStrings.NUGET_API_KEY.DOTNET_NUGET_CLI, snippetText: dotnetCmdPreview },
            { id: 'nuget', label: UserPagesStrings.NUGET_API_KEY.NUGET_EXE_CLI, snippetText: nugetCmdPreview },
            { id: 'choco', label: UserPagesStrings.NUGET_API_KEY.CHOCOLATEY_CLI, snippetText: chocoCmdPreview },
          ]}
        />
      </Card>

      {showReveal && apiKey && (
        <NuGetRevealDialog
          apiKey={apiKey}
          userId={userId}
          onClose={handleCloseReveal}
        />
      )}

      <ConfirmDialog
        open={showResetConfirm}
        onOpenChange={setShowResetConfirm}
        title={UserPagesStrings.NUGET_API_KEY.RESET_NUGET_API_KEY}
        message={UserPagesStrings.NUGET_API_KEY.PERMANENTLY_INVALIDATE_CURRENT_NUGET_API}
        confirmLabel={UserPagesStrings.NUGET_API_KEY.RESET_KEY}
        variant="danger"
        onConfirm={handleResetConfirm}
        testId="reset-nuget-key-dialog"
      />
    </Box>
  );
}

export default NuGetApiTokenPage;
