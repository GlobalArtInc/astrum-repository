import { Button, Flex, Spinner } from '@radix-ui/themes';
import { Eye, Trash2 } from 'lucide-react';
import { type FormEvent, useMemo, useState } from 'react';
import { useNavigate, useParams } from 'react-router';
import { api, path, useApi, useInvalidateApi } from '../../../api';
import { useCan } from '../../../auth/state';
import { type Translate, useT } from '../../../i18n';
import { ConfirmDialog, ErrorState, FormError, Page, UnsavedChangesGuard, useToast } from '../../../ui';
import { Area, Choice, Input } from '../../identity/shared/Fields';
import { explainExpression } from '../../selectors/explain';
import {
  type Action,
  type ContentSelector,
  PRIVILEGES,
  type Privilege,
  REPOSITORIES,
  type Repository,
  ROLES,
  SELECTORS,
} from '../api';
import { ActionBoxes, EditBar, EditForm, FormSection, Progress, StepTitle } from '../shared/Edit';
import editStyles from '../shared/Edit.module.css';
import { PRIVILEGES_PAGE, privilegeProfileHref } from '../shared/links';
import { refusedBy } from '../shared/messages';
import { useLeave } from '../shared/useLeave';
import { actionLabel, typeLabel } from './labels';
import { ANY, EMPTY_VALUES, fromPrivilege, type PrivilegeValues } from './model';
import styles from './PrivilegeForm.module.css';

const NAME = /^[A-Za-z0-9-][A-Za-z0-9_.-]*$/;
const FORMAT_PREFIX = 'format:';
const REPOSITORY_ACTIONS: Action[] = ['BROWSE', 'READ', 'EDIT', 'ADD', 'DELETE'];
const APPLICATION_ACTIONS: Action[] = [
  'ADD',
  'READ',
  'EDIT',
  'DELETE',
  'START',
  'STOP',
  'ASSOCIATE',
  'DISASSOCIATE',
];
const KNOWN_TYPES = [
  'application',
  'wildcard',
  'repository-view',
  'repository-admin',
  'repository-content-selector',
] as const;

type Step = 'settings' | 'content' | 'preview';

export function stepsOf(type: string): Step[] {
  return type === 'repository-content-selector'
    ? ['settings', 'content', 'preview']
    : ['settings', 'content'];
}

export function selectorTarget(values: Pick<PrivilegeValues, 'format' | 'repository'>): string {
  if (values.repository !== ANY) {
    return values.repository;
  }
  return values.format === ANY ? ANY : `${FORMAT_PREFIX}${values.format}`;
}

export function fromSelectorTarget(
  target: string,
  repositories: readonly Repository[],
): Pick<PrivilegeValues, 'format' | 'repository'> {
  if (target === ANY) {
    return { format: ANY, repository: ANY };
  }
  if (target.startsWith(FORMAT_PREFIX)) {
    return { format: target.slice(FORMAT_PREFIX.length), repository: ANY };
  }
  const repository = repositories.find((item) => item.name === target);
  return { format: repository?.format ?? ANY, repository: target };
}

export function formatOptions(t: Translate, repositories: readonly Repository[]) {
  const formats = [...new Set(repositories.map((repository) => repository.format))].sort();
  return [
    { value: ANY, label: t('security.privilege.allFormats') },
    ...formats.map((format) => ({ value: format, label: format })),
  ];
}

export function repositoryOptions(
  t: Translate,
  repositories: readonly Repository[],
  format: string,
  current: string,
) {
  const names = repositories
    .filter((repository) => format === ANY || repository.format === format)
    .map((repository) => repository.name);
  if (current !== ANY && current !== '' && !names.includes(current)) {
    names.unshift(current);
  }
  return [
    { value: ANY, label: t('security.privilege.allRepositories') },
    ...names.map((name) => ({ value: name, label: name })),
  ];
}

export function selectorTargetOptions(t: Translate, repositories: readonly Repository[], current: string) {
  const formats = [...new Set(repositories.map((repository) => repository.format))].sort();
  const names = repositories.map((repository) => repository.name);
  if (current !== ANY && current !== '' && !current.startsWith(FORMAT_PREFIX) && !names.includes(current)) {
    names.unshift(current);
  }
  return [
    { value: ANY, label: t('security.privilege.allRepositories') },
    ...formats.map((format) => ({
      value: `${FORMAT_PREFIX}${format}`,
      label: t('security.privilege.allRepositoriesOf', { format }),
    })),
    ...names.map((name) => ({ value: name, label: name })),
  ];
}

