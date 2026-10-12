import { Button, Callout, Card, Dialog, Flex, Heading, RadioGroup, Text } from '@radix-ui/themes';
import { KeyRound, LogOut, Plus, ShieldAlert, Trash2, TriangleAlert } from 'lucide-react';
import { useMemo, useState } from 'react';
import { api, path, useApi, useInvalidateApi } from '../../api';
import { useSession } from '../../auth/state';
import { useSignOut } from '../../auth/useSignOut';
import { useT } from '../../i18n';
import {
  CodeBlock,
  type Column,
  ConfirmDialog,
  CopyButton,
  DataTable,
  DateTime,
  EmptyState,
  FieldShell,
  FormError,
  NumberField,
  Page,
  RelativeTime,
  StatusBadge,
  TextField,
  useForm,
  useToast,
} from '../../ui';
import {
  type AccessToken,
  basicExample,
  type CreatedToken,
  DAY_PRESETS,
  MAX_DAYS,
  MIN_DAYS,
  needsPasswordSignIn,
  type Scope,
  TOKENS,
  type TokenValues,
  tokenExpired,
  toTokenRequest,
  validateToken,
} from './model';
import styles from './User.module.css';

const INITIAL: TokenValues = { name: '', scope: 'READ', days: 90 };

function CreateTokenForm({ onCreated }: { onCreated(token: CreatedToken): void }) {
  const t = useT();
  const invalidate = useInvalidateApi();
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
  return (
    <form className={styles.form} onSubmit={form.submit} noValidate aria-labelledby="tokens-create">
      <Card size="3">
        <div className={styles.card}>
          <Heading as="h2" size="4" id="tokens-create">
            {t('user.tokens.createTitle')}
          </Heading>
          <TextField
            label={t('user.tokens.name')}
            hint={t('user.tokens.nameHint')}
            {...form.field('name')}
            required
            maxLength={200}
            autoComplete="off"
          />
          <FieldShell id="token-scope" label={t('user.tokens.scope')} required error={form.errors.scope}>
            <RadioGroup.Root
              id="token-scope"
              name="scope"
              value={form.values.scope}
              onValueChange={(value) => form.set('scope', value as Scope)}
              aria-label={t('user.tokens.scope')}
            >
              <RadioGroup.Item value="READ">
                <Flex direction="column">
                  <Text weight="medium">{t('user.tokens.scopeRead')}</Text>
                  <Text size="1" color="gray">
                    {t('user.tokens.scopeReadText')}
                  </Text>
                </Flex>
              </RadioGroup.Item>
              <RadioGroup.Item value="WRITE">
                <Flex direction="column">
                  <Text weight="medium">{t('user.tokens.scopeWrite')}</Text>
                  <Text size="1" color="gray">
                    {t('user.tokens.scopeWriteText')}
                  </Text>
                </Flex>
              </RadioGroup.Item>
            </RadioGroup.Root>
          </FieldShell>
          <div className={styles.days}>
            <NumberField
              label={t('user.tokens.days')}
              hint={t('user.tokens.daysHint', { min: MIN_DAYS, max: MAX_DAYS })}
              {...form.field('days')}
              min={MIN_DAYS}
              max={MAX_DAYS}
              step={1}
              required
            />
            <Flex gap="2" wrap="wrap" role="group" aria-label={t('user.tokens.presets')}>
              {DAY_PRESETS.map((days) => (
                <Button
                  key={days}
                  type="button"
                  size="1"
                  variant={form.values.days === days ? 'solid' : 'soft'}
                  color="gray"
                  aria-pressed={form.values.days === days}
                  onClick={() => form.set('days', days)}
                >
                  {t('user.tokens.preset', { count: days })}
                </Button>
              ))}
            </Flex>
          </div>
          <FormError message={form.formError} />
          <div className={styles.actions}>
            <Button type="submit" loading={form.submitting}>
              <Plus size={16} aria-hidden="true" />
              {t('user.tokens.create')}
            </Button>
          </div>
        </div>
      </Card>
    </form>
  );
}

