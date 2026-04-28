import { Badge, Button, Callout, Flex, Spinner, TextField } from '@radix-ui/themes';
import { useQueryClient } from '@tanstack/react-query';
import { Info, Shield, User } from 'lucide-react';
import { type ReactNode, useId } from 'react';
import { api, useApi, useInvalidateApi } from '../../api';
import { STATE_KEY } from '../../auth/state';
import { useT } from '../../i18n';
import { ErrorState, FormError, UnsavedChangesGuard, useDocumentTitle, useForm, useToast } from '../../ui';
import styles from './Account.module.css';
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

const SOURCE_LABELS: Record<string, string> = {
  default: 'user.account.sourceLocal',
  LDAP: 'user.account.sourceLdap',
  OAuth2: 'user.account.sourceOidc',
};

function Field({
  label,
  help,
  error,
  children,
}: {
  label: string;
  help?: string;
  error?: string;
  children(props: { id: string; describedBy?: string; invalid?: boolean }): ReactNode;
}) {
  const id = useId();
  const describedBy = [help ? `${id}-help` : '', error ? `${id}-error` : ''].filter(Boolean).join(' ');
  return (
    <div className={styles.field}>
      <label htmlFor={id} className={styles.label}>
        {label}
        <span className={styles.required} aria-hidden="true">
          *
        </span>
      </label>
      {help && (
        <p id={`${id}-help`} className={styles.help}>
          {help}
        </p>
      )}
      {children({ id, describedBy: describedBy || undefined, invalid: error ? true : undefined })}
      {error && (
        <p id={`${id}-error`} className={styles.error} role="alert">
          {error}
        </p>
      )}
    </div>
  );
}

function InfoRow({ icon, label, children }: { icon: ReactNode; label: string; children: ReactNode }) {
  return (
    <div className={styles.info}>
      <span className={styles.infoIcon}>{icon}</span>
      <div className={styles.infoText}>
        <span className={styles.infoLabel}>{label}</span>
        <div className={styles.infoValue}>{children}</div>
      </div>
    </div>
  );
}

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
  const sourceKey = SOURCE_LABELS[account.source];
  return (
    <form className={styles.section} onSubmit={form.submit} noValidate aria-labelledby="account-details">
      <h2 className={styles.heading} id="account-details">
        {t('user.account.details')}
      </h2>
      <div className={styles.infoList}>
        <InfoRow icon={<User size={16} aria-hidden="true" />} label={t('user.account.userId')}>
          <span className={styles.userId}>{account.userId}</span>
        </InfoRow>
        <InfoRow icon={<Shield size={16} aria-hidden="true" />} label={t('user.account.source')}>
          <Badge size="1">{sourceKey ? t(sourceKey) : account.source}</Badge>
        </InfoRow>
      </div>
      {readOnly && (
        <Callout.Root color="gray" variant="surface">
          <Callout.Icon>
            <Info size={16} />
          </Callout.Icon>
          <Callout.Text>{t('user.account.external')}</Callout.Text>
        </Callout.Root>
      )}
      <div className={styles.fields}>
        <div className={styles.pair}>
          <Field label={t('user.account.firstName')} error={form.errors.firstName}>
            {({ id, describedBy, invalid }) => (
              <TextField.Root
                id={id}
                name="firstName"
                value={form.values.firstName}
                disabled={readOnly}
                autoComplete="given-name"
                aria-describedby={describedBy}
                aria-invalid={invalid}
                onChange={(event) => form.set('firstName', event.target.value)}
              />
            )}
          </Field>
          <Field label={t('user.account.lastName')} error={form.errors.lastName}>
            {({ id, describedBy, invalid }) => (
              <TextField.Root
                id={id}
                name="lastName"
                value={form.values.lastName}
                disabled={readOnly}
                autoComplete="family-name"
                aria-describedby={describedBy}
                aria-invalid={invalid}
                onChange={(event) => form.set('lastName', event.target.value)}
              />
            )}
          </Field>
        </div>
        <Field label={t('user.account.email')} error={form.errors.email}>
          {({ id, describedBy, invalid }) => (
            <TextField.Root
              id={id}
              type="email"
              name="email"
              value={form.values.email}
              disabled={readOnly}
              autoComplete="email"
              aria-describedby={describedBy}
              aria-invalid={invalid}
              onChange={(event) => form.set('email', event.target.value)}
            />
          )}
        </Field>
      </div>
      <FormError message={form.formError} />
      {!readOnly && (
        <div className={styles.actions}>
          <Button type="submit" loading={form.submitting} disabled={!form.dirty}>
            {t('user.account.save')}
          </Button>
          <Button type="button" variant="outline" disabled={!form.dirty} onClick={() => form.reset()}>
            {t('user.account.discard')}
          </Button>
        </div>
      )}
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
  const { values } = form;
  const filled = values.currentPassword !== '' && values.newPassword !== '' && values.confirm !== '';
  const touched = values.currentPassword !== '' || values.newPassword !== '' || values.confirm !== '';
  const input =
    (name: keyof PasswordValues, autoComplete: string) =>
    ({ id, describedBy, invalid }: { id: string; describedBy?: string; invalid?: boolean }) => (
      <TextField.Root
        id={id}
        type="password"
        name={name}
        value={values[name]}
        autoComplete={autoComplete}
        aria-describedby={describedBy}
        aria-invalid={invalid}
        onChange={(event) => form.set(name, event.target.value)}
      />
    );
  return (
    <form className={styles.section} onSubmit={form.submit} noValidate aria-labelledby="account-password">
      <h2 className={styles.heading} id="account-password">
        {t('user.password.title')}
      </h2>
      <p className={styles.intro}>{t('user.password.intro', { count: MIN_PASSWORD })}</p>
      <input type="text" name="username" autoComplete="username" hidden readOnly />
      <div className={styles.fields}>
        <Field label={t('user.password.current')} error={form.errors.currentPassword}>
          {input('currentPassword', 'current-password')}
        </Field>
        <Field
          label={t('user.password.new')}
          help={t('user.password.hint', { count: MIN_PASSWORD })}
          error={form.errors.newPassword}
        >
          {input('newPassword', 'new-password')}
        </Field>
        <Field label={t('user.password.confirm')} error={form.errors.confirm}>
          {input('confirm', 'new-password')}
        </Field>
      </div>
      <FormError message={form.formError} />
      <div className={`${styles.actions} ${styles.passwordActions}`}>
        <Button type="submit" loading={form.submitting} disabled={!filled}>
          {t('user.password.submit')}
        </Button>
        <Button
          type="button"
          variant="outline"
          disabled={!touched}
          onClick={() => form.reset(EMPTY_PASSWORD)}
        >
          {t('user.password.discard')}
        </Button>
      </div>
    </form>
  );
}

export default function AccountPage() {
  const t = useT();
  const account = useApi<Account>(ACCOUNT);
  useDocumentTitle(t('user.account.title'));
  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <User size={24} aria-hidden="true" className={styles.icon} />
        <div>
          <h1 className={styles.title}>{t('user.account.title')}</h1>
          <span className={styles.subtitle}>{t('user.account.intro')}</span>
        </div>
      </header>
      {account.isPending ? (
        <Flex justify="center" py="8">
          <Spinner size="3" />
        </Flex>
      ) : account.error ? (
        <ErrorState error={account.error} onRetry={() => void account.refetch()} />
      ) : (
        <>
          <AccountForm account={account.data} />
          {!account.data.readOnly && (
            <>
              <hr className={styles.rule} />
              <PasswordForm />
            </>
          )}
        </>
      )}
    </div>
  );
}