export function requestOf(type: string, values: PrivilegeValues): Record<string, unknown> {
  const common = { name: values.name.trim(), description: values.description.trim() };
  switch (type) {
    case 'application':
      return { ...common, domain: values.domain.trim(), actions: values.actions };
    case 'wildcard':
      return { ...common, pattern: values.pattern.trim() };
    case 'repository-content-selector':
      return {
        ...common,
        contentSelector: values.contentSelector,
        format: values.format,
        repository: values.repository,
        actions: values.actions,
      };
    default:
      return { ...common, format: values.format, repository: values.repository, actions: values.actions };
  }
}

export function settingsErrors(
  t: Translate,
  values: PrivilegeValues,
): Partial<Record<keyof PrivilegeValues, string>> {
  const name = values.name.trim();
  if (name === '') {
    return { name: t('security.form.required') };
  }
  return NAME.test(name) ? {} : { name: t('security.form.nameRule') };
}

export function contentErrors(
  t: Translate,
  type: string,
  values: PrivilegeValues,
): Partial<Record<keyof PrivilegeValues, string>> {
  const errors: Partial<Record<keyof PrivilegeValues, string>> = {};
  const required = t('security.form.required');
  if (type === 'application' && values.domain.trim() === '') {
    errors.domain = required;
  }
  if (type === 'wildcard' && values.pattern.trim() === '') {
    errors.pattern = required;
  }
  if (type === 'repository-content-selector' && values.contentSelector === '') {
    errors.contentSelector = required;
  }
  if (type !== 'wildcard' && values.actions.length === 0) {
    errors.actions = t('security.privilege.actionsRequired');
  }
  return errors;
}