function CreatedDialog({ token, onClose }: { token: CreatedToken | null; onClose(): void }) {
  const t = useT();
  const { state, user } = useSession();
  return (
    <Dialog.Root open={token !== null} onOpenChange={(open) => !open && onClose()}>
      <Dialog.Content maxWidth="640px">
        <Dialog.Title>{t('user.tokens.createdTitle')}</Dialog.Title>
        <Dialog.Description size="2" color="gray" mb="3">
          {t('user.tokens.createdText', { name: token?.name ?? '' })}
        </Dialog.Description>
        {token && (
          <div className={styles.dialogBody}>
            <Callout.Root color="amber" role="alert">
              <Callout.Icon>
                <TriangleAlert size={16} />
              </Callout.Icon>
              <Callout.Text>{t('user.tokens.once')}</Callout.Text>
            </Callout.Root>
            <FieldShell id="created-token" label={t('user.tokens.token')}>
              <div className={styles.secret}>
                <code id="created-token" className={styles.token} data-testid="created-token">
                  {token.token}
                </code>
                <CopyButton text={token.token} label={t('user.tokens.copy')} size="2" />
              </div>
            </FieldShell>
            <Text size="2" weight="medium">
              {t('user.tokens.usage')}
            </Text>
            <Text size="2" color="gray">
              {t('user.tokens.usageText')}
            </Text>
            <CodeBlock code={basicExample(state.baseUrl, user?.userId ?? token.userId, token.token)} />
          </div>
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

export default function AccessTokensPage() {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const tokens = useApi<AccessToken[]>(TOKENS);
  const [created, setCreated] = useState<CreatedToken | null>(null);
  const [revoking, setRevoking] = useState<AccessToken | null>(null);
  const now = useMemo(() => new Date(), []);
  const blocked = needsPasswordSignIn(tokens.error);

  const columns: Column<AccessToken>[] = [
    {
      id: 'name',
      header: t('user.tokens.name'),
      cell: (row) => row.name,
      sortValue: (row) => row.name,
      primary: true,
    },
    {
      id: 'scope',
      header: t('user.tokens.scope'),
      cell: (row) => (
        <StatusBadge tone={row.scope === 'WRITE' ? 'warning' : 'info'}>
          {row.scope === 'WRITE' ? t('user.tokens.scopeWrite') : t('user.tokens.scopeRead')}
        </StatusBadge>
      ),
      sortValue: (row) => row.scope,
    },
    {
      id: 'hint',
      header: t('user.tokens.hint'),
      cell: (row) => <code className={styles.mono}>{`${row.hint}…`}</code>,
      hideOnPhone: true,
    },
    {
      id: 'created',
      header: t('user.tokens.created'),
      cell: (row) => <DateTime value={row.createdAt} dateOnly />,
      sortValue: (row) => new Date(row.createdAt),
      hideOnPhone: true,
    },
    {
      id: 'expires',
      header: t('user.tokens.expires'),
      cell: (row) =>
        tokenExpired(row, now) ? (
          <StatusBadge tone="danger">{t('user.tokens.expired')}</StatusBadge>
        ) : (
          <DateTime value={row.expiresAt} dateOnly />
        ),
      sortValue: (row) => new Date(row.expiresAt),
    },
    {
      id: 'used',
      header: t('user.tokens.lastUsed'),
      cell: (row) =>
        row.lastUsedAt ? (
          <RelativeTime value={row.lastUsedAt} />
        ) : (
          <Text color="gray">{t('user.tokens.neverUsed')}</Text>
        ),
      sortValue: (row) => (row.lastUsedAt ? new Date(row.lastUsedAt) : null),
    },
  ];

  const revoke = async () => {
    if (!revoking) {
      return;
    }
    await api.delete(`${TOKENS}${path`/${revoking.id}`}`);
    toast.success(t('user.tokens.revoked', { name: revoking.name }));
    await invalidate(TOKENS);
  };

  return (
    <Page title={t('user.tokens.title')} description={t('user.tokens.intro')}>
      {blocked ? (
        <PasswordRequired />
      ) : (
        <>
          <Callout.Root color="gray" variant="surface">
            <Callout.Icon>
              <KeyRound size={16} />
            </Callout.Icon>
            <Callout.Text>{t('user.tokens.rule')}</Callout.Text>
          </Callout.Root>
          <CreateTokenForm onCreated={setCreated} />
          <section className={styles.section} aria-labelledby="tokens-list">
            <Heading as="h2" size="4" id="tokens-list">
              {t('user.tokens.listTitle')}
            </Heading>
            <DataTable
              label={t('user.tokens.listTitle')}
              columns={columns}
              rows={tokens.data}
              getRowId={(row) => row.id}
              loading={tokens.isPending}
              error={tokens.error}
              onRetry={() => void tokens.refetch()}
              initialSort={{ column: 'created', direction: 'desc' }}
              empty={
                <EmptyState
                  icon={KeyRound}
                  title={t('user.tokens.empty')}
                  description={t('user.tokens.emptyText')}
                />
              }
              rowActions={() => [
                {
                  id: 'revoke',
                  label: t('user.tokens.revoke'),
                  icon: Trash2,
                  danger: true,
                  onSelect: setRevoking,
                },
              ]}
            />
          </section>
        </>
      )}
      <CreatedDialog token={created} onClose={() => setCreated(null)} />
      <ConfirmDialog
        open={revoking !== null}
        onOpenChange={(open) => !open && setRevoking(null)}
        onConfirm={revoke}
        title={t('user.tokens.revokeTitle', { name: revoking?.name ?? '' })}
        description={t('user.tokens.revokeText')}
        confirmLabel={t('user.tokens.revoke')}
        danger
      />
    </Page>
  );
}
