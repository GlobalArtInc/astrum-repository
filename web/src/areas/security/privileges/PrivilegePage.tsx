import { Button, Card, CheckboxGroup, Code, Flex, Spinner, Text } from '@radix-ui/themes';
import { Trash2 } from 'lucide-react';
import { useState } from 'react';
import { Link, useParams } from 'react-router';
import { api, path, useApi, useInvalidateApi } from '../../../api';
import { useCan } from '../../../auth/state';
import { type Translate, useT } from '../../../i18n';
import {
  ConfirmDialog,
  ErrorState,
  FieldShell,
  FormError,
  formatName,
  type Option,
  Page,
  SelectField,
  TextField,
  UnsavedChangesGuard,
  useForm,
  useToast,
} from '../../../ui';
import {
  type Action,
  type ContentSelector,
  PRIVILEGE_TYPES,
  PRIVILEGES,
  type Privilege,
  type PrivilegeField,
  type PrivilegeType,
  REPOSITORIES,
  type Repository,
  ROLES,
  SELECTORS,
} from '../api';
import { BuiltInNote } from '../shared/BuiltInNote';
import formStyles from '../shared/Form.module.css';
import { explainRefusal } from '../shared/messages';
import { useLeave } from '../shared/useLeave';
import { actionLabel, fieldHint, fieldLabel, typeLabel } from './labels';
import {
  ANY,
  EMPTY_VALUES,
  fromPrivilege,
  type PrivilegeValues,
  permissionOf,
  toggleAction,
  toRequest,
  validatePrivilege,
} from './model';
import styles from './Privileges.module.css';

const LIST = '/admin/security/privileges';

export function formatOptions(t: Translate, values: readonly string[] | undefined): Option[] {
  return (values ?? [ANY]).map((value) => ({
    value,
    label: value === ANY ? t('security.privilege.anyFormat') : formatName(value),
  }));
}

export function repositoryOptions(
  t: Translate,
  repositories: readonly Repository[],
  format: string,
  current: string,
): Option[] {
  const names = repositories
    .filter((repository) => format === ANY || repository.format === format)
    .map((repository) => repository.name)
    .sort((left, right) => left.localeCompare(right));
  if (current !== ANY && current !== '' && !names.includes(current)) {
    names.unshift(current);
  }
  return [
    { value: ANY, label: t('security.privilege.anyRepository') },
    ...names.map((name) => ({ value: name, label: name })),
  ];
}

interface Leaving {
  leaving: boolean;
  leave(to: string): void;
}

