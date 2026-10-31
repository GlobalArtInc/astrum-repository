import {
  Badge,
  Button,
  Callout,
  Card,
  Code,
  Dialog,
  Flex,
  SegmentedControl,
  Select,
  Table,
  Tabs,
  Text,
  TextField,
} from '@radix-ui/themes';
import { KeyRound, LogOut, ShieldAlert, Trash2, TriangleAlert } from 'lucide-react';
import { useId, useMemo, useState } from 'react';
import { api, path, useApi, useInvalidateApi, withQuery } from '../../api';
import { useCan, useSession } from '../../auth/state';
import { useSignOut } from '../../auth/useSignOut';
import { useT } from '../../i18n';
import {
  ConfirmDialog,
  CopyButton,
  DateTime,
  ErrorState,
  useDocumentTitle,
  useForm,
  useToast,
} from '../../ui';
import { ListPager, SearchBox, usePaging } from '../security/shared/AdminList';
import { AgentGrants } from './AgentGrants';
import {
  type AccessToken,
  type CreatedToken,
  filterTokens,
  LIFETIMES,
  type Lifetime,
  needsPasswordSignIn,
  type Scope,
  sortNewestFirst,
  TOKENS,
  type TokenValues,
  toTokenRequest,
  USAGE_KINDS,
  type UsageKind,
  usageExamples,
  validateToken,
} from './model';
import styles from './Tokens.module.css';

const TOKENS_PAGE_SIZE = 20;
const INITIAL: TokenValues = { name: '', scope: 'READ', days: 90 };
const USAGE_LABELS: Record<UsageKind, string> = {
  docker: 'Docker',
  podman: 'Podman',
  npm: 'npm',
  maven: 'Maven',
  pip: 'pip',
  curl: 'curl',
};

function lifetimeKey(days: Lifetime): string {
  return days === null ? 'never' : String(days);
}

function CreateTokenForm({ onCreated }: { onCreated(token: CreatedToken): void }) {
  const t = useT();
  const invalidate = useInvalidateApi();
  const nameId = useId();
  const scopeId = useId();
  const daysId = useId();
  const form = useForm<TokenValues>({
    initial: INITIAL,
    validate: (values) => validateToken(t, values),
    onSubmit: async (values) => {
      const created = await api.post<CreatedToken>(TOKENS, toTokenRequest(values));
      form.reset(INITIAL);
      await invalidate(TOKENS);
      onCreated(created);
    },
  });
  const lifetimeLabel = (days: Lifetime) =>
    days === null
      ? t('user.tokens.never')
      : days === 365
        ? t('user.tokens.year')
        : t('user.tokens.days', { count: days });
  const nameError = form.errors.name;
  return (
    <Card size="1" className={styles.card}>
      <form onSubmit={form.submit} noValidate aria-labelledby={`${nameId}-title`}>
        <h2 id={`${nameId}-title`} className={styles.cardTitle}>
          {t('user.tokens.createTitle')}
        </h2>
        <div className={styles.row}>
          <div className={`${styles.field} ${styles.nameField}`}>
            <label htmlFor={nameId} className={styles.label}>
              {t('user.tokens.name')}
            </label>
            <TextField.Root
              id={nameId}
              className={styles.nameInput}
              name="name"
              value={form.values.name}
              placeholder={t('user.tokens.namePlaceholder')}
              maxLength={200}
              autoComplete="off"
              aria-invalid={nameError ? true : undefined}
              aria-describedby={nameError ? `${nameId}-error` : undefined}
              onChange={(event) => form.set('name', event.target.value)}
            />
            {nameError && (
              <p id={`${nameId}-error`} className={styles.error} role="alert">
                {nameError}
              </p>
            )}
          </div>
          <div className={`${styles.field} ${styles.scopeField}`}>
            <span id={scopeId} className={styles.label}>
              {t('user.tokens.scope')}
            </span>
            <Select.Root
              value={form.values.scope}
              onValueChange={(value) => form.set('scope', value as Scope)}
            >
              <Select.Trigger
                className={styles.select}
                aria-labelledby={scopeId}
                aria-describedby={`${scopeId}-hint`}
              />
              <Select.Content position="popper">
                <Select.Item value="READ">{t('user.tokens.scopeRead')}</Select.Item>
                <Select.Item value="WRITE">{t('user.tokens.scopeWrite')}</Select.Item>
              </Select.Content>
            </Select.Root>
            <p id={`${scopeId}-hint`} className={styles.hint}>
              {form.values.scope === 'WRITE'
                ? t('user.tokens.scopeWriteText')
                : t('user.tokens.scopeReadText')}
            </p>
          </div>
          <div className={`${styles.field} ${styles.daysField}`}>
            <span id={daysId} className={styles.label}>
              {t('user.tokens.expiry')}
            </span>
            <Select.Root
              value={lifetimeKey(form.values.days)}
              onValueChange={(value) => {
                const index = LIFETIMES.findIndex((days) => lifetimeKey(days) === value);
                form.set('days', index >= 0 ? (LIFETIMES[index] ?? null) : 90);
              }}
            >
              <Select.Trigger className={styles.select} aria-labelledby={daysId} />
              <Select.Content position="popper">
                {LIFETIMES.map((days) => (
                  <Select.Item key={lifetimeKey(days)} value={lifetimeKey(days)}>
                    {lifetimeLabel(days)}
                  </Select.Item>
                ))}
              </Select.Content>
            </Select.Root>
          </div>
          <Button type="submit" className={styles.create} loading={form.submitting}>
            {t('user.tokens.create')}
          </Button>
        </div>
        {form.formError && (
          <p className={styles.error} role="alert">
            {form.formError}
          </p>
        )}
      </form>
    </Card>
  );
}

