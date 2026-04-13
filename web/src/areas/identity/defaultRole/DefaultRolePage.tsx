import { Button, Callout, Flex, Spinner } from '@radix-ui/themes';
import { useQueryClient } from '@tanstack/react-query';
import { CircleAlert, Trash2 } from 'lucide-react';
import { type FormEvent, useState } from 'react';
import { ApiError, api, useApi, useInvalidateApi } from '../../../api';
import { STATE_KEY, useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import {
  ConfirmDialog,
  describeError,
  ErrorState,
  FormError,
  Page,
  UnsavedChangesGuard,
  useToast,
} from '../../../ui';
import { DEFAULT_ROLE, type DefaultRole, ROLES, type Role } from '../api';
import oidc from '../oidc/Oidc.module.css';
import { Check, Choice, Section } from '../shared/Fields';
import styles from './DefaultRole.module.css';

interface Values {
  enabled: boolean;
  role: string;
}

function DefaultRoleForm({ current, roles }: { current: DefaultRole; roles: Role[] }) {
  const t = useT();
  const toast = useToast();
  const client = useQueryClient();
  const invalidate = useInvalidateApi();
  const canWrite = useCan('astrum:settings:update');
  const start: Values = { enabled: current.role === null ? true : current.enabled, role: current.role ?? '' };
  const [baseline, setBaseline] = useState(start);
  const [values, setValues] = useState(start);
  const [stored, setStored] = useState(current.role);
  const [error, setError] = useState<string | undefined>();
  const [formError, setFormError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [removing, setRemoving] = useState(false);
  const dirty = JSON.stringify(values) !== JSON.stringify(baseline);
  const readOnly = !canWrite;
  const missing = values.role !== '' && !roles.some((role) => role.id === values.role);
  const options = [...roles]
    .sort((left, right) => left.name.localeCompare(right.name))
    .map((role) => ({
      value: role.id,
      label: role.name === role.id ? role.id : `${role.name} (${role.id})`,
    }));
  if (missing) {
    options.unshift({ value: values.role, label: values.role });
  }

  const refresh = () =>
    Promise.all([invalidate(DEFAULT_ROLE), client.invalidateQueries({ queryKey: STATE_KEY })]);

  const save = async (event: FormEvent) => {
    event.preventDefault();
    if (values.role === '') {
      setError(t('identity.defaultRole.required'));
      return;
    }
    setSaving(true);
    setFormError(null);
    try {
      await api.put(DEFAULT_ROLE, { role: values.role, enabled: values.enabled });
      setBaseline(values);
      setStored(values.role);
      toast.success(t('identity.defaultRole.saved'));
      await refresh();
    } catch (failure) {
      if (failure instanceof ApiError && failure.status === 400 && failure.fieldErrors.length > 0) {
        setError(failure.fieldErrors.map((problem) => problem.message).join(' '));
      } else {
        setFormError(describeError(t, failure));
      }
    } finally {
      setSaving(false);
    }
  };

  const remove = async () => {
    await api.delete(DEFAULT_ROLE);
    const cleared = { enabled: true, role: '' };
    setBaseline(cleared);
    setValues(cleared);
    setStored(null);
    toast.success(t('identity.defaultRole.removed'));
    await refresh();
  };

  return (
    <form onSubmit={save} noValidate>
      {!readOnly && (
        <div className={`${oidc.bar} ${styles.bar}`}>
          <div>
            {stored !== null && (
              <Button type="button" variant="soft" color="red" onClick={() => setRemoving(true)}>
                <Trash2 size={16} aria-hidden="true" />
                {t('identity.defaultRole.remove')}
              </Button>
            )}
          </div>
          <Flex gap="3">
            <Button
              type="button"
              variant="outline"
              onClick={() => {
                setValues(baseline);
                setError(undefined);
                setFormError(null);
              }}
            >
              {t('identity.defaultRole.discard')}
            </Button>
            <Button type="submit" loading={saving} disabled={!dirty && stored !== null}>
              {t('identity.defaultRole.save')}
            </Button>
          </Flex>
        </div>
      )}
      <div className={oidc.body}>
        <Section title={t('identity.defaultRole.settings')} id="default-role-settings">
          {current.missing && values.role === current.role && (
            <Callout.Root color="amber" role="status">
              <Callout.Icon>
                <CircleAlert size={16} aria-hidden="true" />
              </Callout.Icon>
              <Callout.Text>{t('identity.defaultRole.missing', { role: values.role })}</Callout.Text>
            </Callout.Root>
          )}
          <Check
            label={t('identity.defaultRole.enabled')}
            name="enabled"
            checked={values.enabled}
            disabled={readOnly}
            onChange={(checked) => setValues((current) => ({ ...current, enabled: checked }))}
          />
          <Choice
            label={t('identity.defaultRole.role')}
            help={t('identity.defaultRole.roleHelp')}
            required
            name="role"
            disabled={readOnly}
            placeholder={t('identity.defaultRole.placeholder')}
            value={values.role}
            error={error}
            onChange={(role) => {
              setValues((current) => ({ ...current, role }));
              setError(undefined);
            }}
            options={options}
          />
        </Section>
        <FormError message={formError} />
      </div>
      <UnsavedChangesGuard when={dirty && !saving && !readOnly} />
      <ConfirmDialog
        open={removing}
        onOpenChange={setRemoving}
        onConfirm={remove}
        title={t('identity.defaultRole.removeTitle')}
        description={t('identity.defaultRole.removeText', { role: stored ?? '' })}
        confirmLabel={t('identity.defaultRole.remove')}
        danger
      />
    </form>
  );
}

export default function DefaultRolePage() {
  const t = useT();
  const current = useApi<DefaultRole>(DEFAULT_ROLE);
  const roles = useApi<Role[]>(ROLES);
  const error = current.error ?? roles.error;
  return (
    <Page
      title={t('identity.defaultRole.title')}
      description={t('identity.defaultRole.description')}
      breadcrumbs={[
        { label: t('identity.crumbs.settings'), to: '/settings' },
        { label: t('identity.defaultRole.title') },
      ]}
    >
      {error ? (
        <ErrorState error={error} />
      ) : current.data && roles.data ? (
        <DefaultRoleForm current={current.data} roles={roles.data} />
      ) : (
        <Flex justify="center" py="8">
          <Spinner size="3" />
        </Flex>
      )}
    </Page>
  );
}
