import { Button, Callout, Card, Flex, Spinner, Text } from '@radix-ui/themes';
import { TriangleAlert } from 'lucide-react';
import { useState } from 'react';
import { ApiError, api, useApi, useInvalidateApi } from '../../../api';
import { useT } from '../../../i18n';
import {
  CheckboxField,
  DateTime,
  ErrorState,
  type Errors,
  type FieldBinding,
  FormError,
  MultiSelectField,
  NumberField,
  Page,
  SwitchField,
  TextField,
  UnsavedChangesGuard,
  useForm,
  useToast,
} from '../../../ui';
import { ROLES, type Role } from '../api';
import { ReadOnlyValue } from '../shared/Edit';
import formStyles from '../shared/Form.module.css';
import { explainRefusal } from '../shared/messages';
import { ListEditor } from '../ssrf/ListEditor';
import { TWO_FACTOR, type TwoFactorPolicy } from './api';
import {
  GRACE_DAYS_MAX,
  noMethodWarning,
  type PolicyForm,
  serverProblems,
  toForm,
  toPolicy,
  validatePolicy,
} from './model';
import styles from './TwoFactor.module.css';

export interface PolicyEditorProps {
  policy: TwoFactorPolicy;
  roles: Role[];
}

function PolicyEditor({ policy, roles }: PolicyEditorProps) {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const [serverErrors, setServerErrors] = useState<Errors<PolicyForm>>({});
  const form = useForm<PolicyForm>({
    initial: toForm(policy),
    validate: (values) => validatePolicy(t, values),
    onSubmit: (values) =>
      explainRefusal(t, async () => {
        setServerErrors({});
        try {
          await api.put<TwoFactorPolicy>(TWO_FACTOR, toPolicy(values));
        } catch (error) {
          if (error instanceof ApiError && error.status === 400 && error.fieldErrors.length > 0) {
            const problems = serverProblems(error);
            setServerErrors(problems.fields);
            throw new Error(problems.unmatched.join(' ') || t('security.twoFactor.fixFields'));
          }
          throw error;
        }
        toast.success(t('security.twoFactor.saved'));
        await invalidate(TWO_FACTOR);
      }),
  });

  const bind = <K extends keyof PolicyForm & string>(name: K): FieldBinding<PolicyForm[K]> => ({
    name,
    value: form.values[name],
    error: serverErrors[name] ?? form.errors[name],
    onChange: (value) => {
      form.set(name, value);
      setServerErrors((current) => {
        const next = { ...current };
        delete next[name];
        return next;
      });
    },
  });

  const origins = bind('origins');
  const roleOptions = roles.map((role) => ({ value: role.id, label: role.id }));
  const warn = noMethodWarning(form.values);
  return (
    <form className={formStyles.form} onSubmit={form.submit} noValidate>
      <Card size="3">
        <div className={formStyles.card}>
          <SwitchField
            label={t('security.twoFactor.totp')}
            description={t('security.twoFactor.totpText')}
            {...bind('totp')}
          />
          <SwitchField
            label={t('security.twoFactor.passkeys')}
            description={t('security.twoFactor.passkeysText')}
            disabled
            {...bind('passkeys')}
          />
          {warn && (
            <Callout.Root color="amber" role="status">
              <Callout.Icon>
                <TriangleAlert size={16} />
              </Callout.Icon>
              <Callout.Text>{t('security.twoFactor.noMethodWarning')}</Callout.Text>
            </Callout.Root>
          )}
          <fieldset className={styles.group}>
            <legend className={styles.legend}>
              <Text size="2" weight="medium">
                {t('security.twoFactor.requiredFor')}
              </Text>
            </legend>
            <CheckboxField label={t('security.twoFactor.everyone')} {...bind('everyone')} />
            <CheckboxField label={t('security.twoFactor.administrators')} {...bind('administrators')} />
            <MultiSelectField
              label={t('security.twoFactor.roles')}
              hint={t('security.twoFactor.rolesHint')}
              options={roleOptions}
              {...bind('roles')}
            />
          </fieldset>
          <NumberField
            label={t('security.twoFactor.graceDays')}
            hint={t('security.twoFactor.graceDaysHint')}
            min={0}
            max={GRACE_DAYS_MAX}
            step={1}
            {...bind('graceDays')}
          />
          <ReadOnlyValue
            label={t('security.twoFactor.requiredSince')}
            value={
              form.values.requiredSince ? (
                <DateTime value={form.values.requiredSince} dateOnly />
              ) : (
                t('security.twoFactor.notSet')
              )
            }
          />
          {form.values.passkeys && (
            <div className={formStyles.card}>
              <TextField
                label={t('security.twoFactor.relyingPartyId')}
                hint={t('security.twoFactor.relyingPartyIdHint')}
                placeholder="example.org"
                {...bind('relyingPartyId')}
              />
              <ListEditor
                label={t('security.twoFactor.origins')}
                hint={t('security.twoFactor.originsHint')}
                placeholder="https://sso.example.org"
                values={form.values.origins}
                onChange={origins.onChange}
                check={() => undefined}
                error={origins.error}
                emptyText={t('security.twoFactor.noOrigins')}
              />
            </div>
          )}
        </div>
      </Card>
      <FormError message={form.formError} />
      <div className={formStyles.actions}>
        <Button type="submit" loading={form.submitting} disabled={!form.dirty}>
          {t('security.form.save')}
        </Button>
        <Button type="button" variant="soft" color="gray" disabled={!form.dirty} onClick={() => form.reset()}>
          {t('security.form.discard')}
        </Button>
      </div>
      <UnsavedChangesGuard when={form.dirty} />
    </form>
  );
}

export default function TwoFactorPolicyPage() {
  const t = useT();
  const policy = useApi<TwoFactorPolicy>(TWO_FACTOR);
  const roles = useApi<Role[]>(ROLES);
  return (
    <Page
      title={t('security.twoFactor.title')}
      description={t('security.twoFactor.intro')}
      breadcrumbs={[
        { label: t('security.crumbs.settings'), to: '/settings' },
        { label: t('security.twoFactor.title') },
      ]}
    >
      {policy.isPending ? (
        <Flex justify="center" py="8">
          <Spinner size="3" />
        </Flex>
      ) : policy.error ? (
        <ErrorState error={policy.error} onRetry={() => void policy.refetch()} />
      ) : (
        <PolicyEditor policy={policy.data} roles={roles.data ?? []} />
      )}
    </Page>
  );
}
