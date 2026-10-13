import { AlertDialog, Button, Flex, Spinner, Text, Tooltip } from '@radix-ui/themes';
import { ExternalLink, Info, Trash2, TriangleAlert } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { Link, useNavigate, useParams } from 'react-router';
import { ApiError, api, fieldName, path, useApi, useInvalidateApi } from '../../api';
import { useT } from '../../i18n';
import {
  describeError,
  ErrorState,
  FormError,
  Page,
  SelectField,
  TextAreaField,
  TextField,
  UnsavedChangesGuard,
  useToast,
} from '../../ui';
import { SettingsFields, SettingsForm, SettingsSection, settingsButton } from '../../ui/SettingsForm';
import { MatcherList, MODES, RuleSummary } from './MatcherEditor';
import {
  DOCS,
  emptyRule,
  fromRule,
  LIST_PAGE,
  type Mode,
  type Problems,
  type RoutingRuleDetails,
  RULES,
  type RuleValues,
  sameValues,
  toRequest,
  UI_RULES,
  validate,
} from './model';
import styles from './Routing.module.css';

function DeleteRuleDialog({
  open,
  onOpenChange,
  name,
  onDelete,
}: {
  open: boolean;
  onOpenChange(open: boolean): void;
  name: string;
  onDelete(): Promise<void>;
}) {
  const t = useT();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const confirm = async () => {
    setBusy(true);
    setError(null);
    try {
      await onDelete();
      onOpenChange(false);
    } catch (failure) {
      setError(failure);
    } finally {
      setBusy(false);
    }
  };
  return (
    <AlertDialog.Root open={open} onOpenChange={(next) => !busy && onOpenChange(next)}>
      <AlertDialog.Content maxWidth="450px">
        <AlertDialog.Title>{t('routing.delete.title')}</AlertDialog.Title>
        <AlertDialog.Description size="2">{t('routing.delete.text')}</AlertDialog.Description>
        <Flex mt="4" p="4" className={styles.dangerBox}>
          <Text size="2" color="red">
            {name}
          </Text>
        </Flex>
        {error !== null && (
          <Text as="p" size="2" color="red" mt="3" role="alert">
            {describeError(t, error)}
          </Text>
        )}
        <Flex gap="3" mt="4" justify="end">
          <AlertDialog.Cancel>
            <Button variant="soft" color="gray" disabled={busy}>
              {t('routing.actions.cancel')}
            </Button>
          </AlertDialog.Cancel>
          <Button color="red" loading={busy} onClick={() => void confirm()}>
            {t('routing.actions.delete')}
          </Button>
        </Flex>
      </AlertDialog.Content>
    </AlertDialog.Root>
  );
}