function ContentFields({
  type,
  values,
  set,
  errors,
  disabled,
}: {
  type: string;
  values: PrivilegeValues;
  set(patch: Partial<PrivilegeValues>): void;
  errors: Partial<Record<keyof PrivilegeValues, string>>;
  disabled?: boolean;
}) {
  const t = useT();
  const canReadSelectors = useCan('astrum:selectors:read');
  const repositories = useApi<Repository[]>(type.startsWith('repository-') ? REPOSITORIES : null);
  const selectors = useApi<ContentSelector[]>(
    type === 'repository-content-selector' && canReadSelectors ? SELECTORS : null,
  );
  const privileges = useApi<Privilege[]>(type === 'application' ? PRIVILEGES : null);
  const domains = useMemo(() => {
    const found = new Set(
      (privileges.data ?? []).flatMap((privilege) => (privilege.domain ? [privilege.domain] : [])),
    );
    if (values.domain) {
      found.add(values.domain);
    }
    return [ANY, ...[...found].filter((domain) => domain !== ANY).sort()];
  }, [privileges.data, values.domain]);
  const repos = repositories.data ?? [];
  const actions = (list: Action[]) => {
    const options = values.actions.includes('ALL') ? [...list, 'ALL' as Action] : list;
    return options.map((action) => ({ value: action, label: actionLabel(t, action) }));
  };

  if (type === 'application') {
    return (
      <FormSection title={t('security.privilege.configuration')}>
        <Choice
          label={t('security.privilegeField.domain')}
          required
          help={t('security.privilege.domainHelp')}
          placeholder={t('security.privilege.domainPlaceholder')}
          value={values.domain}
          disabled={disabled}
          error={errors.domain}
          onChange={(domain) => set({ domain })}
          options={domains.map((domain) => ({
            value: domain,
            label: domain === ANY ? t('security.privilege.allDomains') : domain,
          }))}
        />
        <ActionBoxes
          label={t('security.privilegeField.actions')}
          required
          help={t('security.privilege.applicationActionsHelp')}
          options={actions(APPLICATION_ACTIONS)}
          value={values.actions}
          disabled={disabled}
          error={errors.actions}
          onChange={(next) => set({ actions: next })}
        />
      </FormSection>
    );
  }
  if (type === 'wildcard') {
    return (
      <FormSection title={t('security.privilege.configuration')}>
        <Input
          label={t('security.privilege.pattern')}
          required
          help={t('security.privilege.patternHelp')}
          value={values.pattern}
          disabled={disabled}
          error={errors.pattern}
          onChange={(pattern) => set({ pattern })}
        />
      </FormSection>
    );
  }
  if (type === 'repository-content-selector') {
    return (
      <>
        <FormSection title={t('security.privilege.content')}>
          <Choice
            label={t('security.privilegeField.contentSelector')}
            required
            help={t('security.privilege.selectorHelp')}
            placeholder={t('security.privilege.selectorPlaceholder')}
            value={values.contentSelector}
            disabled={disabled}
            error={errors.contentSelector}
            onChange={(contentSelector) => set({ contentSelector })}
            options={[
              ...(selectors.data ?? []).map((selector) => ({ value: selector.name, label: selector.name })),
              ...(values.contentSelector &&
              !selectors.data?.some((item) => item.name === values.contentSelector)
                ? [{ value: values.contentSelector, label: values.contentSelector }]
                : []),
            ]}
          />
        </FormSection>
        <FormSection title={t('security.privilegeField.repository')}>
          <Choice
            label={t('security.privilegeField.repository')}
            required
            help={t('security.privilege.selectorRepositoryHelp')}
            placeholder={t('security.privilege.repositoryPlaceholder')}
            value={selectorTarget(values)}
            disabled={disabled}
            onChange={(target) => set(fromSelectorTarget(target, repos))}
            options={selectorTargetOptions(t, repos, selectorTarget(values))}
          />
        </FormSection>
        <FormSection title={t('security.privilegeField.actions')}>
          <ActionBoxes
            label={t('security.privilegeField.actions')}
            required
            help={t('security.privilege.selectorActionsHelp')}
            options={actions(REPOSITORY_ACTIONS)}
            value={values.actions}
            disabled={disabled}
            error={errors.actions}
            onChange={(next) => set({ actions: next })}
          />
        </FormSection>
      </>
    );
  }
  return (
    <FormSection title={t('security.privilege.configuration')}>
      <Choice
        label={t('security.privilege.repositoryFormat')}
        required
        help={t('security.privilege.formatHelp')}
        placeholder={t('security.privilege.formatPlaceholder')}
        value={values.format}
        disabled={disabled}
        onChange={(format) => {
          const match = repos.find((item) => item.name === values.repository);
          set({
            format,
            repository: format !== ANY && match && match.format !== format ? ANY : values.repository,
          });
        }}
        options={formatOptions(t, repos)}
      />
      <Choice
        label={t('security.privilegeField.repository')}
        required
        help={t('security.privilege.repositoryHelp')}
        placeholder={t('security.privilege.repositoryPlaceholder')}
        value={values.repository}
        disabled={disabled}
        onChange={(repository) => set({ repository })}
        options={repositoryOptions(t, repos, values.format, values.repository)}
      />
      <ActionBoxes
        label={t('security.privilegeField.actions')}
        required
        help={t('security.privilege.repositoryActionsHelp')}
        options={actions(REPOSITORY_ACTIONS)}
        value={values.actions}
        disabled={disabled}
        error={errors.actions}
        onChange={(next) => set({ actions: next })}
      />
    </FormSection>
  );
}

function SettingsFields({
  values,
  set,
  errors,
  nameLocked,
  disabled,
}: {
  values: PrivilegeValues;
  set(patch: Partial<PrivilegeValues>): void;
  errors: Partial<Record<keyof PrivilegeValues, string>>;
  nameLocked?: boolean;
  disabled?: boolean;
}) {
  const t = useT();
  return (
    <FormSection title={t('security.privilege.settings')}>
      <Input
        label={t('security.privilegeField.name')}
        name="name"
        required
        help={t('security.privilege.nameHelp')}
        value={values.name}
        disabled={nameLocked || disabled}
        autoFocus={!nameLocked}
        error={errors.name}
        onChange={(name) => set({ name })}
      />
      <Area
        label={t('security.privilegeField.description')}
        name="description"
        mono={false}
        rows={3}
        help={t('security.privilege.descriptionHelp')}
        value={values.description}
        disabled={disabled}
        onChange={(description) => set({ description })}
      />
    </FormSection>
  );
}

function SelectorPreview({ values }: { values: PrivilegeValues }) {
  const t = useT();
  const selectors = useApi<ContentSelector[]>(SELECTORS);
  const selector = selectors.data?.find((item) => item.name === values.contentSelector);
  return (
    <section className={styles.preview} aria-label={t('security.privilege.scope')}>
      <h3 className={styles.previewTitle}>
        <Eye size={16} aria-hidden="true" />
        {t('security.privilege.scope')}
      </h3>
      <div className={styles.placeholder}>
        {values.repository === ANY
          ? t('security.privilege.scopeChoose')
          : t('security.privilege.scopeRepository', { repository: values.repository })}
      </div>
      {selector && (
        <div className={styles.logic}>
          <span className={styles.logicLabel}>{t('security.privilege.logic')}</span>
          <span className={styles.logicName}>{selector.name}</span>
          <p className={styles.logicText}>{explainExpression(t, selector.expression)}</p>
          <pre className={styles.logicCode}>{selector.expression}</pre>
        </div>
      )}
    </section>
  );
}

