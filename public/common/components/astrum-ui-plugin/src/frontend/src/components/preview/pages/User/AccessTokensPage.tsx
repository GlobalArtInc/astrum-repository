import React, { useState } from 'react';
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
  SegmentedControl,
  Select,
  Spinner,
  Table,
  Tabs,
  Text,
  TextField,
  Tooltip,
} from '@radix-ui/themes';
import { AlertTriangle, Check, Copy, KeyRound, Trash2 } from 'lucide-react';
import { ExtJS } from '../../../../interface/ExtJS';
import UIStrings from '../../../../constants/UIStrings';
import { ConfirmDialog } from '../../shared/ConfirmDialog';
import {
  AccessToken,
  AccessTokenScope,
  CreatedAccessToken,
  errorMessage,
  useAccessTokens,
} from './useAccessTokens';

const READ_ALL_PERMISSION = 'astrum:accesstokens:read';

const EXPIRATIONS: { value: string; days: number | null; label: () => string }[] = [
  { value: '7', days: 7, label: () => UIStrings.ACCESS_TOKENS.EXPIRATION.DAYS_7 },
  { value: '30', days: 30, label: () => UIStrings.ACCESS_TOKENS.EXPIRATION.DAYS_30 },
  { value: '90', days: 90, label: () => UIStrings.ACCESS_TOKENS.EXPIRATION.DAYS_90 },
  { value: '365', days: 365, label: () => UIStrings.ACCESS_TOKENS.EXPIRATION.DAYS_365 },
  { value: 'never', days: null, label: () => UIStrings.ACCESS_TOKENS.EXPIRATION.NEVER },
];

function language(): string | undefined {
  return (window as unknown as { AS_I18N?: { language?: string } }).AS_I18N?.language || undefined;
}

export function formatDate(value: string | null): string {
  if (!value) {
    return '';
  }
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }
  return date.toLocaleDateString(language(), { year: 'numeric', month: 'short', day: 'numeric' });
}