function CodeSample({ code }: { code: string }) {
  const t = useT();
  return (
    <div className={styles.sample}>
      <pre className={styles.sampleCode}>{code}</pre>
      <CopyButton text={code} label={t('user.tokens.copy')} />
    </div>
  );
}

function CreatedDialog({ token, onClose }: { token: CreatedToken | null; onClose(): void }) {
  const t = useT();
  const { state, user } = useSession();
  const examples = token ? usageExamples(state.baseUrl, user?.userId ?? token.userId, token.token) : null;
  return (
    <Dialog.Root open={token !== null} onOpenChange={(open) => !open && onClose()}>
      <Dialog.Content maxWidth="640px" className={styles.dialog}>
        <Dialog.Title className={styles.dialogTitle}>{t('user.tokens.createdTitle')}</Dialog.Title>
        <Dialog.Description className={styles.hidden}>
          {t('user.tokens.createdText', { name: token?.name ?? '' })}
        </Dialog.Description>
        {token && examples && (
          <>
            <Callout.Root color="amber" role="alert" className={styles.once}>
              <Callout.Icon>
                <TriangleAlert size={16} />
              </Callout.Icon>
              <Callout.Text>{t('user.tokens.once')}</Callout.Text>
            </Callout.Root>
            <div className={styles.secretLabel}>{t('user.tokens.token')}</div>
            <div className={styles.secret}>
              <Code size="3" variant="soft" className={styles.token} data-testid="created-token">
                {token.token}
              </Code>
              <CopyButton text={token.token} label={t('user.tokens.copy')} />
            </div>
            <h3 className={styles.usageTitle}>{t('user.tokens.usage')}</h3>
            <p className={styles.usageText}>{t('user.tokens.usageText')}</p>
            <Tabs.Root defaultValue="docker">
              <Tabs.List>
                {USAGE_KINDS.map((kind) => (
                  <Tabs.Trigger key={kind} value={kind}>
                    {USAGE_LABELS[kind]}
                  </Tabs.Trigger>
                ))}
              </Tabs.List>
              {USAGE_KINDS.map((kind) => (
                <Tabs.Content key={kind} value={kind} className={styles.tab}>
                  <CodeSample code={examples[kind]} />
                </Tabs.Content>
              ))}
            </Tabs.Root>
          </>
        )}
        <Flex justify="end" mt="4">
          <Dialog.Close>
            <Button>{t('user.tokens.done')}</Button>
          </Dialog.Close>
        </Flex>
      </Dialog.Content>
    </Dialog.Root>
  );
}

