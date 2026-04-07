import { Button, Flex, Spinner } from '@radix-ui/themes';
import { Trash2 } from 'lucide-react';
import { type FormEvent, useMemo, useState } from 'react';
import { useNavigate, useParams } from 'react-router';
import { api, path, useApi, useInvalidateApi } from '../../../api';
import { useCan } from '../../../auth/state';
import { type Translate, useT } from '../../../i18n';
import { ConfirmDialog, ErrorState, FormError, Page, UnsavedChangesGuard, useToast } from '../../../ui';
import { Area, Choice, Input } from '../../identity/shared/Fields';
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
import { EditBar, EditForm, FormSection, Progress, StepTitle } from '../shared/Edit';
import editStyles from '../shared/Edit.module.css';
import { ROLES_PAGE, roleProfileHref } from '../shared/links';
import { refusedBy } from '../shared/messages';
import { TransferList } from '../shared/TransferList';
import { useLeave } from '../shared/useLeave';
import { MappedRoleField } from './MappedRoleField';
import { EMPTY_ROLE, fromRole, type RoleType, type RoleValues, toRoleRequest, validateId } from './model';
import { PrivilegePicker } from './PrivilegePicker';
import styles from './Roles.module.css';

type Step = 'type' | 'setup' | 'privileges' | 'roles';

export function roleSteps(creating: boolean): Step[] {
  return creating ? ['type', 'setup', 'privileges', 'roles'] : ['setup', 'privileges', 'roles'];
}

export function setupErrors(t: Translate, values: RoleValues): Partial<Record<keyof RoleValues, string>> {
  const errors: Partial<Record<keyof RoleValues, string>> = {};
  const id = validateId(t, values.id);
  if (id) {
    errors.id = id;
  }
  if (values.name.trim() === '') {
    errors.name = t('security.form.required');
  }
  if (values.type === 'external' && (values.source === '' || values.source === DEFAULT_SOURCE)) {
    errors.source = t('security.form.required');
  }
  return errors;
}

const STEP_NAMES: Record<Step, string> = {
  type: 'security.role.stepType',
  setup: 'security.role.stepSetup',
  privileges: 'security.role.stepPrivileges',
  roles: 'security.role.stepRoles',
};

