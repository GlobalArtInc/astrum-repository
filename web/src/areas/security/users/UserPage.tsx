import { Badge, Button, Card, Flex, Heading, Spinner, Text } from '@radix-ui/themes';
import { KeySquare, Trash2 } from 'lucide-react';
import { useMemo, useState } from 'react';
import { Link, useParams } from 'react-router';
import { api, path, useApi, useInvalidateApi, withQuery } from '../../../api';
import { useCan, useRefreshSession, useSession } from '../../../auth/state';
import { useT } from '../../../i18n';
import {
  ConfirmDialog,
  ErrorState,
  FormError,
  Page,
  PasswordField,
  SelectField,
  StatusBadge,
  TextField,
  UnsavedChangesGuard,
  useForm,
  useToast,
} from '../../../ui';
import {
  ASSIGNABLE_ROLES,
  DEFAULT_SOURCE,
  ROLES,
  type Role,
  USER_SOURCES,
  USERS,
  type User,
  type UserSource,
} from '../api';
import { roleOptions } from '../roles/model';
import { ChipPicker } from '../shared/ChipPicker';
import { EffectivePrivileges } from '../shared/EffectivePrivileges';
import styles from '../shared/Form.module.css';
import { USERS_PAGE, userHref } from '../shared/links';
import { explainRefusal } from '../shared/messages';
import { useLeave } from '../shared/useLeave';
import { ChangePasswordDialog } from './ChangePasswordDialog';
import {
  deleteQuery,
  EMPTY_USER,
  fromUser,
  fullName,
  STATUS_TONES,
  STATUSES,
  sourceLabel,
  statusLabel,
  toCreateRequest,
  toUpdateRequest,
  type UserValues,
  validateUser,
} from './model';

function UserForm({ user, leaving, leave }: { user?: User; leaving: boolean; leave(to: string): void }) {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const refresh = useRefreshSession();
  const { user: me } = useSession();
  const creating = user === undefined;
  const local = creating || user.source === DEFAULT_SOURCE;
  const canWrite = useCan(creating ? 'astrum:users:create' : 'astrum:users:update');
  const canDelete = useCan('astrum:users:delete');
  const canChangePassword = useCan('astrum:userschangepw:create');
  const readOnly = !canWrite;
  const isMe = !creating && me !== null && me.userId === user.userId && me.source === user.source;
  const [deleting, setDeleting] = useState(false);
  const [changing, setChanging] = useState(false);
  const roles = useApi<Role[]>(ROLES);
  const assignable = useApi<Role[]>(readOnly ? null : ASSIGNABLE_ROLES);
  const sources = useApi<UserSource[]>(creating ? null : USER_SOURCES);

  const form = useForm<UserValues>({
    initial: user ? fromUser(user) : EMPTY_USER,
    validate: (values) => validateUser(t, values, { creating, local }),
    onSubmit: (values) =>
      explainRefusal(t, async () => {
        if (creating) {
          const body = toCreateRequest(values);
          await api.post(USERS, body);
          toast.success(t('security.user.created', { id: body.userId }));
          await invalidate(USERS);
          leave(userHref(body.userId));
          return;
        }
        await api.put(`${USERS}${path`/${user.userId}`}`, toUpdateRequest(values, user));
        toast.success(t('security.user.saved', { id: user.userId }));
        await invalidate(USERS);
        if (isMe) {
          await refresh();
        }
      }),
  });

  const roleChoices = useMemo(
    () => roleOptions(assignable.data ?? roles.data ?? [], roles.data ?? [], form.values.roles),
    [assignable.data, roles.data, form.values.roles],
  );

  const remove = async () => {
    if (!user) {
      return;
    }
    await api.delete(deleteQuery(user, sources.data ?? []));
    toast.success(t('security.users.deleted', { id: user.userId }));
    leave(USERS_PAGE);
    await invalidate(USERS);
  };

  const detailsLocked = readOnly || !local;

  return (
    <>
      <form className={styles.form} onSubmit={form.submit} noValidate>
        <Card size="3">
          <div className={styles.card}>
            {creating && (
              <TextField
                label={t('security.user.userId')}
                {...form.field('userId')}
                required
                autoFocus
                autoComplete="off"
                hint={t('security.user.userIdHint')}
              />
            )}
            {!local && (
              <StatusBadge tone="info">
                {t('security.user.external', { source: sourceLabel(t, user?.source ?? '') })}
              </StatusBadge>
            )}
            <div className={styles.pair}>
              <TextField
                label={t('security.user.firstName')}
                {...form.field('firstName')}
                required={local}
                disabled={detailsLocked}
                autoComplete="off"
              />
              <TextField
                label={t('security.user.lastName')}
                {...form.field('lastName')}
                required={local}
                disabled={detailsLocked}
                autoComplete="off"
              />
            </div>
            <TextField
              label={t('security.user.email')}
              type="email"
              {...form.field('emailAddress')}
              required={local}
              disabled={detailsLocked}
              autoComplete="off"
            />
            {creating && (
              <div className={styles.pair}>
                <PasswordField
                  label={t('security.user.password')}
                  {...form.field('password')}
                  required
                  hint={t('security.password.hint')}
                />
                <PasswordField label={t('security.user.confirm')} {...form.field('confirm')} required />
              </div>
            )}
            <SelectField
              label={t('security.user.status')}
              {...form.field('status')}
              onChange={(value) => form.set('status', value as UserValues['status'])}
              options={STATUSES.map((status) => ({ value: status, label: statusLabel(t, status) }))}
              required
              disabled={detailsLocked}
              hint={t('security.user.statusHint')}
            />
            <ChipPicker
              label={t('security.user.roles')}
              name="roles"
              value={form.values.roles}
              onChange={(value) => form.set('roles', value)}
              options={roleChoices}
              loading={roles.isPending}
              error={form.errors.roles}
              disabled={readOnly}
              hint={readOnly ? undefined : t('security.user.rolesHint')}
            />
            {!local && user && (
              <div className={styles.section}>
                <Text as="p" size="2" weight="medium">
                  {t('security.user.externalRoles')}
                </Text>
                {user.externalRoles.length > 0 ? (
                  <Flex gap="1" wrap="wrap">
                    {user.externalRoles.map((role) => (
                      <Badge key={role} color="gray" variant="soft">
                        {role}
                      </Badge>
                    ))}
                  </Flex>
                ) : (
                  <Text as="p" size="2" color="gray">
                    {t('security.user.externalRolesNone', { source: sourceLabel(t, user.source) })}
                  </Text>
                )}
              </div>
            )}
          </div>
        </Card>
        <FormError message={form.formError} />
        <div className={styles.actions}>
          {!readOnly && (
            <Button type="submit" loading={form.submitting} disabled={!creating && !form.dirty}>
              {creating ? t('security.user.createSubmit') : t('security.form.save')}
            </Button>
          )}
          <Button asChild variant="soft" color="gray">
            <Link to={USERS_PAGE}>{readOnly ? t('security.form.back') : t('security.form.cancel')}</Link>
          </Button>
          {!creating && local && canChangePassword && (
            <Button type="button" variant="soft" onClick={() => setChanging(true)}>
              <KeySquare size={16} aria-hidden="true" />
              {t('security.password.open')}
            </Button>
          )}
          {!creating && canDelete && !isMe && (
            <Button type="button" variant="soft" color="red" onClick={() => setDeleting(true)}>
              <Trash2 size={16} aria-hidden="true" />
              {t('security.form.delete')}
            </Button>
          )}
        </div>
        <UnsavedChangesGuard when={form.dirty && !leaving && !readOnly} />
      </form>
      {user && <ChangePasswordDialog userId={user.userId} open={changing} onOpenChange={setChanging} />}
      <ConfirmDialog
        open={deleting}
        onOpenChange={setDeleting}
        onConfirm={remove}
        title={t('security.users.deleteTitle', { id: user?.userId ?? '' })}
        description={t('security.users.deleteText')}
        confirmLabel={t('security.form.delete')}
        danger
      />
    </>
  );
}