function PrivilegeEditor({
  type,
  privilege,
  leaving,
  leave,
}: { type: PrivilegeType; privilege?: Privilege } & Leaving) {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const creating = privilege === undefined;
  const canWrite = useCan(creating ? 'astrum:privileges:create' : 'astrum:privileges:update');
  const canDelete = useCan('astrum:privileges:delete');
  const canReadSelectors = useCan('astrum:selectors:read');
  const builtIn = privilege?.readOnly === true;
  const readOnly = builtIn || !canWrite;
  const [deleting, setDeleting] = useState(false);
  const needsRepositories = type.fields.some((field) => field.kind === 'repository');
  const needsSelectors = type.fields.some((field) => field.kind === 'contentSelector');
  const repositories = useApi<Repository[]>(needsRepositories ? REPOSITORIES : null);
  const selectors = useApi<ContentSelector[]>(needsSelectors && canReadSelectors ? SELECTORS : null);

  const form = useForm<PrivilegeValues>({
    initial: privilege ? fromPrivilege(privilege) : EMPTY_VALUES,
    validate: (values) => validatePrivilege(t, type, values),
    onSubmit: (values) =>
      explainRefusal(t, async () => {
        const body = toRequest(type, values);
        if (creating) {
          await api.post(`${PRIVILEGES}${path`/${type.id}`}`, body);
          toast.success(t('security.privilege.created', { name: values.name.trim() }));
        } else {
          await api.put(`${PRIVILEGES}${path`/${type.id}/${privilege.name}`}`, body);
          toast.success(t('security.privilege.saved', { name: privilege.name }));
        }
        await invalidate(PRIVILEGES);
        if (creating) {
          leave(LIST);
        }
      }),
  });

  const remove = async () => {
    if (!privilege) {
      return;
    }
    await api.delete(`${PRIVILEGES}${path`/${privilege.name}`}`);
    toast.success(t('security.privileges.deleted', { name: privilege.name }));
    leave(LIST);
    await Promise.all([invalidate(PRIVILEGES), invalidate(ROLES)]);
  };

  const granted = permissionOf({ type: type.id, readOnly: false, ...form.values });

  const renderField = (field: PrivilegeField) => {
    const label = fieldLabel(t, field.id);
    const common = { label, required: field.required, disabled: readOnly };
    switch (field.kind) {
      case 'actions':
        return (
          <FieldShell
            key={field.id}
            id="privilege-actions"
            label={label}
            required={field.required}
            error={form.errors.actions}
          >
            <CheckboxGroup.Root
              id="privilege-actions"
              name="actions"
              value={form.values.actions}
              disabled={readOnly}
              aria-label={label}
              className={formStyles.checks}
              onValueChange={(next) => {
                const added = next.find((value) => !form.values.actions.includes(value as Action));
                const removed = form.values.actions.find((value) => !next.includes(value));
                if (added) {
                  form.set('actions', toggleAction(form.values.actions, added as Action, true));
                } else if (removed) {
                  form.set('actions', toggleAction(form.values.actions, removed, false));
                }
              }}
            >
              {(field.values ?? []).map((value) => (
                <CheckboxGroup.Item key={value} value={value}>
                  {actionLabel(t, value)}
                </CheckboxGroup.Item>
              ))}
            </CheckboxGroup.Root>
          </FieldShell>
        );
      case 'format':
        return (
          <SelectField
            key={field.id}
            {...common}
            {...form.field('format')}
            onChange={(value) => {
              form.set('format', value);
              const repository = form.values.repository;
              const match = repositories.data?.find((item) => item.name === repository);
              if (value !== ANY && match && match.format !== value) {
                form.set('repository', ANY);
              }
            }}
            options={formatOptions(t, field.values)}
          />
        );
      case 'repository':
        return repositories.isError ? (
          <TextField key={field.id} {...common} {...form.field('repository')} hint={fieldHint(t, field.id)} />
        ) : (
          <SelectField
            key={field.id}
            {...common}
            {...form.field('repository')}
            hint={fieldHint(t, field.id)}
            options={repositoryOptions(
              t,
              repositories.data ?? [],
              form.values.format,
              form.values.repository,
            )}
          />
        );
      case 'contentSelector':
        return selectors.data ? (
          <SelectField
            key={field.id}
            {...common}
            {...form.field('contentSelector')}
            hint={fieldHint(t, field.id)}
            options={selectors.data.map((selector) => ({ value: selector.name, label: selector.name }))}
          />
        ) : (
          <TextField
            key={field.id}
            {...common}
            {...form.field('contentSelector')}
            hint={fieldHint(t, field.id)}
          />
        );
      default: {
        if (field.id === 'name') {
          return (
            <TextField
              key={field.id}
              {...common}
              {...form.field('name')}
              disabled={readOnly || !creating}
              hint={creating ? fieldHint(t, 'name') : undefined}
              autoFocus={creating}
              maxLength={200}
            />
          );
        }
        const id = field.id as 'description' | 'domain' | 'pattern';
        return <TextField key={field.id} {...common} {...form.field(id)} hint={fieldHint(t, field.id)} />;
      }
    }
  };

  return (
    <form className={formStyles.form} onSubmit={form.submit} noValidate>
      {builtIn && <BuiltInNote text={t('security.privilege.builtIn')} />}
      <Card size="3">
        <div className={formStyles.card}>
          <Text size="2" color="gray">
            {t('security.privilege.typeIs', { type: typeLabel(t, type.id) })}
          </Text>
          {type.fields.map(renderField)}
          <div className={styles.grants}>
            <Text size="1" weight="medium" color="gray">
              {t('security.privilege.grants')}
            </Text>
            <Code variant="ghost" className={formStyles.mono}>
              {granted ?? t('security.privilege.grantsNothing')}
            </Code>
          </div>
        </div>
      </Card>
      <FormError message={form.formError} />
      <div className={formStyles.actions}>
        {!readOnly && (
          <Button type="submit" loading={form.submitting} disabled={!creating && !form.dirty}>
            {creating ? t('security.privilege.createSubmit') : t('security.form.save')}
          </Button>
        )}
        <Button asChild variant="soft" color="gray">
          <Link to={LIST}>{readOnly ? t('security.form.back') : t('security.form.cancel')}</Link>
        </Button>
        {!creating && canDelete && !builtIn && (
          <Button type="button" variant="soft" color="red" onClick={() => setDeleting(true)}>
            <Trash2 size={16} aria-hidden="true" />
            {t('security.form.delete')}
          </Button>
        )}
      </div>
      <UnsavedChangesGuard when={form.dirty && !leaving && !readOnly} />
      <ConfirmDialog
        open={deleting}
        onOpenChange={setDeleting}
        onConfirm={remove}
        title={t('security.privileges.deleteTitle', { name: privilege?.name ?? '' })}
        description={t('security.privileges.deleteText')}
        confirmLabel={t('security.form.delete')}
        danger
      />
    </form>
  );
}

export default function PrivilegePage() {
  const t = useT();
  const { type: typeId, name } = useParams();
  const { leaving, leave } = useLeave();
  const creating = name === undefined;
  const types = useApi<PrivilegeType[]>(PRIVILEGE_TYPES);
  const existing = useApi<Privilege>(creating ? null : `${PRIVILEGES}${path`/${name}`}`);
  const wanted = creating ? typeId : existing.data?.type;
  const type = types.data?.find((candidate) => candidate.id === wanted);
  const title = creating
    ? t('security.privilege.createTitleOf', { type: typeLabel(t, typeId ?? '') })
    : (name ?? '');
  const breadcrumbs = [
    { label: t('security.crumbs.settings'), to: '/settings' },
    { label: t('security.privileges.title'), to: LIST },
    ...(creating ? [{ label: t('security.privilege.create'), to: `${LIST}/create` }] : []),
    { label: creating ? typeLabel(t, typeId ?? '') : (name ?? '') },
  ];
  const loading = types.isPending || (!creating && existing.isPending);
  const error = types.error ?? existing.error;
  return (
    <Page title={title} breadcrumbs={breadcrumbs}>
      {loading ? (
        <Flex justify="center" py="8">
          <Spinner size="3" />
        </Flex>
      ) : error ? (
        <ErrorState error={error} />
      ) : !type ? (
        <ErrorState error={new Error(t('security.privilege.unknownType', { type: wanted ?? '' }))} />
      ) : (
        <PrivilegeEditor
          key={name ?? typeId}
          type={type}
          privilege={existing.data}
          leaving={leaving}
          leave={leave}
        />
      )}
    </Page>
  );
}