export function formatDateTime(value: string | null): string {
  if (!value) {
    return '';
  }
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }
  return date.toLocaleString(language(), {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

export function serverUrl(): string {
  const url = ExtJS.absolutePath('');
  return /^https?:\/\//.test(url) ? url : `${window.location.origin}${url}`;
}

export function usageExamples(baseUrl: string, user: string, secret: string) {
  const base = baseUrl.replace(/\/+$/, '');
  const host = base.replace(/^https?:\/\//, '').split('/')[0];
  const hostPath = base.replace(/^https?:\/\//, '');
  const auth = btoa(`${user}:${secret}`);
  const scheme = base.startsWith('http://') ? 'http' : 'https';
  return [
    {
      id: 'docker',
      label: 'Docker',
      code: `echo '${secret}' | docker login ${host} -u ${user} --password-stdin`,
    },
    {
      id: 'npm',
      label: 'npm',
      code: `registry=${base}/repository/<repository>/\n//${hostPath}/repository/<repository>/:_auth=${auth}`,
    },
    {
      id: 'maven',
      label: 'Maven',
      code: `<server>\n  <id>astrum</id>\n  <username>${user}</username>\n  <password>${secret}</password>\n</server>`,
    },
    {
      id: 'pip',
      label: 'pip',
      code: `pip install --index-url ${scheme}://${encodeURIComponent(user)}:${secret}@${hostPath}/repository/<repository>/simple/ <package>`,
    },
    {
      id: 'curl',
      label: 'curl',
      code: `curl -u ${user}:${secret} -O ${base}/repository/<repository>/<path>`,
    },
  ];
}

function CopyButton({ value, testId }: { value: string; testId?: string }) {
  const [copied, setCopied] = useState(false);
  const label = copied ? UIStrings.ACCESS_TOKENS.COPIED : UIStrings.ACCESS_TOKENS.COPY;
  const handleCopy = () => {
    navigator.clipboard?.writeText(value);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };
  return (
    <Tooltip content={label}>
      <IconButton variant="ghost" size="2" onClick={handleCopy} aria-label={label} data-testid={testId}>
        {copied ? <Check size={14} color="var(--green-9)" /> : <Copy size={14} />}
      </IconButton>
    </Tooltip>
  );
}

function CodeBlock({ code, testId }: { code: string; testId?: string }) {
  return (
    <Flex align="start" gap="2">
      <Box
        style={{
          flex: 1,
          background: 'var(--gray-2)',
          borderRadius: 'var(--radius-2)',
          padding: 'var(--space-3)',
          fontFamily: 'var(--code-font-family, monospace)',
          fontSize: '13px',
          whiteSpace: 'pre',
          overflowX: 'auto',
        }}
        data-testid={testId}
      >
        {code}
      </Box>
      <CopyButton value={code} />
    </Flex>
  );
}

function CreatedTokenDialog({ created, user, onClose }: { created: CreatedAccessToken; user: string; onClose: () => void }) {
  const strings = UIStrings.ACCESS_TOKENS.CREATED;
  const examples = usageExamples(serverUrl(), user, created.secret);
  return (
    <Dialog.Root open onOpenChange={(open) => !open && onClose()}>
      <Dialog.Content maxWidth="640px" data-testid="access-token-created-dialog">
        <Dialog.Title>{strings.TITLE}</Dialog.Title>
        <Callout.Root color="amber" mb="4">
          <Callout.Icon>
            <AlertTriangle size={16} />
          </Callout.Icon>
          <Callout.Text>{strings.WARNING}</Callout.Text>
        </Callout.Root>

        <Text size="1" color="gray" as="div" mb="1">
          {strings.TOKEN_LABEL}
        </Text>
        <Flex align="center" gap="2" mb="4">
          <Code size="3" variant="soft" style={{ flex: 1, wordBreak: 'break-all' }} data-testid="access-token-secret">
            {created.secret}
          </Code>
          <CopyButton value={created.secret} testId="access-token-copy" />
        </Flex>

        <Heading size="3" mb="1">
          {strings.USAGE_TITLE}
        </Heading>
        <Text size="2" color="gray" as="p" mb="3">
          {strings.USAGE_HINT}
        </Text>
        <Tabs.Root defaultValue="docker">
          <Tabs.List>
            {examples.map((example) => (
              <Tabs.Trigger key={example.id} value={example.id}>
                {example.label}
              </Tabs.Trigger>
            ))}
          </Tabs.List>
          {examples.map((example) => (
            <Tabs.Content key={example.id} value={example.id}>
              <Box pt="3">
                <CodeBlock code={example.code} testId={`access-token-usage-${example.id}`} />
              </Box>
            </Tabs.Content>
          ))}
        </Tabs.Root>

        <Flex justify="end" mt="4">
          <Button onClick={onClose} data-testid="access-token-created-done">
            {strings.DONE}
          </Button>
        </Flex>
      </Dialog.Content>
    </Dialog.Root>
  );
}

function CreateTokenForm({ onCreate }: { onCreate: (name: string, scope: AccessTokenScope, days: number | null) => Promise<void> }) {
  const strings = UIStrings.ACCESS_TOKENS;
  const [name, setName] = useState('');
  const [scope, setScope] = useState<AccessTokenScope>('read');
  const [expiration, setExpiration] = useState('90');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const submit = async (event: React.FormEvent) => {
    event.preventDefault();
    if (!name.trim()) {
      setError(strings.CREATE.NAME_REQUIRED);
      return;
    }
    setSaving(true);
    setError(null);
    try {
      await onCreate(name.trim(), scope, EXPIRATIONS.find((option) => option.value === expiration)?.days ?? null);
      setName('');
    } catch (e) {
      setError(errorMessage(e, strings.ERRORS.CREATE));
    } finally {
      setSaving(false);
    }
  };

  return (
    <Card mb="5" data-testid="access-token-create">
      <form onSubmit={submit}>
        <Heading size="3" mb="3">
          {strings.CREATE.TITLE}
        </Heading>
        <Flex gap="3" wrap="wrap" align="end">
          <Box style={{ flex: '2 1 220px' }}>
            <Text as="label" size="2" weight="medium" htmlFor="access-token-name">
              {strings.CREATE.NAME_LABEL}
            </Text>
            <TextField.Root
              id="access-token-name"
              mt="1"
              value={name}
              maxLength={100}
              placeholder={strings.CREATE.NAME_PLACEHOLDER}
              onChange={(event) => setName(event.target.value)}
              data-testid="access-token-name"
            />
          </Box>
          <Box style={{ flex: '1 1 180px' }}>
            <Text as="div" size="2" weight="medium" mb="1">
              {strings.CREATE.SCOPE_LABEL}
            </Text>
            <Select.Root value={scope} onValueChange={(value) => setScope(value as AccessTokenScope)}>
              <Select.Trigger style={{ width: '100%' }} data-testid="access-token-scope" />
              <Select.Content>
                <Select.Item value="read">{strings.SCOPES.read}</Select.Item>
                <Select.Item value="write">{strings.SCOPES.write}</Select.Item>
              </Select.Content>
            </Select.Root>
          </Box>
          <Box style={{ flex: '1 1 140px' }}>
            <Text as="div" size="2" weight="medium" mb="1">
              {strings.CREATE.EXPIRATION_LABEL}
            </Text>
            <Select.Root value={expiration} onValueChange={setExpiration}>
              <Select.Trigger style={{ width: '100%' }} data-testid="access-token-expiration" />
              <Select.Content>
                {EXPIRATIONS.map((option) => (
                  <Select.Item key={option.value} value={option.value}>
                    {option.label()}
                  </Select.Item>
                ))}
              </Select.Content>
            </Select.Root>
          </Box>
          <Button type="submit" disabled={saving} data-testid="access-token-create-button">
            {saving && <Spinner size="1" />}
            {strings.CREATE.BUTTON}
          </Button>
        </Flex>
        <Text size="1" color="gray" as="p" mt="2">
          {scope === 'read' ? strings.SCOPES.READ_DESCRIPTION : strings.SCOPES.WRITE_DESCRIPTION}
        </Text>
        {error && (
          <Callout.Root color="red" mt="3" data-testid="access-token-create-error">
            <Callout.Text>{error}</Callout.Text>
          </Callout.Root>
        )}
      </form>
    </Card>
  );
}

function TokensTable({
  tokens,
  showOwner,
  onRevoke,
}: {
  tokens: AccessToken[];
  showOwner: boolean;
  onRevoke: (token: AccessToken) => void;
}) {
  const strings = UIStrings.ACCESS_TOKENS.LIST;
  return (
    <Table.Root variant="surface" data-testid="access-token-table">
      <Table.Header>
        <Table.Row>
          <Table.ColumnHeaderCell>{strings.NAME}</Table.ColumnHeaderCell>
          {showOwner && <Table.ColumnHeaderCell>{strings.OWNER}</Table.ColumnHeaderCell>}
          <Table.ColumnHeaderCell>{strings.SCOPE}</Table.ColumnHeaderCell>
          <Table.ColumnHeaderCell>{strings.TOKEN}</Table.ColumnHeaderCell>
          <Table.ColumnHeaderCell>{strings.CREATED}</Table.ColumnHeaderCell>
          <Table.ColumnHeaderCell>{strings.LAST_USED}</Table.ColumnHeaderCell>
          <Table.ColumnHeaderCell>{strings.EXPIRES}</Table.ColumnHeaderCell>
          <Table.ColumnHeaderCell />
        </Table.Row>
      </Table.Header>
      <Table.Body>
        {tokens.map((token) => (
          <Table.Row key={token.id} data-testid={`access-token-row-${token.name}`}>
            <Table.RowHeaderCell>
              <Text weight="medium">{token.name}</Text>
            </Table.RowHeaderCell>
            {showOwner && <Table.Cell>{token.owner}</Table.Cell>}
            <Table.Cell>
              <Badge color={token.scope === 'write' ? 'orange' : 'blue'} variant="soft">
                {UIStrings.ACCESS_TOKENS.SCOPES[token.scope] ?? token.scope}
              </Badge>
            </Table.Cell>
            <Table.Cell>
              <Code variant="ghost">{token.hint}</Code>
            </Table.Cell>
            <Table.Cell>{formatDate(token.created)}</Table.Cell>
            <Table.Cell>
              {token.lastUsed ? formatDateTime(token.lastUsed) : <Text color="gray">{strings.NEVER_USED}</Text>}
            </Table.Cell>
            <Table.Cell>
              {token.expired ? (
                <Badge color="red" variant="soft">
                  {strings.EXPIRED}
                </Badge>
              ) : token.expires ? (
                formatDate(token.expires)
              ) : (
                <Text color="gray">{strings.NEVER_EXPIRES}</Text>
              )}
            </Table.Cell>
            <Table.Cell align="right">
              <Tooltip content={strings.REVOKE}>
                <IconButton
                  variant="ghost"
                  color="red"
                  aria-label={`${strings.REVOKE} ${token.name}`}
                  onClick={() => onRevoke(token)}
                  data-testid={`access-token-revoke-${token.name}`}
                >
                  <Trash2 size={16} />
                </IconButton>
              </Tooltip>
            </Table.Cell>
          </Table.Row>
        ))}
      </Table.Body>
    </Table.Root>
  );
}

export function AccessTokensPage() {
  const strings = UIStrings.ACCESS_TOKENS;
  const user = ExtJS.useUser();
  const userId: string = user?.id ?? user?.userId ?? '';
  const canSeeAll = Boolean(ExtJS.usePermission(() => ExtJS.checkPermission(READ_ALL_PERMISSION), [userId]));
  const [view, setView] = useState<'mine' | 'all'>('mine');
  const showAll = canSeeAll && view === 'all';
  const { tokens, loading, error, create, revoke } = useAccessTokens(showAll);
  const [created, setCreated] = useState<CreatedAccessToken | null>(null);
  const [revoking, setRevoking] = useState<AccessToken | null>(null);
  const [revokeError, setRevokeError] = useState<string | null>(null);

  const handleCreate = async (name: string, scope: AccessTokenScope, days: number | null) => {
    setCreated(await create({ name, scope, expiresInDays: days }));
  };

  const handleRevoke = async () => {
    const token = revoking;
    if (!token) {
      return;
    }
    setRevokeError(null);
    try {
      await revoke(token.id);
      ExtJS.showSuccessMessage(strings.REVOKE.SUCCESS(token.name));
    } catch (e) {
      setRevokeError(errorMessage(e, strings.ERRORS.REVOKE));
    }
  };

  return (
    <Box style={{ maxWidth: 1100, margin: '0 auto', padding: 'var(--space-5)' }} data-testid="access-tokens-page">
      <Flex align="center" gap="3" mb="2">
        <KeyRound size={24} color="var(--accent-9)" />
        <Heading size="6">{strings.TITLE}</Heading>
      </Flex>
      <Text size="2" color="gray" as="p" mb="5" style={{ maxWidth: 760 }}>
        {strings.DESCRIPTION}
      </Text>

      <CreateTokenForm onCreate={handleCreate} />

      {canSeeAll && (
        <SegmentedControl.Root
          value={view}
          onValueChange={(value) => setView(value as 'mine' | 'all')}
          mb="3"
          data-testid="access-token-view"
        >
          <SegmentedControl.Item value="mine">{strings.LIST.MINE}</SegmentedControl.Item>
          <SegmentedControl.Item value="all">{strings.LIST.ALL}</SegmentedControl.Item>
        </SegmentedControl.Root>
      )}

      {revokeError && (
        <Callout.Root color="red" mb="3">
          <Callout.Text>{revokeError}</Callout.Text>
        </Callout.Root>
      )}

      {loading && tokens.length === 0 ? (
        <Flex justify="center" p="6">
          <Spinner size="3" />
        </Flex>
      ) : error ? (
        <Callout.Root color="red" data-testid="access-token-load-error">
          <Callout.Text>{errorMessage(error, strings.ERRORS.LOAD)}</Callout.Text>
        </Callout.Root>
      ) : tokens.length === 0 ? (
        <Card>
          <Text color="gray" data-testid="access-token-empty">
            {showAll ? strings.LIST.EMPTY_ALL : strings.LIST.EMPTY}
          </Text>
        </Card>
      ) : (
        <TokensTable tokens={tokens} showOwner={showAll} onRevoke={setRevoking} />
      )}

      {created && <CreatedTokenDialog created={created} user={userId} onClose={() => setCreated(null)} />}

      <ConfirmDialog
        open={revoking !== null}
        onOpenChange={(open) => !open && setRevoking(null)}
        title={strings.REVOKE.TITLE}
        message={revoking ? strings.REVOKE.MESSAGE(revoking.name) : ''}
        confirmLabel={strings.REVOKE.CONFIRM}
        cancelLabel={strings.REVOKE.CANCEL}
        onConfirm={handleRevoke}
      />
    </Box>
  );
}

export default AccessTokensPage;