function useSave() {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  return async (type: string, values: PrivilegeValues, existing?: Privilege) => {
    try {
      const body = requestOf(type, values);
      if (existing) {
        await api.put(`${PRIVILEGES}${path`/${type}/${existing.name}`}`, body);
        toast.success(t('security.privilege.saved', { name: existing.name }));
      } else {
        await api.post(`${PRIVILEGES}${path`/${type}`}`, body);
        toast.success(t('security.privilege.created', { name: values.name.trim() }));
      }
      await invalidate(PRIVILEGES);
      return null;
    } catch (error) {
      return refusedBy(t, error);
    }
  };
}

function errorText(error: unknown): string | undefined {
  if (!error) {
    return undefined;
  }
  return error instanceof Error ? error.message : String(error);
}

function Wizard({ type }: { type: string }) {
  const t = useT();
  const navigate = useNavigate();
  const save = useSave();
  const { leaving, leave } = useLeave();
  const steps = stepsOf(type);
  const [index, setIndex] = useState(0);
  const [values, setValues] = useState<PrivilegeValues>(EMPTY_VALUES);
  const [submitting, setSubmitting] = useState(false);
  const [failure, setFailure] = useState<string | undefined>();
  const [shown, setShown] = useState(false);
  const step = steps[index] ?? 'settings';
  const set = (patch: Partial<PrivilegeValues>) => setValues((current) => ({ ...current, ...patch }));
  const errors = step === 'settings' ? settingsErrors(t, values) : contentErrors(t, type, values);
  const ready = Object.keys(step === 'preview' ? contentErrors(t, type, values) : errors).length === 0;
  const last = index === steps.length - 1;
  const dirty = values !== EMPTY_VALUES;

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
    const problem = await save(type, values);
    setSubmitting(false);
    if (problem) {
      setFailure(errorText(problem));
      return;
    }
    leave(PRIVILEGES_PAGE);
  };

  return (
    <Page
      gutter="small"
      title={t('security.privilege.createTitleOf', { type: typeLabel(t, type) })}
      description={t('security.privilege.stepSettings', { step: index + 2 })}
      breadcrumbs={[
        { label: t('security.crumbs.settings'), to: '/settings' },
        { label: t('security.privileges.title'), to: PRIVILEGES_PAGE },
        { label: t('security.privilege.create') },
      ]}
    >
      <EditForm
        id="privilege-wizard"
        onSubmit={submit}
        bar={
          <EditBar
            start={
              <Button type="button" variant="outline" onClick={() => navigate(PRIVILEGES_PAGE)}>
                {t('security.form.cancel')}
              </Button>
            }
            end={
              <>
                <Button
                  type="button"
                  variant="soft"
                  onClick={() => (index === 0 ? navigate(`${PRIVILEGES_PAGE}/create`) : setIndex(index - 1))}
                >
                  <span aria-hidden="true">←</span>
                  {t('security.wizard.back')}
                </Button>
                <Button type="submit" disabled={!ready} loading={submitting}>
                  {last ? t('security.privilege.createSubmit') : t('security.wizard.continue')}
                </Button>
              </>
            }
            progress={
              <Progress total={steps.length + 1} done={index + 2} label={t('security.wizard.progress')} />
            }
          />
        }
      >
        <StepTitle>
          {step === 'preview' ? t('security.privilege.previewStep') : t('security.privilege.configuration')}
        </StepTitle>
        {step === 'settings' && <SettingsFields values={values} set={set} errors={shown ? errors : {}} />}
        {step === 'content' && (
          <ContentFields type={type} values={values} set={set} errors={shown ? errors : {}} />
        )}
        {step === 'preview' && <SelectorPreview values={values} />}
        <FormError message={failure} />
      </EditForm>
      <UnsavedChangesGuard when={dirty && !leaving && !submitting} />
    </Page>
  );
}

