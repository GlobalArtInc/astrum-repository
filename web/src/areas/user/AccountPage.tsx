import { Badge, Button, Callout, Card, DataList, Flex, Heading, Spinner, Text } from '@radix-ui/themes';
import { useQueryClient } from '@tanstack/react-query';
import { Info } from 'lucide-react';
import { api, useApi, useInvalidateApi } from '../../api';
import { STATE_KEY } from '../../auth/state';
import { useT } from '../../i18n';
import {
  ErrorState,
  FormError,
  Page,
  PasswordField,
  TextField,
  UnsavedChangesGuard,
  useForm,
  useToast,
} from '../../ui';
import {
  ACCOUNT,
  ACCOUNT_PASSWORD,
  type Account,
  type AccountValues,
  fromAccount,
  MIN_PASSWORD,
  type PasswordValues,
  passwordFailure,
  toAccountRequest,
  validateAccount,
  validatePassword,
} from './model';
import styles from './User.module.css';

const SOURCE_LABELS: Record<string, string> = {
  default: 'user.account.sourceLocal',
  LDAP: 'user.account.sourceLdap',
  OAuth2: 'user.account.sourceOidc',
};

function AccountForm({ account }: { account: Account }) {
  const t = useT();
  const toast = useToast();
  const client = useQueryClient();
  const invalidate = useInvalidateApi();
  const form = useForm<AccountValues>({
    initial: fromAccount(account),
    validate: (values) => validateAccount(t, values),
    onSubmit: async (values) => {
      try {
        await api.put(ACCOUNT, toAccountRequest(values));
      } catch (error) {
        throw passwordFailure(t, error);
      }
      toast.success(t('user.account.saved'));
      await Promise.all([invalidate(ACCOUNT), client.invalidateQueries({ queryKey: STATE_KEY })]);
    },
  });
  const readOnly = account.readOnly;
  return (
    <form className={styles.form} onSubmit={form.submit} noValidate aria-labelledby="account-details">
      <Card size="3">
        <div className={styles.card}>
          <Heading as="h2" size="4" id="account-details">
            {t('user.account.details')}
          </Heading>
          <DataList.Root orientation={{ initial: 'vertical', sm: 'horizontal' }}>
            <DataList.Item>
              <DataList.Label>{t('user.account.userId')}</DataList.Label>
              <DataList.Value>
                <code className={styles.mono}>{account.userId}</code>
              </DataList.Value>
            </DataList.Item>
            <DataList.Item>
              <DataList.Label>{t('user.account.source')}</DataList.Label>
              <DataList.Value>
                <Badge variant="soft">
                  {SOURCE_LABELS[account.source] ? t(SOURCE_LABELS[account.source] ?? '') : account.source}
                </Badge>
              </DataList.Value>
            </DataList.Item>
            <DataList.Item>
              <DataList.Label>{t('user.account.roles')}</DataList.Label>
              <DataList.Value>
                {account.roles.length === 0 ? (
                  <Text color="gray">{t('user.account.noRoles')}</Text>
                ) : (
                  <Flex gap="1" wrap="wrap">
                    {account.roles.map((role) => (
                      <Badge key={role} variant="outline" color="gray">
                        {role}
                      </Badge>
                    ))}
                  </Flex>
                )}
              </DataList.Value>
            </DataList.Item>
          </DataList.Root>
          {readOnly && (
            <Callout.Root color="gray" variant="surface">
              <Callout.Icon>
                <Info size={16} />
              </Callout.Icon>
              <Callout.Text>{t('user.account.external')}</Callout.Text>
            </Callout.Root>
          )}
          <div className={styles.pair}>
            <TextField
              label={t('user.account.firstName')}
              {...form.field('firstName')}
              required
              disabled={readOnly}
              autoComplete="given-name"
            />
            <TextField
              label={t('user.account.lastName')}
              {...form.field('lastName')}
              required
              disabled={readOnly}
              autoComplete="family-name"
            />
          </div>
          <TextField
            label={t('user.account.email')}
            type="email"
            {...form.field('email')}
            required
            disabled={readOnly}
            autoComplete="email"
          />
          <FormError message={form.formError} />
          {!readOnly && (
            <div className={styles.actions}>
              <Button type="submit" loading={form.submitting} disabled={!form.dirty}>
                {t('user.account.save')}
              </Button>
              <Button
                type="button"
                variant="soft"
                color="gray"
                disabled={!form.dirty}
                onClick={() => form.reset()}
              >
                {t('user.account.discard')}
              </Button>
            </div>
          )}
        </div>
      </Card>
      <UnsavedChangesGuard when={form.dirty} />
    </form>
  );
}

const EMPTY_PASSWORD: PasswordValues = { currentPassword: '', newPassword: '', confirm: '' };

function PasswordForm() {
  const t = useT();
  const toast = useToast();
  const form = useForm<PasswordValues>({
    initial: EMPTY_PASSWORD,
    validate: (values) => validatePassword(t, values),
    onSubmit: async (values) => {
      try {
        await api.put(ACCOUNT_PASSWORD, {
          currentPassword: values.currentPassword,
          newPassword: values.newPassword,
        });
      } catch (error) {
        throw passwordFailure(t, error);
      }
      toast.success(t('user.password.changed'));
      form.reset(EMPTY_PASSWORD);
    },
  });
  return (
    <form className={styles.form} onSubmit={form.submit} noValidate aria-labelledby="account-password">
      <Card size="3">
        <div className={styles.card}>
          <Flex direction="column" gap="1">
            <Heading as="h2" size="4" id="account-password">
              {t('user.password.title')}
            </Heading>
            <Text size="2" color="gray">
              {t('user.password.intro')}
            </Text>
          </Flex>
          <input type="text" name="username" autoComplete="username" hidden readOnly />
          <PasswordField
            label={t('user.password.current')}
            {...form.field('currentPassword')}
            required
            autoComplete="current-password"
          />
          <div className={styles.pair}>
            <PasswordField
              label={t('user.password.new')}
              {...form.field('newPassword')}
              required
              autoComplete="new-password"
              hint={t('user.password.hint', { count: MIN_PASSWORD })}
            />
            <PasswordField
              label={t('user.password.confirm')}
              {...form.field('confirm')}
              required
              autoComplete="new-password"
            />
          </div>
          <FormError message={form.formError} />
          <div className={styles.actions}>
            <Button type="submit" loading={form.submitting}>
              {t('user.password.submit')}
            </Button>
          </div>
        </div>
      </Card>
    </form>
  );
}

export default function AccountPage() {
  const t = useT();
  const account = useApi<Account>(ACCOUNT);
  return (
    <Page title={t('user.account.title')} description={t('user.account.intro')}>
      {account.isPending ? (
        <Flex justify="center" py="8">
          <Spinner size="3" />
        </Flex>
      ) : account.error ? (
        <ErrorState error={account.error} onRetry={() => void account.refetch()} />
      ) : (
        <>
          <AccountForm account={account.data} />
          {!account.data.readOnly && <PasswordForm />}
        </>
      )}
    </Page>
  );
}
