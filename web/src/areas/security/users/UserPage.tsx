import { Button, Flex, Spinner, Switch } from '@radix-ui/themes';
import { Key, Trash2 } from 'lucide-react';
import { type FormEvent, useMemo, useState } from 'react';
import { useNavigate, useParams } from 'react-router';
import { api, path, useApi, useInvalidateApi, withQuery } from '../../../api';
import { useCan, useRefreshSession, useSession } from '../../../auth/state';
import { useT } from '../../../i18n';
import { ConfirmDialog, ErrorState, FormError, Page, UnsavedChangesGuard, useToast } from '../../../ui';
import { Input, Password } from '../../identity/shared/Fields';
import {
  ASSIGNABLE_ROLES,
  type ContentSelector,
  DEFAULT_SOURCE,
  PRIVILEGES,
  type Privilege,
  ROLES,
  type Role,
  SELECTORS,
  USER_SOURCES,
  USERS,
  type User,
  type UserSource,
} from '../api';
import { EditBar, EditForm, FormSection } from '../shared/Edit';
import { USERS_PAGE } from '../shared/links';
import { refusedBy } from '../shared/messages';
import { SecurityTree, userTree } from '../shared/SecurityTree';
import { TransferList } from '../shared/TransferList';
import { useLeave } from '../shared/useLeave';
import { ChangePasswordDialog } from './ChangePasswordDialog';
import {
  deleteQuery,
  EMPTY_USER,
  fromUser,
  fullName,
  sourceLabel,
  toCreateRequest,
  toUpdateRequest,
  type UserValues,
  validateUser,
} from './model';
import styles from './Users.module.css';

function Tree({ user, roleIds, source }: { user: string; roleIds: readonly string[]; source: string }) {
  const t = useT();
  const canSelectors = useCan('astrum:selectors:read');
  const roles = useApi<Role[]>(ROLES);
  const privileges = useApi<Privilege[]>(PRIVILEGES);
  const selectors = useApi<ContentSelector[]>(canSelectors ? SELECTORS : null);
  const nodes = useMemo(
    () =>
      userTree(
        t('security.user.treeRoot', { id: user, source }),
        roleIds,
        new Map((roles.data ?? []).map((role) => [role.id, role])),
        new Map((privileges.data ?? []).map((privilege) => [privilege.name, privilege])),
        new Map((selectors.data ?? []).map((selector) => [selector.name, selector])),
      ),
    [t, user, source, roleIds, roles.data, privileges.data, selectors.data],
  );
  if (roles.isPending || privileges.isPending) {
    return (
      <Flex justify="center" py="4">
        <Spinner />
      </Flex>
    );
  }
  return (
    <SecurityTree key={roleIds.join('|')} label={t('security.user.treeLabel')} nodes={nodes} framed={false} />
  );
}

