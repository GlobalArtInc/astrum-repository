import { Button, Card, Flex, Heading, Spinner } from '@radix-ui/themes';
import { Trash2 } from 'lucide-react';
import { useMemo, useState } from 'react';
import { Link, useParams } from 'react-router';
import { api, path, useApi, useInvalidateApi } from '../../../api';
import { useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import {
  ConfirmDialog,
  ErrorState,
  FormError,
  Page,
  SelectField,
  TextField,
  UnsavedChangesGuard,
  useForm,
  useToast,
} from '../../../ui';
import {
  ASSIGNABLE_ROLES,
  DEFAULT_SOURCE,
  PRIVILEGES,
  type Privilege,
  ROLE_SOURCES,
  ROLES,
  type Role,
  type RoleSource,
  USERS,
} from '../api';
import { BuiltInNote } from '../shared/BuiltInNote';
import { ChipPicker } from '../shared/ChipPicker';
import { EffectivePrivileges } from '../shared/EffectivePrivileges';
import styles from '../shared/Form.module.css';
import { ROLES_PAGE, roleHref } from '../shared/links';
import { explainRefusal } from '../shared/messages';
import { useLeave } from '../shared/useLeave';
import { MappedRoleField } from './MappedRoleField';
import {
  EMPTY_ROLE,
  fromRole,
  privilegeOptions,
  type RoleType,
  type RoleValues,
  roleOptions,
  toRoleRequest,
  validateRole,
} from './model';

function RoleForm({ role, leaving, leave }: { role?: Role; leaving: boolean; leave(to: string): void }) {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const creating = role === undefined;
  const canWrite = useCan(creating ? 'astrum:roles:create' : 'astrum:roles:update');
  const canDelete = useCan('astrum:roles:delete');
  const builtIn = role?.readOnly === true;
  const readOnly = builtIn || !canWrite;
  const [deleting, setDeleting] = useState(false);
  const privileges = useApi<Privilege[]>(PRIVILEGES);
  const roles = useApi<Role[]>(ROLES);
  const assignable = useApi<Role[]>(readOnly ? null : ASSIGNABLE_ROLES);
  const sources = useApi<RoleSource[]>(ROLE_SOURCES);

  const form = useForm<RoleValues>({
    initial: role ? fromRole(role) : EMPTY_ROLE,
    validate: (values) => validateRole(t, values),
    onSubmit: (values) =>
      explainRefusal(t, async () => {
        const body = toRoleRequest(values);
        if (creating) {
          await api.post(ROLES, body);
          toast.success(t('security.role.created', { id: body.id }));
        } else {
          await api.put(`${ROLES}${path`/${role.id}`}`, body);
          toast.success(t('security.role.saved', { id: role.id }));
        }
        await Promise.all([invalidate(ROLES), invalidate(USERS)]);
        if (creating) {
          leave(roleHref(body.id));
        }
      }),
  });

  const privilegeChoices = useMemo(() => privilegeOptions(t, privileges.data ?? []), [t, privileges.data]);
  const roleChoices = useMemo(
    () =>
      roleOptions(assignable.data ?? roles.data ?? [], roles.data ?? [], form.values.roles, form.values.id),
    [assignable.data, roles.data, form.values.roles, form.values.id],
  );

  const sourceName = (id: string) => sources.data?.find((source) => source.id === id)?.name ?? id;

  const remove = async () => {
    if (!role) {
      return;
    }
    await api.delete(`${ROLES}${path`/${role.id}`}`);
    toast.success(t('security.roles.deleted', { id: role.id }));
    leave(ROLES_PAGE);
    await Promise.all([invalidate(ROLES), invalidate(USERS)]);
  };

  return (
    <form className={styles.form} onSubmit={form.submit} noValidate>
      {builtIn && <BuiltInNote text={t('security.role.builtIn')} />}
      <Card size="3">
        <div className={styles.card}>
          {(creating || form.values.type === 'external') && (
            <SelectField
              label={t('security.role.type')}
              name="type"
              value={form.values.type}
              onChange={(value) => {
                const type = value as RoleType;
                form.set('type', type);
                if (type === 'external' && form.values.source === DEFAULT_SOURCE) {
                  form.set('source', sources.data?.[0]?.id ?? '');
                }
              }}
              options={[
                { value: 'astrum', label: t('security.role.typeAstrum') },
                { value: 'external', label: t('security.role.typeExternal') },
              ]}
              required
              disabled={readOnly || !creating}
              hint={creating ? t('security.role.typeHint') : undefined}
            />
          )}
          {form.values.type === 'external' && (
            <>
              <SelectField
                label={t('security.role.source')}
                {...form.field('source')}
                options={(sources.data ?? []).map((source) => ({ value: source.id, label: source.name }))}
                required
                disabled={readOnly || !creating}
                hint={creating ? t('security.role.sourceHint') : undefined}
              />
              {creating && form.values.source !== '' && (
                <MappedRoleField
                  key={form.values.source}
                  source={form.values.source}
                  sourceName={sourceName(form.values.source)}
                  value={form.values.id}
                  disabled={readOnly}
                  onPick={(id) => {
                    if (form.values.name === '' || form.values.name === form.values.id) {
                      form.set('name', id);
                    }
                    form.set('id', id);
                  }}
                />
              )}
            </>
          )}
          <div className={styles.pair}>
            <TextField
              label={t('security.role.id')}
              {...form.field('id')}
              required
              disabled={readOnly || !creating}
              hint={creating ? t('security.role.idHint') : undefined}
              autoFocus={creating}
            />
            <TextField label={t('security.role.name')} {...form.field('name')} required disabled={readOnly} />
          </div>
          <TextField
            label={t('security.role.description')}
            {...form.field('description')}
            disabled={readOnly}
          />
          <ChipPicker
            label={t('security.role.privileges')}
            name="privileges"
            value={form.values.privileges}
            onChange={(value) => form.set('privileges', value)}
            options={privilegeChoices}
            loading={privileges.isPending}
            error={form.errors.privileges}
            disabled={readOnly}
            hint={readOnly ? undefined : t('security.role.privilegesHint')}
          />
          <ChipPicker
            label={t('security.role.roles')}
            name="roles"
            value={form.values.roles}
            onChange={(value) => form.set('roles', value)}
            options={roleChoices}
            loading={roles.isPending}
            error={form.errors.roles}
            disabled={readOnly}
            hint={readOnly ? undefined : t('security.role.rolesHint')}
          />
        </div>
      </Card>
      <FormError message={form.formError} />
      <div className={styles.actions}>
        {!readOnly && (
          <Button type="submit" loading={form.submitting} disabled={!creating && !form.dirty}>
            {creating ? t('security.role.createSubmit') : t('security.form.save')}
          </Button>
        )}
        <Button asChild variant="soft" color="gray">
          <Link to={ROLES_PAGE}>{readOnly ? t('security.form.back') : t('security.form.cancel')}</Link>
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
        title={t('security.roles.deleteTitle', { id: role?.id ?? '' })}
        description={t('security.roles.deleteText')}
        confirmLabel={t('security.form.delete')}
        danger
      />
    </form>
  );
}

export default function RolePage() {
  const t = useT();
  const { id } = useParams();
  const { leaving, leave } = useLeave();
  const creating = id === undefined;
  const role = useApi<Role>(creating ? null : `${ROLES}${path`/${id}`}`);
  const breadcrumbs = [
    { label: t('security.crumbs.settings'), to: '/settings' },
    { label: t('security.roles.title'), to: ROLES_PAGE },
    { label: creating ? t('security.role.create') : (id ?? '') },
  ];
  const title = creating ? t('security.role.createTitle') : (role.data?.name ?? id ?? '');
  return (
    <Page title={title} breadcrumbs={breadcrumbs} description={creating ? undefined : role.data?.description}>
      {!creating && role.isPending ? (
        <Flex justify="center" py="8">
          <Spinner size="3" />
        </Flex>
      ) : role.error ? (
        <ErrorState error={role.error} />
      ) : creating ? (
        <RoleForm key="create" leaving={leaving} leave={leave} />
      ) : (
        <>
          <RoleForm key={id} role={role.data} leaving={leaving} leave={leave} />
          <section aria-labelledby="role-effective" className={styles.section}>
            <Heading as="h2" size="4" id="role-effective">
              {t('security.role.effectiveTab')}
            </Heading>
            <EffectivePrivileges roleIds={[id ?? '']} label={t('security.role.effectiveTab')} />
          </section>
        </>
      )}
    </Page>
  );
}
