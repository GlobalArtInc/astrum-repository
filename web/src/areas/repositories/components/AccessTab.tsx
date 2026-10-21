import { Badge, Button, IconButton, Select, Switch } from '@radix-ui/themes';
import { CircleCheck, CircleX, Ghost, KeyRound, Lock, User, Users, X } from 'lucide-react';
import { useState } from 'react';
import { Link } from 'react-router';
import { api, path, useApi, useInvalidateApi } from '../../../api';
import { useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import { describeError, useConfirm, useToast } from '../../../ui';
import { ANONYMOUS, ASSIGNABLE_ROLES, PRIVILEGES, ROLES, USERS } from '../../security/api';
import { roleHref, userHref } from '../../security/shared/links';
import type { UiRepository } from '../api';
import {
  ANONYMOUS_ROLE,
  accessPaths,
  canSetLevel,
  closeForAnonymous,
  LEVELS,
  type Level,
  levelOf,
  openForAnonymous,
  ownPrivileges,
  type PrivilegeRow,
  type RoleLevel,
  type RoleRow,
  repositoryPrivileges,
  roleBody,
  rolesWith,
  type UserRow,
  userBody,
  withLevel,
} from '../model/access';
import { Card } from './EditTabs';
import styles from './tabs.module.css';

const LEVEL_KEYS: Record<RoleLevel, string> = {
  read: 'repositories.access.level.read',
  write: 'repositories.access.level.write',
  full: 'repositories.access.level.full',
  custom: 'repositories.access.level.custom',
  none: 'repositories.access.revoke',
};

export function AccessTab({
  repository,
  formatWide = true,
}: {
  repository: UiRepository;
  formatWide?: boolean;
}) {
  const t = useT();
  const toast = useToast();
  const confirm = useConfirm();
  const invalidate = useInvalidateApi();
  const canPrivileges = useCan('astrum:privileges:read');
  const canRoles = useCan('astrum:roles:read');
  const canUsers = useCan('astrum:users:read');
  const canAnonymous = useCan('astrum:settings:read');
  const canEditRoles = useCan('astrum:roles:update');
  const canCreateRoles = useCan('astrum:roles:create');
  const canEditUsers = useCan('astrum:users:update');
  const privileges = useApi<PrivilegeRow[]>(canPrivileges ? PRIVILEGES : null);
  const roles = useApi<RoleRow[]>(canRoles ? ROLES : null);
  const users = useApi<UserRow[]>(canUsers ? USERS : null);
  const assignable = useApi<RoleRow[]>(canRoles && canEditUsers ? ASSIGNABLE_ROLES : null);
  const anonymous = useApi<{ enabled: boolean; userId: string }>(canAnonymous ? ANONYMOUS : null);
  const [busy, setBusy] = useState(false);
  const [grantRole, setGrantRole] = useState('');
  const [grantLevel, setGrantLevel] = useState<Level>('read');
  const [addUser, setAddUser] = useState('');
  const [addRole, setAddRole] = useState('');

  const roleRows = roles.data ?? [];
  const userRows = users.data ?? [];
  const found = repositoryPrivileges(privileges.data ?? [], repository, formatWide);
  const names = new Set(found.map((privilege) => privilege.name));
  const own = ownPrivileges(privileges.data ?? [], repository);
  const roleIds = rolesWith(roleRows, names);
  const roleSet = new Set(roleIds);
  const byRole = new Map(roleRows.map((role) => [role.id, role]));
  const granted = new Set((assignable.data ?? []).map((role) => role.id));
  const withAccess = userRows.filter((user) => user.roles.some((role) => roleSet.has(role)));
  const anonymousUser = userRows.find((user) => user.userId === anonymous.data?.userId);
  const serverAnonymous = anonymous.data?.enabled === true;
  const anonymousOn = serverAnonymous && anonymousUser?.roles.some((role) => roleSet.has(role)) === true;
  const levels = LEVELS.filter((level) => canSetLevel(own, level));
  const grantable = roleRows.filter((role) => !role.readOnly && !roleSet.has(role.id));
  const addable = userRows.filter((user) => !withAccess.includes(user));
  const throughRoles = roleIds.filter((id) => granted.has(id));
  const opening = anonymousUser ? openForAnonymous(anonymousUser, userRows, roleRows) : null;
  const canOpen = opening !== null || byRole.has(ANONYMOUS_ROLE) || canCreateRoles;
  const switchShown = canEditRoles && canEditUsers && serverAnonymous && anonymousUser !== undefined;

  const run = async (work: () => Promise<void>) => {
    setBusy(true);
    try {
      await work();
      toast.success(t('repositories.access.saved'));
    } catch (error) {
      toast.error(t('repositories.access.saveFailed'), describeError(t, error));
    } finally {
      await Promise.all([invalidate(ROLES), invalidate(USERS)]);
      setBusy(false);
    }
  };

  const putRole = (role: RoleRow, next: readonly string[]) =>
    api.put<void>(`${ROLES}${path`/${role.id}`}`, roleBody(role, next));

  const putUser = (user: UserRow, next: readonly string[]) =>
    api.put<void>(`${USERS}${path`/${user.userId}`}`, userBody(user, next));

  const setLevel = (role: RoleRow, level: Level | 'none') =>
    run(() => putRole(role, withLevel(role, own, level)));

  const grant = () => {
    const role = byRole.get(grantRole);
    if (!role) {
      return;
    }
    void run(async () => {
      await putRole(role, withLevel(role, own, grantLevel));
      setGrantRole('');
    });
  };

  const removeRole = async (user: UserRow, role: string) => {
    const agreed = await confirm({
      title: t('repositories.access.removeRoleTitle'),
      description: t('repositories.access.removeRoleText', { user: user.userId, role }),
      confirmLabel: t('repositories.access.removeRoleConfirm'),
      danger: true,
    });
    if (agreed) {
      await run(() =>
        putUser(
          user,
          user.roles.filter((held) => held !== role),
        ),
      );
    }
  };

  const add = () => {
    const user = userRows.find((row) => row.userId === addUser);
    if (!user || addRole === '') {
      return;
    }
    void run(async () => {
      await putUser(user, [...user.roles, addRole]);
      setAddUser('');
      setAddRole('');
    });
  };

  const closeAnonymous = async () => {
    if (!anonymousUser) {
      return;
    }
    const plan = closeForAnonymous(anonymousUser, userRows, roleRows, own, names);
    if (plan.blocked.length > 0 || plan.roles.length === 0) {
      toast.error(
        t('repositories.access.anonymousBlockedTitle'),
        t('repositories.access.anonymousBlocked', { roles: plan.blocked.join(', ') }),
      );
      return;
    }
    const list = plan.roles.map((role) => role.id).join(', ');
    const agreed = await confirm({
      title: t('repositories.access.anonymousOffTitle'),
      description:
        plan.sharedWith.length > 0
          ? t('repositories.access.anonymousOffShared', { roles: list, users: plan.sharedWith.join(', ') })
          : t('repositories.access.anonymousOffText', { roles: list }),
      confirmLabel: t('repositories.access.anonymousOff'),
      danger: true,
    });
    if (agreed) {
      await run(async () => {
        for (const role of plan.roles) {
          await putRole(role, withLevel(role, own, 'none'));
        }
      });
    }
  };

  const openAnonymous = async () => {
    if (!anonymousUser) {
      return;
    }
    const target = opening ?? byRole.get(ANONYMOUS_ROLE);
    const id = target?.id ?? ANONYMOUS_ROLE;
    const agreed = await confirm({
      title: t('repositories.access.anonymousOnTitle'),
      description: t('repositories.access.anonymousOnText', { role: id }),
      confirmLabel: t('repositories.access.anonymousOn'),
    });
    if (!agreed) {
      return;
    }
    await run(async () => {
      if (!target) {
        await api.post(ROLES, {
          ...roleBody({ id, privileges: [] }, withLevel({ id, privileges: [] }, own, 'read')),
          description: 'Repositories anonymous users may read',
        });
      } else if (levelOf(target, own) === 'none') {
        await putRole(target, withLevel(target, own, 'read'));
      }
      if (!anonymousUser.roles.includes(id)) {
        await putUser(anonymousUser, [...anonymousUser.roles, id]);
      }
    });
  };

  const levelLabel = (level: RoleLevel) => t(LEVEL_KEYS[level]);

  const levelSelect = (role: RoleRow, level: RoleLevel) => (
    <Select.Root
      size="1"
      value={level}
      disabled={busy}
      onValueChange={(next) => void setLevel(role, next as Level | 'none')}
    >
      <Select.Trigger aria-label={t('repositories.access.levelOf', { role: role.id })} />
      <Select.Content>
        {level === 'custom' && (
          <Select.Item value="custom" disabled>
            {levelLabel('custom')}
          </Select.Item>
        )}
        {levels.map((option) => (
          <Select.Item key={option} value={option}>
            {levelLabel(option)}
          </Select.Item>
        ))}
        <Select.Separator />
        <Select.Item value="none">{t('repositories.access.revoke')}</Select.Item>
      </Select.Content>
    </Select.Root>
  );

  return (
    <div className={styles.stack}>
      <Card
        icon={<Users size={18} />}
        title={t('repositories.access.roles')}
        count={t('repositories.access.roleCount', { count: roleIds.length })}
      >
        {roleIds.length === 0 ? (
          <p className={styles.text}>{t('repositories.access.noRoles')}</p>
        ) : (
          <ul className={styles.rows}>
            {roleIds.map((id) => {
              const role = byRole.get(id);
              if (!role) {
                return null;
              }
              const level = levelOf(role, own);
              const paths = accessPaths(role, roleRows, own, names);
              const editable = canEditRoles && !role.readOnly && levels.length > 0 && paths.direct;
              return (
                <li key={id} className={styles.row}>
                  <span className={styles.inline}>
                    <Link to={roleHref(id)} className={`${styles.link} ${styles.mono}`}>
                      {id}
                    </Link>
                    {role.readOnly && (
                      <span className={`${styles.inline} ${styles.muted}`}>
                        <Lock size={12} aria-hidden="true" />
                        {t('repositories.access.builtIn')}
                      </span>
                    )}
                    {paths.direct && (paths.wide || paths.nested) && (
                      <span className={styles.muted}>{t('repositories.access.alsoOther')}</span>
                    )}
                  </span>
                  {editable ? (
                    levelSelect(role, level)
                  ) : (
                    <Badge variant="soft" color="gray" radius="small" size="1">
                      {paths.direct
                        ? levelLabel(level)
                        : paths.wide
                          ? t('repositories.access.wide')
                          : t('repositories.access.nested')}
                    </Badge>
                  )}
                </li>
              );
            })}
          </ul>
        )}
        {canEditRoles && levels.length > 0 && grantable.length > 0 && (
          <div className={styles.addRow}>
            <Select.Root size="1" value={grantRole} onValueChange={setGrantRole} disabled={busy}>
              <Select.Trigger
                placeholder={t('repositories.access.pickRole')}
                aria-label={t('repositories.access.pickRole')}
              />
              <Select.Content>
                {grantable.map((role) => (
                  <Select.Item key={role.id} value={role.id}>
                    {role.id}
                  </Select.Item>
                ))}
              </Select.Content>
            </Select.Root>
            <Select.Root
              size="1"
              value={grantLevel}
              onValueChange={(next) => setGrantLevel(next as Level)}
              disabled={busy}
            >
              <Select.Trigger aria-label={t('repositories.access.pickLevel')} />
              <Select.Content>
                {levels.map((option) => (
                  <Select.Item key={option} value={option}>
                    {levelLabel(option)}
                  </Select.Item>
                ))}
              </Select.Content>
            </Select.Root>
            <Button size="1" variant="soft" disabled={busy || grantRole === ''} onClick={grant}>
              {t('repositories.access.grant')}
            </Button>
          </div>
        )}
      </Card>
      <Card
        icon={<User size={18} />}
        title={t('repositories.access.users')}
        count={t('repositories.access.userCount', { count: withAccess.length })}
      >
        {withAccess.length === 0 ? (
          <p className={styles.text}>{t('repositories.access.noUsers')}</p>
        ) : (
          <ul className={styles.rows}>
            {withAccess.map((user) => (
              <li key={`${user.source ?? ''}/${user.userId}`} className={styles.row}>
                <Link to={userHref(user.userId, user.source)} className={`${styles.link} ${styles.mono}`}>
                  {user.userId}
                </Link>
                <span className={styles.chips}>
                  {user.roles
                    .filter((role) => roleSet.has(role))
                    .map((role) => (
                      <Badge key={role} variant="soft" radius="small" size="1">
                        {role}
                        {canEditUsers && (
                          <IconButton
                            size="1"
                            variant="ghost"
                            color="gray"
                            className={styles.chipRemove}
                            disabled={busy}
                            aria-label={t('repositories.access.removeRole', { role, user: user.userId })}
                            onClick={() => void removeRole(user, role)}
                          >
                            <X size={12} aria-hidden="true" />
                          </IconButton>
                        )}
                      </Badge>
                    ))}
                </span>
              </li>
            ))}
          </ul>
        )}
        {canEditUsers && throughRoles.length > 0 && addable.length > 0 && (
          <div className={styles.addRow}>
            <Select.Root size="1" value={addUser} onValueChange={setAddUser} disabled={busy}>
              <Select.Trigger
                placeholder={t('repositories.access.pickUser')}
                aria-label={t('repositories.access.pickUser')}
              />
              <Select.Content>
                {addable.map((user) => (
                  <Select.Item key={`${user.source ?? ''}/${user.userId}`} value={user.userId}>
                    {user.userId}
                  </Select.Item>
                ))}
              </Select.Content>
            </Select.Root>
            <Select.Root size="1" value={addRole} onValueChange={setAddRole} disabled={busy}>
              <Select.Trigger
                placeholder={t('repositories.access.throughRole')}
                aria-label={t('repositories.access.throughRole')}
              />
              <Select.Content>
                {throughRoles.map((id) => (
                  <Select.Item key={id} value={id}>
                    {id}
                  </Select.Item>
                ))}
              </Select.Content>
            </Select.Root>
            <Button size="1" variant="soft" disabled={busy || addUser === '' || addRole === ''} onClick={add}>
              {t('repositories.access.addUser')}
            </Button>
          </div>
        )}
      </Card>
      <Card
        icon={<Ghost size={18} />}
        title={t('repositories.access.anonymous')}
        count={
          <span className={styles.inline}>
            {anonymousOn ? (
              <CircleCheck size={12} aria-hidden="true" />
            ) : (
              <CircleX size={12} aria-hidden="true" />
            )}
            {anonymousOn ? t('repositories.access.enabled') : t('repositories.access.disabled')}
          </span>
        }
        action={
          switchShown && (
            <Switch
              checked={anonymousOn}
              disabled={busy || (!anonymousOn && !canOpen)}
              aria-label={t('repositories.access.anonymousSwitch')}
              onCheckedChange={(checked) => void (checked ? openAnonymous() : closeAnonymous())}
            />
          )
        }
      >
        <p className={styles.text}>
          {anonymous.data && !anonymous.data.enabled
            ? t('repositories.access.anonymousServerOff')
            : anonymousOn
              ? t('repositories.access.anonymousYes')
              : t('repositories.access.anonymousNo')}
        </p>
        {anonymous.data && !anonymous.data.enabled && (
          <Link to="/admin/security/anonymous" className={styles.link}>
            {t('repositories.access.anonymousSettings')}
          </Link>
        )}
      </Card>
      <Card
        icon={<KeyRound size={18} />}
        title={t('repositories.access.privileges')}
        count={t('repositories.access.privilegeCount', { count: found.length })}
      >
        {found.length === 0 ? (
          <p className={styles.text}>{t('repositories.access.noPrivileges')}</p>
        ) : (
          <ul className={styles.rows}>
            {found.map((privilege) => (
              <li key={privilege.name} className={styles.row}>
                <span>
                  <span className={styles.mono}>{privilege.name}</span>{' '}
                  {privilege.description && <span className={styles.muted}>({privilege.description})</span>}
                </span>
                <Badge variant="soft" radius="small" size="1">
                  {(privilege.actions ?? []).join(', ')}
                </Badge>
              </li>
            ))}
          </ul>
        )}
      </Card>
    </div>
  );
}