function Editor({ privilege }: { privilege: Privilege }) {
  const t = useT();
  const navigate = useNavigate();
  const save = useSave();
  const invalidate = useInvalidateApi();
  const toast = useToast();
  const { leaving, leave } = useLeave();
  const canWrite = useCan('astrum:privileges:update');
  const canDelete = useCan('astrum:privileges:delete');
  const readOnly = privilege.readOnly || !canWrite;
  const initial = useMemo(() => fromPrivilege(privilege), [privilege]);
  const [values, setValues] = useState<PrivilegeValues>(initial);
  const [submitting, setSubmitting] = useState(false);
  const [failure, setFailure] = useState<string | undefined>();
  const [deleting, setDeleting] = useState(false);
  const [shown, setShown] = useState(false);
  const set = (patch: Partial<PrivilegeValues>) => setValues((current) => ({ ...current, ...patch }));
  const dirty = JSON.stringify(values) !== JSON.stringify(initial);
  const errors = contentErrors(t, privilege.type, values);

  const submit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (Object.keys(errors).length > 0) {
      setShown(true);
      return;
    }
    setSubmitting(true);
    const problem = await save(privilege.type, values, privilege);
    setSubmitting(false);
    setFailure(errorText(problem));
  };

  const remove = async () => {
    await api.delete(`${PRIVILEGES}${path`/${privilege.name}`}`);
    toast.success(t('security.privileges.deleted', { name: privilege.name }));
    leave(PRIVILEGES_PAGE);
    await Promise.all([invalidate(PRIVILEGES), invalidate(ROLES)]);
  };

  return (
    <Page
      gutter="small"
      title={t('security.privilege.editTitle', { name: privilege.name })}
      description={t('security.privilege.typeIs', { type: typeLabel(t, privilege.type) })}
      breadcrumbs={[
        { label: t('security.crumbs.settings'), to: '/settings' },
        { label: t('security.privileges.title'), to: PRIVILEGES_PAGE },
        { label: privilege.name },
      ]}
    >
      <EditForm
        id="privilege-edit"
        onSubmit={submit}
        bar={
          <EditBar
            start={
              !readOnly &&
              canDelete && (
                <Button
                  type="button"
                  color="red"
                  size="3"
                  className={editStyles.bigButton}
                  onClick={() => setDeleting(true)}
                >
                  <Trash2 size={16} aria-hidden="true" />
                  {t('security.form.delete')}
                </Button>
              )
            }
            end={
              <>
                <Button
                  type="button"
                  variant="outline"
                  onClick={() =>
                    dirty ? setValues(initial) : navigate(privilegeProfileHref(privilege.name))
                  }
                >
                  {t('security.form.discard')}
                </Button>
                {!readOnly && (
                  <Button type="submit" disabled={!dirty} loading={submitting}>
                    {t('security.form.save')}
                  </Button>
                )}
              </>
            }
          />
        }
      >
        <SettingsFields values={values} set={set} errors={{}} nameLocked disabled={readOnly} />
        <ContentFields
          type={privilege.type}
          values={values}
          set={set}
          errors={shown ? errors : {}}
          disabled={readOnly}
        />
        <FormError message={failure} />
      </EditForm>
      <UnsavedChangesGuard when={dirty && !leaving && !readOnly} />
      <ConfirmDialog
        open={deleting}
        onOpenChange={setDeleting}
        onConfirm={remove}
        title={t('security.privileges.deleteTitle', { name: privilege.name })}
        description={t('security.privileges.deleteText')}
        confirmLabel={t('security.form.delete')}
        danger
      />
    </Page>
  );
}

export default function PrivilegePage() {
  const t = useT();
  const { type, name } = useParams();
  const existing = useApi<Privilege>(name === undefined ? null : `${PRIVILEGES}${path`/${name}`}`);
  if (name === undefined) {
    if (!KNOWN_TYPES.includes((type ?? '') as (typeof KNOWN_TYPES)[number])) {
      return (
        <Page gutter="small" title={t('security.privilege.createTitle')}>
          <ErrorState error={new Error(t('security.privilege.unknownType', { type: type ?? '' }))} />
        </Page>
      );
    }
    return <Wizard key={type} type={type ?? ''} />;
  }
  if (existing.isPending) {
    return (
      <Flex justify="center" py="8">
        <Spinner size="3" />
      </Flex>
    );
  }
  if (existing.error || !existing.data) {
    return (
      <Page gutter="small" title={name}>
        <ErrorState error={existing.error} onRetry={() => void existing.refetch()} />
      </Page>
    );
  }
  return <Editor key={existing.data.name} privilege={existing.data} />;
}