function UserForm({ user }: { user?: User }) {
  const t = useT();
  const toast = useToast();
  const navigate = useNavigate();
  const invalidate = useInvalidateApi();
  const refresh = useRefreshSession();
  const { user: me } = useSession();
  const { leaving, leave } = useLeave();
  const creating = user === undefined;
  const local = creating || user.source === DEFAULT_SOURCE;
  const canWrite = useCan(creating ? 'astrum:users:create' : 'astrum:users:update');
  const canDelete = useCan('astrum:users:delete');
  const canChangePassword = useCan('astrum:userschangepw:create');
  const readOnly = !canWrite;
  const isMe = !creating && me !== null && me.userId === user.userId && me.source === user.source;
  const initial = useMemo(() => (user ? fromUser(user) : EMPTY_USER), [user]);
  const [values, setValues] = useState<UserValues>(initial);
  const [shown, setShown] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [failure, setFailure] = useState<string | undefined>();
  const [deleting, setDeleting] = useState(false);
  const [changing, setChanging] = useState(false);
  const roles = useApi<Role[]>(ROLES);
  const assignable = useApi<Role[]>(readOnly ? null : ASSIGNABLE_ROLES);
  const sources = useApi<UserSource[]>(creating ? null : USER_SOURCES);
  const set = (patch: Partial<UserValues>) => setValues((current) => ({ ...current, ...patch }));
  const errors = validateUser(t, values, { creating, local });
  const noRoles = values.roles.length === 0;
  const dirty = JSON.stringify(values) !== JSON.stringify(initial);
  const detailsLocked = readOnly || !local;
  const sourceName = (source: string) =>
    source === DEFAULT_SOURCE ? t('security.users.sourceLocal') : sourceLabel(t, source);

  const roleChoices = useMemo(
    () => (assignable.data ?? roles.data ?? []).map((role) => ({ value: role.id, label: role.id })),
    [assignable.data, roles.data],
  );

  const submit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (Object.keys(errors).length > 0 || (creating && noRoles)) {
      setShown(true);
      return;
    }
    setSubmitting(true);
    setFailure(undefined);
    try {
      if (creating) {
        const body = toCreateRequest(values);
        await api.post(USERS, body);
        toast.success(t('security.user.created', { id: body.userId }));
        await invalidate(USERS);
        leave(USERS_PAGE);
        return;
      }
      await api.put(`${USERS}${path`/${user.userId}`}`, toUpdateRequest(values, user));
      toast.success(t('security.user.saved', { id: user.userId }));
      await invalidate(USERS);
      if (isMe) {
        await refresh();
      }
    } catch (error) {
      const refused = refusedBy(t, error);
      setFailure(refused instanceof Error ? refused.message : String(refused));
    } finally {
      setSubmitting(false);
    }
  };

  const remove = async () => {
    if (!user) {
      return;
    }
    await api.delete(deleteQuery(user, sources.data ?? []));
    toast.success(t('security.users.deleted', { id: user.userId }));
    leave(USERS_PAGE);
    await invalidate(USERS);
  };

  const shownErrors = shown ? errors : {};
  const title = creating
    ? t('security.user.createTitle')
    : t('security.user.editTitle', { name: fullName(user) || user.userId });
  const active = values.status === 'active';
  return (
    <Page
      gutter="small"
      title={title}
      description={creating ? t('security.user.createIntro') : t('security.user.editIntro')}
      breadcrumbs={[
        { label: t('security.crumbs.settings'), to: '/settings' },
        { label: t('security.users.title'), to: USERS_PAGE },
        { label: creating ? t('security.user.create') : user.userId },
      ]}
      actions={
        !creating &&
        local && (
          // biome-ignore lint/a11y/noLabelWithoutControl: the Radix switch inside is the control
          <label className={styles.status}>
            <Switch
              size="2"
              checked={active}
              disabled={readOnly || isMe}
              aria-label={t('security.user.toggleStatus', {
                status: active ? t('security.user.active') : t('security.user.inactive'),
              })}
              onCheckedChange={(on) => set({ status: on ? 'active' : 'disabled' })}
            />
            <span>{active ? t('security.user.active') : t('security.user.inactive')}</span>
          </label>
        )
      }
    >
      <EditForm
        id="user-form"
        onSubmit={submit}
        bar={
          <EditBar
            start={
              !creating &&
              canDelete &&
              !isMe && (
                <Button type="button" color="red" variant="soft" onClick={() => setDeleting(true)}>
                  <Trash2 size={16} aria-hidden="true" />
                  {t('security.user.delete')}
                </Button>
              )
            }
            end={
              <>
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => (dirty ? setValues(initial) : navigate(USERS_PAGE))}
                >
                  {t('security.form.discard')}
                </Button>
                {!readOnly && (
                  <Button type="submit" disabled={!creating && !dirty} loading={submitting}>
                    {creating ? t('security.user.createSubmit') : t('security.form.save')}
                  </Button>
                )}
              </>
            }
          />
        }
      >
        <FormSection title={t('security.user.details')}>
          <Input
            label={t('security.user.id')}
            name="userId"
            required
            help={t('security.user.idHelp')}
            placeholder={creating ? t('security.user.idPlaceholder') : undefined}
            value={values.userId}
            disabled={!creating || readOnly}
            autoFocus={creating}
            error={shownErrors.userId}
            onChange={(userId) => set({ userId })}
          />
          <Input
            label={t('security.user.firstName')}
            name="firstName"
            required
            help={t('security.user.firstNameHelp')}
            placeholder={creating ? t('security.user.firstNamePlaceholder') : undefined}
            value={values.firstName}
            disabled={detailsLocked}
            error={shownErrors.firstName}
            onChange={(firstName) => set({ firstName })}
          />
          <Input
            label={t('security.user.lastName')}
            name="lastName"
            required
            help={t('security.user.lastNameHelp')}
            placeholder={creating ? t('security.user.lastNamePlaceholder') : undefined}
            value={values.lastName}
            disabled={detailsLocked}
            error={shownErrors.lastName}
            onChange={(lastName) => set({ lastName })}
          />
          <Input
            label={t('security.user.email')}
            name="emailAddress"
            required
            help={t('security.user.emailHelp')}
            placeholder={creating ? t('security.user.emailPlaceholder') : undefined}
            value={values.emailAddress}
            disabled={detailsLocked}
            error={shownErrors.emailAddress}
            onChange={(emailAddress) => set({ emailAddress })}
          />
          {creating && (
            <>
              <Password
                label={t('security.user.password')}
                name="password"
                required
                help={t('security.user.passwordHelp')}
                value={values.password}
                error={shownErrors.password}
                onChange={(password) => set({ password })}
              />
              <Password
                label={t('security.user.confirm')}
                name="confirm"
                required
                help={t('security.user.confirmHelp')}
                value={values.confirm}
                error={shownErrors.confirm}
                onChange={(confirm) => set({ confirm })}
              />
            </>
          )}
          {!creating && local && canChangePassword && (
            <div className={styles.passwordRow}>
              <Button
                type="button"
                size="3"
                variant="outline"
                color="gray"
                className={styles.passwordButton}
                onClick={() => setChanging(true)}
              >
                <Key size={16} aria-hidden="true" />
                {t('security.password.open')}
              </Button>
            </div>
          )}
        </FormSection>
        <FormSection title={t('security.user.roles')}>
          <TransferList
            label={t('security.user.roles')}
            available={t('security.user.available')}
            chosen={t('security.user.granted')}
            options={roleChoices}
            value={values.roles}
            disabled={readOnly}
            onChange={(next) => set({ roles: next })}
            countLabel={(count) => t('security.transfer.items', { count })}
            emptyLabel={t('security.user.nothingChosen')}
          />
          {noRoles ? (
            <p className={styles.required}>{t('security.user.rolesRequired')}</p>
          ) : (
            <p className={styles.granted}>
              {t('security.user.rolesGranted', { count: values.roles.length })}
            </p>
          )}
        </FormSection>
        <FormSection title={t('security.user.tree')}>
          {noRoles ? (
            <p className={styles.treeEmpty}>{t('security.user.treeEmpty')}</p>
          ) : (
            <Tree
              user={values.userId || '—'}
              roleIds={values.roles}
              source={sourceName(user?.source ?? DEFAULT_SOURCE)}
            />
          )}
        </FormSection>
        <FormError message={failure} />
      </EditForm>
      <UnsavedChangesGuard when={dirty && !leaving && !readOnly && !submitting} />
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
    </Page>
  );
}

export default function UserPage() {
  const t = useT();
  const { userId, source = DEFAULT_SOURCE } = useParams();
  const users = useApi<User[]>(userId === undefined ? null : withQuery(USERS, { source, userId }));
  if (userId === undefined) {
    return <UserForm key="create" />;
  }
  const user = users.data?.find((candidate) => candidate.userId === userId && candidate.source === source);
  if (users.isPending) {
    return (
      <Flex justify="center" py="8">
        <Spinner size="3" />
      </Flex>
    );
  }
  if (users.error || !user) {
    return (
      <Page gutter="small" title={userId}>
        <ErrorState error={users.error ?? new Error(t('security.user.notFound', { id: userId }))} />
      </Page>
    );
  }
  return <UserForm key={`${user.source}/${user.userId}`} user={user} />;
}