function PasswordRequired() {
  const t = useT();
  const signOut = useSignOut();
  return (
    <Callout.Root color="amber" size="2">
      <Callout.Icon>
        <ShieldAlert size={18} />
      </Callout.Icon>
      <Flex direction="column" gap="2" align="start">
        <Text as="p" size="2" weight="medium">
          {t('user.tokens.passwordFirstTitle')}
        </Text>
        <Text as="p" size="2">
          {t('user.tokens.passwordFirst')}
        </Text>
        <Button size="1" variant="soft" color="amber" onClick={() => void signOut()}>
          <LogOut size={14} aria-hidden="true" />
          {t('user.tokens.signOut')}
        </Button>
      </Flex>
    </Callout.Root>
  );
}

function TokenTable({
  tokens,
  everyone,
  canRevoke,
  onRevoke,
}: {
  tokens: AccessToken[];
  everyone: boolean;
  canRevoke(token: AccessToken): boolean;
  onRevoke(token: AccessToken): void;
}) {
  const t = useT();
  return (
    <Table.Root variant="surface" className={styles.table} aria-label={t('user.tokens.title')}>
      <Table.Header>
        <Table.Row>
          <Table.ColumnHeaderCell>{t('user.tokens.name')}</Table.ColumnHeaderCell>
          {everyone && <Table.ColumnHeaderCell>{t('user.tokens.user')}</Table.ColumnHeaderCell>}
          <Table.ColumnHeaderCell>{t('user.tokens.scope')}</Table.ColumnHeaderCell>
          <Table.ColumnHeaderCell>{t('user.tokens.token')}</Table.ColumnHeaderCell>
          <Table.ColumnHeaderCell>{t('user.tokens.created')}</Table.ColumnHeaderCell>
          <Table.ColumnHeaderCell>{t('user.tokens.lastUsed')}</Table.ColumnHeaderCell>
          <Table.ColumnHeaderCell>{t('user.tokens.expires')}</Table.ColumnHeaderCell>
          <Table.ColumnHeaderCell>
            <span className={styles.hidden}>{t('user.tokens.actions')}</span>
          </Table.ColumnHeaderCell>
        </Table.Row>
      </Table.Header>
      <Table.Body>
        {tokens.map((token) => (
          <Table.Row key={token.id}>
            <Table.RowHeaderCell>
              <span className={styles.name}>{token.name}</span>
            </Table.RowHeaderCell>
            {everyone && <Table.Cell>{token.userId}</Table.Cell>}
            <Table.Cell>
              <Badge size="1">
                {token.scope === 'WRITE' ? t('user.tokens.scopeWrite') : t('user.tokens.scopeRead')}
              </Badge>
            </Table.Cell>
            <Table.Cell>
              <Code variant="ghost">{`${token.hint}…`}</Code>
            </Table.Cell>
            <Table.Cell>
              <DateTime value={token.createdAt} dateOnly />
            </Table.Cell>
            <Table.Cell>
              {token.lastUsedAt ? (
                <DateTime value={token.lastUsedAt} dateOnly />
              ) : (
                <span className={styles.muted}>{t('user.tokens.neverUsed')}</span>
              )}
            </Table.Cell>
            <Table.Cell>
              {token.expiresAt ? <DateTime value={token.expiresAt} dateOnly /> : t('user.tokens.never')}
            </Table.Cell>
            <Table.Cell className={styles.actions}>
              {canRevoke(token) && (
                <button
                  type="button"
                  className={styles.revoke}
                  aria-label={t('user.tokens.revokeOne', { name: token.name })}
                  title={t('user.tokens.revoke')}
                  onClick={() => onRevoke(token)}
                >
                  <Trash2 size={16} aria-hidden="true" />
                </button>
              )}
            </Table.Cell>
          </Table.Row>
        ))}
      </Table.Body>
    </Table.Root>
  );
}