function RuleForm({
  rule,
  leave,
  leaving,
}: {
  rule?: RoutingRuleDetails;
  leave(to: string): void;
  leaving: boolean;
}) {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const creating = rule === undefined;
  const initial = useMemo(() => (rule ? fromRule(rule) : emptyRule()), [rule]);
  const [values, setValues] = useState<RuleValues>(initial);
  const [problems, setProblems] = useState<Problems>({});
  const [formError, setFormError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [testPath, setTestPath] = useState('');
  const dirty = !sameValues(values, initial);
  const assigned = rule?.assignedRepositoryNames ?? [];
  const assignedCount = rule?.assignedRepositoryCount ?? 0;
  const texts = {
    nameRequired: t('routing.problems.nameRequired'),
    namePattern: t('routing.problems.namePattern'),
    nameLength: t('routing.problems.nameLength'),
    matchersRequired: t('routing.problems.matchersRequired'),
  };

  const set = <K extends keyof RuleValues>(key: K, value: RuleValues[K]) => {
    setValues((current) => ({ ...current, [key]: value }));
    setProblems((current) => {
      const next = { ...current };
      delete next[key as keyof Problems];
      return next;
    });
  };

  const save = async () => {
    const found = validate(values, texts);
    setProblems(found);
    if (Object.keys(found).length > 0) {
      return;
    }
    setSaving(true);
    setFormError(null);
    const body = toRequest(values);
    try {
      if (creating) {
        await api.post(RULES, body);
        toast.success(t('routing.toasts.created', { name: body.name }));
      } else {
        await api.put(`${RULES}${path`/${rule.name}`}`, body);
        toast.success(t('routing.toasts.updated', { name: body.name }));
      }
      await invalidate(UI_RULES);
      await invalidate(RULES);
      leave(LIST_PAGE);
    } catch (error) {
      if (error instanceof ApiError && error.fieldErrors.length > 0) {
        const mapped: Problems = {};
        const unmatched: string[] = [];
        for (const problem of error.fieldErrors) {
          const key = fieldName(problem.id);
          if (key === 'name' || key === 'mode' || key === 'description') {
            mapped[key] = problem.message;
          } else if (key.startsWith('matchers')) {
            mapped.matchers = mapped.matchers ? `${mapped.matchers} ${problem.message}` : problem.message;
          } else {
            unmatched.push(problem.message);
          }
        }
        setProblems(mapped);
        setFormError(unmatched.length > 0 ? unmatched.join(' ') : null);
      } else {
        setFormError(describeError(t, error));
      }
    } finally {
      setSaving(false);
    }
  };

  const remove = async () => {
    if (!rule) {
      return;
    }
    await api.delete(`${RULES}${path`/${rule.name}`}`);
    toast.success(t('routing.toasts.deleted', { name: rule.name }));
    await invalidate(UI_RULES);
    await invalidate(RULES);
    leave(LIST_PAGE);
  };

  const modeOptions = [
    { value: 'BLOCK', label: t(MODES.BLOCK) },
    { value: 'ALLOW', label: t(MODES.ALLOW) },
  ];
  const hint = values.mode === 'BLOCK' ? t('routing.form.blockHint') : t('routing.form.allowHint');
  const deleteButton = (
    <Button
      type="button"
      size="3"
      variant="soft"
      color="gray"
      className={`${settingsButton} ${styles.deleteButton}`}
      disabled={assignedCount > 0}
      onClick={() => setDeleting(true)}
    >
      <Trash2 size={16} aria-hidden="true" />
      {t('routing.actions.delete')}
    </Button>
  );

  return (
    <SettingsForm
      label={creating ? t('routing.form.createTitle') : t('routing.form.editTitle', { name: rule.name })}
      onSubmit={() => void save()}
      unsaved={dirty ? t('routing.form.unsaved') : null}
      start={
        !creating &&
        (assignedCount > 0 ? (
          <Tooltip content={t('routing.form.cannotDelete', { count: assignedCount })}>
            <span>{deleteButton}</span>
          </Tooltip>
        ) : (
          deleteButton
        ))
      }
      end={
        <>
          <Button asChild variant="outline">
            <Link to={LIST_PAGE}>{t('routing.actions.cancel')}</Link>
          </Button>
          <Button type="submit" loading={saving} disabled={!dirty}>
            {creating ? t('routing.actions.create') : t('routing.actions.save')}
          </Button>
        </>
      }
    >
      <UnsavedChangesGuard when={dirty && !leaving} />
      <SettingsSection title={t('routing.form.basic')}>
        <SettingsFields>
          <TextField
            label={t('routing.form.name')}
            hint={t('routing.form.nameHint')}
            placeholder={t('routing.form.namePlaceholder')}
            required
            value={values.name}
            onChange={(value) => set('name', value)}
            error={problems.name}
            autoFocus={creating}
            maxLength={200}
          />
          <TextAreaField
            label={t('routing.form.description')}
            hint={t('routing.form.descriptionHint')}
            placeholder={t('routing.form.descriptionPlaceholder')}
            value={values.description}
            onChange={(value) => set('description', value)}
            error={problems.description}
            rows={3}
          />
          <SelectField
            label={t('routing.form.mode')}
            hint={hint}
            required
            value={values.mode}
            onChange={(value) => set('mode', value as Mode)}
            options={modeOptions}
            error={problems.mode}
          />
        </SettingsFields>
      </SettingsSection>
      <SettingsSection title={t('routing.form.matchers')}>
        <MatcherList
          mode={values.mode}
          matchers={values.matchers}
          onChange={(next) => set('matchers', next)}
          path={testPath}
          onPathChange={setTestPath}
          problem={problems.matchers}
        />
        <RuleSummary mode={values.mode} matchers={values.matchers} />
      </SettingsSection>
      <FormError message={formError} />
      <hr className={styles.divider} />
      <section className={styles.help} aria-labelledby="routing-about">
        <h2 id="routing-about" className={styles.helpTitle}>
          <Info size={16} aria-hidden="true" />
          {t('routing.form.about')}
        </h2>
        <p className={styles.helpText}>
          {t('routing.form.aboutText')}{' '}
          <a className={styles.helpLink} href={DOCS} target="_blank" rel="noreferrer">
            {t('routing.form.learnMore')}
            <ExternalLink size={12} aria-hidden="true" />
          </a>
        </p>
      </section>
      {assignedCount > 0 && (
        <div className={styles.warning} role="note">
          <TriangleAlert size={18} aria-hidden="true" className={styles.warningIcon} />
          <span>
            {t('routing.form.assignedWarning', { count: assignedCount, names: assigned.join(', ') })}
          </span>
        </div>
      )}
      {rule && (
        <DeleteRuleDialog open={deleting} onOpenChange={setDeleting} name={rule.name} onDelete={remove} />
      )}
    </SettingsForm>
  );
}

export default function RoutingRulePage() {
  const t = useT();
  const { name } = useParams();
  const navigate = useNavigate();
  const [target, setTarget] = useState<string | null>(null);
  useEffect(() => {
    if (target !== null) {
      navigate(target);
    }
  }, [target, navigate]);
  const creating = name === undefined;
  const rule = useApi<RoutingRuleDetails>(creating ? null : `${UI_RULES}${path`/${name}`}`);
  const crumbs = [
    { label: t('routing.crumbs.settings'), to: '/settings' },
    { label: t('routing.list.title'), to: LIST_PAGE },
    { label: creating ? t('routing.crumbs.create') : (name ?? '') },
  ];
  const data = rule.data;
  const line = data
    ? [
        t('routing.form.line', { count: data.matchers.length, mode: t(MODES[data.mode]) }),
        ...(data.assignedRepositoryCount > 0
          ? [t('routing.form.assigned', { count: data.assignedRepositoryCount })]
          : []),
      ].join(' • ')
    : undefined;
  return (
    <Page
      className={styles.page}
      title={creating ? t('routing.form.createTitle') : t('routing.form.editTitle', { name: name ?? '' })}
      description={creating ? undefined : line}
      breadcrumbs={crumbs}
    >
      {!creating && rule.isPending ? (
        <Flex justify="center" py="8">
          <Spinner size="3" />
        </Flex>
      ) : !creating && rule.error ? (
        <ErrorState error={rule.error} />
      ) : (
        <RuleForm
          key={name ?? 'create'}
          rule={creating ? undefined : data}
          leave={setTarget}
          leaving={target !== null}
        />
      )}
    </Page>
  );
}