function Wizard({ role }: { role?: Role }) {
  const t = useT();
  const toast = useToast();
  const navigate = useNavigate();
  const invalidate = useInvalidateApi();
  const { leaving, leave } = useLeave();
  const creating = role === undefined;
  const canWrite = useCan(creating ? 'astrum:roles:create' : 'astrum:roles:update');
  const canDelete = useCan('astrum:roles:delete');
  const readOnly = role?.readOnly === true || !canWrite;
  const steps = roleSteps(creating);
  const [index, setIndex] = useState(0);
  const initial = useMemo(() => (role ? fromRole(role) : EMPTY_ROLE), [role]);
  const [values, setValues] = useState<RoleValues>(initial);
  const [shown, setShown] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [failure, setFailure] = useState<string | undefined>();
  const [deleting, setDeleting] = useState(false);
  const privileges = useApi<Privilege[]>(PRIVILEGES);
  const roles = useApi<Role[]>(ROLES);
  const assignable = useApi<Role[]>(readOnly ? null : ASSIGNABLE_ROLES);
  const sources = useApi<RoleSource[]>(ROLE_SOURCES);
  const step = steps[index] ?? 'setup';
  const last = index === steps.length - 1;
  const set = (patch: Partial<RoleValues>) => setValues((current) => ({ ...current, ...patch }));
  const errors = step === 'setup' ? setupErrors(t, values) : {};
  const ready = Object.keys(errors).length === 0;
  const dirty = JSON.stringify(values) !== JSON.stringify(initial);
  const sourceName = (id: string) =>
    id === DEFAULT_SOURCE ? 'Default' : (sources.data?.find((source) => source.id === id)?.name ?? id);

  const roleChoices = useMemo(() => {
    const pool = assignable.data ?? roles.data ?? [];
    return pool
      .filter((candidate) => candidate.id !== values.id)
      .map((candidate) => ({ value: candidate.id, label: candidate.id }));
  }, [assignable.data, roles.data, values.id]);

  const submit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!ready) {
      setShown(true);
      return;
    }
    if (!last) {
      setIndex(index + 1);
      setShown(false);
      return;
    }
    setSubmitting(true);
    setFailure(undefined);
    try {
      const body = toRoleRequest(values);
      if (creating) {
        await api.post(ROLES, body);
        toast.success(t('security.role.created', { id: body.id }));
      } else {
        await api.put(`${ROLES}${path`/${role.id}`}`, body);
        toast.success(t('security.role.saved', { id: role.id }));
      }
      await Promise.all([invalidate(ROLES), invalidate(USERS)]);
      leave(creating ? ROLES_PAGE : roleProfileHref(body.id));
    } catch (error) {
      const refused = refusedBy(t, error);
      setFailure(refused instanceof Error ? refused.message : String(refused));
    } finally {
      setSubmitting(false);
    }
  };

  const remove = async () => {
    if (!role) {
      return;
    }
    await api.delete(`${ROLES}${path`/${role.id}`}`);
    toast.success(t('security.roles.deleted', { id: role.id }));
    leave(ROLES_PAGE);
    await Promise.all([invalidate(ROLES), invalidate(USERS)]);
  };

  const stepName = t(STEP_NAMES[step]);
  return (
    <Page
      gutter="small"
      title={creating ? t('security.role.createTitle') : t('security.role.editTitle', { id: role.id })}
      description={
        creating
          ? t('security.role.stepCreate', { step: index + 1, name: stepName })
          : t('security.role.stepEdit', { step: index + 1, name: stepName, source: sourceName(role.source) })
      }
      breadcrumbs={[
        { label: t('security.crumbs.settings'), to: '/settings' },
        { label: t('security.roles.title'), to: ROLES_PAGE },
        { label: creating ? t('security.role.create') : role.id },
      ]}
    >
      <EditForm
        id="role-wizard"
        tight
        onSubmit={submit}
        bar={
          <EditBar
            start={
              <>
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => navigate(creating ? ROLES_PAGE : roleProfileHref(role.id))}
                >
                  {t('security.form.cancel')}
                </Button>
                {!creating && canDelete && !role.readOnly && (
                  <Button
                    type="button"
                    color="red"
                    size="3"
                    className={editStyles.bigButton}
                    onClick={() => setDeleting(true)}
                  >
                    <Trash2 size={16} aria-hidden="true" />
                    {t('security.role.delete')}
                  </Button>
                )}
              </>
            }
            end={
              <>
                {index > 0 && (
                  <Button type="button" variant="soft" onClick={() => setIndex(index - 1)}>
                    <span aria-hidden="true">←</span>
                    {t('security.wizard.back')}
                  </Button>
                )}
                {(!last || !readOnly) && (
                  <Button type="submit" disabled={!ready} loading={submitting}>
                    {!last
                      ? t('security.wizard.continue')
                      : creating
                        ? t('security.role.createSubmit')
                        : t('security.form.save')}
                  </Button>
                )}
              </>
            }
            progress={
              <Progress total={steps.length} done={index + 1} label={t('security.wizard.progress')} />
            }
          />
        }
      >
        <StepTitle>{stepName}</StepTitle>
        {step === 'type' && (
          <FormSection title={t('security.role.typeSection')}>
            <p className={styles.intro}>{t('security.role.typeIntro')}</p>
            <Choice<RoleType>
              label={t('security.role.type')}
              help={t('security.role.typeHint')}
              value={values.type}
              onChange={(type) =>
                set({
                  type,
                  source: type === 'external' ? (sources.data?.[0]?.id ?? '') : DEFAULT_SOURCE,
                })
              }
              options={[
                { value: 'astrum', label: t('security.role.typeAstrum') },
                { value: 'external', label: t('security.role.typeExternal') },
              ]}
            />
          </FormSection>
        )}
        {step === 'setup' && (
          <FormSection title={t('security.role.setupSection')}>
            {values.type === 'external' && (
              <>
                <Choice
                  label={t('security.role.source')}
                  required
                  help={t('security.role.sourceHint')}
                  value={values.source === DEFAULT_SOURCE ? '' : values.source}
                  disabled={!creating}
                  error={shown ? errors.source : undefined}
                  onChange={(source) => set({ source })}
                  options={(sources.data ?? []).map((source) => ({ value: source.id, label: source.name }))}
                />
                {creating && values.source !== '' && values.source !== DEFAULT_SOURCE && (
                  <MappedRoleField
                    key={values.source}
                    source={values.source}
                    sourceName={sourceName(values.source)}
                    value={values.id}
                    onPick={(id) =>
                      set({ id, name: values.name === '' || values.name === values.id ? id : values.name })
                    }
                  />
                )}
              </>
            )}
            <Input
              label={t('security.role.id')}
              name="id"
              required
              help={t('security.role.idHelp')}
              value={values.id}
              disabled={!creating || readOnly}
              autoFocus={creating}
              error={shown ? errors.id : undefined}
              onChange={(id) => set({ id })}
            />
            <Input
              label={t('security.roles.name')}
              name="name"
              required
              help={t('security.role.nameHelp')}
              value={values.name}
              disabled={readOnly}
              error={shown ? errors.name : undefined}
              onChange={(name) => set({ name })}
            />
            <Area
              label={t('security.roles.descriptionColumn')}
              name="description"
              mono={false}
              rows={3}
              help={t('security.role.descriptionHelp')}
              value={values.description}
              disabled={readOnly}
              onChange={(description) => set({ description })}
            />
          </FormSection>
        )}
        {step === 'privileges' && (
          <FormSection title={t('security.role.stepPrivileges')}>
            <p className={styles.intro}>{t('security.role.privilegesIntro')}</p>
            {privileges.isPending ? (
              <Flex justify="center" py="4">
                <Spinner />
              </Flex>
            ) : (
              <PrivilegePicker
                privileges={privileges.data ?? []}
                value={values.privileges}
                disabled={readOnly}
                onChange={(next) => set({ privileges: next })}
              />
            )}
          </FormSection>
        )}
        {step === 'roles' && (
          <FormSection title={t('security.role.stepRoles')}>
            <p className={styles.intro}>{t('security.role.rolesIntro')}</p>
            <TransferList
              label={t('security.role.stepRoles')}
              available={t('security.role.available')}
              chosen={t('security.role.contained')}
              options={roleChoices}
              value={values.roles}
              disabled={readOnly}
              onChange={(next) => set({ roles: next })}
              countLabel={(count) => t('security.transfer.items', { count })}
            />
          </FormSection>
        )}
        <FormError message={failure} />
      </EditForm>
      <UnsavedChangesGuard when={dirty && !leaving && !readOnly && !submitting} />
      <ConfirmDialog
        open={deleting}
        onOpenChange={setDeleting}
        onConfirm={remove}
        title={t('security.roles.deleteTitle', { id: role?.id ?? '' })}
        description={t('security.roles.deleteText')}
        confirmLabel={t('security.form.delete')}
        danger
      />
    </Page>
  );
}

export default function RolePage() {
  const { id } = useParams();
  const role = useApi<Role>(id === undefined ? null : `${ROLES}${path`/${id}`}`);
  if (id === undefined) {
    return <Wizard key="create" />;
  }
  if (role.isPending) {
    return (
      <Flex justify="center" py="8">
        <Spinner size="3" />
      </Flex>
    );
  }
  if (role.error || !role.data) {
    return (
      <Page gutter="small" title={id}>
        <ErrorState error={role.error} onRetry={() => void role.refetch()} />
      </Page>
    );
  }
  return <Wizard key={role.data.id} role={role.data} />;
}