export default function AccessTokensPage() {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const { user } = useSession();
  const canReadAll = useCan('astrum:accesstokens:read');
  const canDeleteAll = useCan('astrum:accesstokens:delete');
  const [everyone, setEveryone] = useState(false);
  const showAll = everyone && canReadAll;
  const tokens = useApi<AccessToken[]>(showAll ? withQuery(TOKENS, { userId: '*' }) : TOKENS);
  const [created, setCreated] = useState<CreatedToken | null>(null);
  const [revoking, setRevoking] = useState<AccessToken | null>(null);
  const [query, setQuery] = useState('');
  const blocked = needsPasswordSignIn(tokens.error);
  const rows = useMemo(() => filterTokens(sortNewestFirst(tokens.data ?? []), query), [tokens.data, query]);
  const paging = usePaging(rows, TOKENS_PAGE_SIZE);
  useDocumentTitle(t('user.tokens.title'));

  const revoke = async () => {
    if (!revoking) {
      return;
    }
    await api.delete(`${TOKENS}${path`/${revoking.id}`}`);
    toast.success(t('user.tokens.revoked', { name: revoking.name }));
    await invalidate(TOKENS);
  };

  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <div className={styles.titleRow}>
          <KeyRound size={24} aria-hidden="true" className={styles.icon} />
          <h1 className={styles.title}>{t('user.tokens.title')}</h1>
        </div>
        <p className={styles.intro}>{t('user.tokens.intro')}</p>
      </header>
      {blocked ? (
        <PasswordRequired />
      ) : (
        <>
          <CreateTokenForm onCreated={setCreated} />
          {canReadAll && (
            <SegmentedControl.Root
              className={styles.switch}
              value={showAll ? 'all' : 'mine'}
              onValueChange={(value) => {
                setEveryone(value === 'all');
                paging.setPage(0);
              }}
            >
              <SegmentedControl.Item value="mine">{t('user.tokens.mine')}</SegmentedControl.Item>
              <SegmentedControl.Item value="all">{t('user.tokens.everyone')}</SegmentedControl.Item>
            </SegmentedControl.Root>
          )}
          {tokens.error ? (
            <ErrorState error={tokens.error} onRetry={() => void tokens.refetch()} />
          ) : tokens.data && tokens.data.length > 0 ? (
            <>
              <SearchBox
                value={query}
                onChange={(value) => {
                  setQuery(value);
                  paging.setPage(0);
                }}
                placeholder={t('user.tokens.search')}
              />
              {rows.length > 0 ? (
                <>
                  <TokenTable
                    tokens={paging.visible}
                    everyone={showAll}
                    canRevoke={(token) => token.userId === user?.userId || canDeleteAll}
                    onRevoke={setRevoking}
                  />
                  <ListPager
                    range={t('user.tokens.range', { from: paging.from, to: paging.to, total: paging.total })}
                    page={paging.page}
                    pages={paging.pages}
                    onPage={paging.setPage}
                  />
                </>
              ) : (
                <Card size="1" className={styles.empty}>
                  {t('user.tokens.noMatches')}
                </Card>
              )}
            </>
          ) : (
            <Card size="1" className={styles.empty}>
              {tokens.isPending ? '' : showAll ? t('user.tokens.emptyAll') : t('user.tokens.empty')}
            </Card>
          )}
        </>
      )}
      <AgentGrants />
      <CreatedDialog token={created} onClose={() => setCreated(null)} />
      <ConfirmDialog
        open={revoking !== null}
        onOpenChange={(open) => !open && setRevoking(null)}
        onConfirm={revoke}
        title={t('user.tokens.revokeTitle')}
        description={t('user.tokens.revokeText', { name: revoking?.name ?? '' })}
        confirmLabel={t('user.tokens.revoke')}
        danger
      />
    </div>
  );
}