export default function UserPage() {
  const t = useT();
  const { userId, source = DEFAULT_SOURCE } = useParams();
  const { leaving, leave } = useLeave();
  const creating = userId === undefined;
  const users = useApi<User[]>(creating ? null : withQuery(USERS, { source, userId }));
  const user = users.data?.find((candidate) => candidate.userId === userId && candidate.source === source);
  const breadcrumbs = [
    { label: t('security.crumbs.settings'), to: '/settings' },
    { label: t('security.users.title'), to: USERS_PAGE },
    { label: creating ? t('security.user.create') : (userId ?? '') },
  ];
  const name = user ? fullName(user) : '';
  return (
    <Page
      title={creating ? t('security.user.createTitle') : (userId ?? '')}
      breadcrumbs={breadcrumbs}
      description={
        user ? (
          <Flex as="span" gap="2" align="center" wrap="wrap">
            {name && <span>{name}</span>}
            <StatusBadge tone={STATUS_TONES[user.status] ?? 'neutral'}>
              {statusLabel(t, user.status)}
            </StatusBadge>
            <StatusBadge tone="neutral">{sourceLabel(t, user.source)}</StatusBadge>
          </Flex>
        ) : undefined
      }
    >
      {creating ? (
        <UserForm key="create" leaving={leaving} leave={leave} />
      ) : users.isPending ? (
        <Flex justify="center" py="8">
          <Spinner size="3" />
        </Flex>
      ) : users.error ? (
        <ErrorState error={users.error} />
      ) : !user ? (
        <ErrorState error={new Error(t('security.user.notFound', { id: userId ?? '' }))} />
      ) : (
        <>
          <UserForm key={`${user.source}/${user.userId}`} user={user} leaving={leaving} leave={leave} />
          <section aria-labelledby="user-effective" className={styles.section}>
            <Heading as="h2" size="4" id="user-effective">
              {t('security.user.effective')}
            </Heading>
            <EffectivePrivileges roleIds={user.roles} label={t('security.user.effective')} />
          </section>
        </>
      )}
    </Page>
  );
}
