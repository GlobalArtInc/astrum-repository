import {
  AlertDialog,
  Button,
  Checkbox,
  Flex,
  TextField as RadixTextField,
  Spinner,
  Text,
} from '@radix-ui/themes';
import { Trash2 } from 'lucide-react';
import { type ReactNode, useEffect, useId, useMemo, useState } from 'react';
import { Link, useNavigate, useParams } from 'react-router';
import { ApiError, api, fieldName, path, useApi, useInvalidateApi } from '../../api';
import { useCan } from '../../auth/state';
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
import styles from './Cleanup.module.css';
import {
  ANY_RELEASE,
  available,
  CREATE,
  DELETE,
  EMPTY_POLICY,
  FORMATS,
  type FormatCriteria,
  formatLabel,
  fromPolicy,
  LIST_PAGE,
  MAX_DAYS,
  POLICIES,
  type Policy,
  type PolicyValues,
  policyHref,
  toRequest,
  UPDATE,
  validate,
} from './model';
import { PreviewSection } from './PreviewSection';

type Problems = Partial<Record<keyof PolicyValues | 'criteria', string>>;

function Criterion({
  title,
  description,
  checked,
  onCheckedChange,
  disabled,
  children,
}: {
  title: string;
  description: string;
  checked: boolean;
  onCheckedChange(checked: boolean): void;
  disabled?: boolean;
  children?: ReactNode;
}) {
  const id = useId();
  return (
    <div className={styles.criterion}>
      <Checkbox
        id={id}
        checked={checked}
        disabled={disabled}
        onCheckedChange={(value) => onCheckedChange(value === true)}
        aria-describedby={`${id}-text`}
      />
      <div className={styles.criterionBody}>
        <label htmlFor={id} className={styles.criterionTitle}>
          {title}
        </label>
        <p id={`${id}-text`} className={styles.criterionText}>
          {description}
        </p>
        {checked && <div className={styles.criterionInput}>{children}</div>}
      </div>
    </div>
  );
}

function ProblemText({ message }: { message?: string }) {
  return message ? (
    <Text as="p" size="1" color="red" role="alert" mt="1">
      {message}
    </Text>
  ) : null;
}

function DeletePolicyDialog({
  open,
  onOpenChange,
  policy,
  onDelete,
}: {
  open: boolean;
  onOpenChange(open: boolean): void;
  policy: Policy;
  onDelete(): Promise<void>;
}) {
  const t = useT();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const used = policy.repositories.length;
  const change = (next: boolean) => {
    if (!busy) {
      setError(null);
      onOpenChange(next);
    }
  };
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
    <AlertDialog.Root open={open} onOpenChange={change}>
      <AlertDialog.Content maxWidth="450px">
        <AlertDialog.Title>{t('cleanup.delete.title')}</AlertDialog.Title>
        <AlertDialog.Description size="2">
          {used > 0 && `${t('cleanup.form.usedBy', { count: used })} `}
          {t('cleanup.delete.text')}
        </AlertDialog.Description>
        <Flex mt="4" p="4" className={styles.dangerBox}>
          <Text size="2" color="red" className={styles.dialogName}>
            {policy.name}
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
              {t('cleanup.actions.cancel')}
            </Button>
          </AlertDialog.Cancel>
          <Button color="red" loading={busy} onClick={() => void confirm()}>
            {t('cleanup.actions.delete')}
          </Button>
        </Flex>
      </AlertDialog.Content>
    </AlertDialog.Root>
  );
}

function PolicyForm({
  policy,
  formats,
  leave,
  leaving,
}: {
  policy?: Policy;
  formats: FormatCriteria[];
  leave(to: string): void;
  leaving: boolean;
}) {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const creating = policy === undefined;
  const canWrite = useCan(creating ? CREATE : UPDATE);
  const canDelete = useCan(DELETE);
  const readOnly = !canWrite;
  const initial = useMemo(() => (policy ? fromPolicy(policy) : EMPTY_POLICY), [policy]);
  const [values, setValues] = useState<PolicyValues>(initial);
  const [problems, setProblems] = useState<Problems>({});
  const [formError, setFormError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const criteria = useMemo(() => available(formats, values.format), [formats, values.format]);
  const dirty = JSON.stringify(values) !== JSON.stringify(initial);
  const texts = {
    name: t('cleanup.problems.name'),
    nameRule: t('cleanup.problems.nameRule'),
    format: t('cleanup.problems.format'),
    notes: t('cleanup.problems.notes'),
    criteria: t('cleanup.problems.criteria'),
    age: t('cleanup.problems.age'),
    usage: t('cleanup.problems.usage'),
    days: t('cleanup.problems.days', { max: MAX_DAYS }),
    regex: t('cleanup.problems.regex'),
    regexInvalid: t('cleanup.problems.regexInvalid'),
  };
  const live = validate(values, criteria, creating, texts);

  const set = <K extends keyof PolicyValues>(name: K, value: PolicyValues[K]) => {
    setValues((current) => ({ ...current, [name]: value }));
    setProblems((current) => {
      const next = { ...current };
      delete next[name];
      delete next.criteria;
      return next;
    });
  };

  const save = async () => {
    const found = validate(values, criteria, creating, texts);
    setProblems(found);
    if (Object.keys(found).length > 0) {
      return;
    }
    setSaving(true);
    setFormError(null);
    const body = toRequest(values, criteria);
    try {
      if (creating) {
        await api.post(POLICIES, body);
        toast.success(t('cleanup.toasts.created', { name: body.name }));
        await invalidate(POLICIES);
        leave(policyHref(body.name));
      } else {
        await api.put(`${POLICIES}${path`/${policy.name}`}`, body);
        toast.success(t('cleanup.toasts.updated', { name: policy.name }));
        await invalidate(POLICIES);
        leave(LIST_PAGE);
      }
    } catch (error) {
      if (error instanceof ApiError && error.fieldErrors.length > 0) {
        const mapped: Problems = {};
        const keys: Record<string, keyof PolicyValues | 'criteria'> = {
          name: 'name',
          format: 'format',
          notes: 'notes',
          criteriaLastBlobUpdated: 'age',
          criteriaLastDownloaded: 'usage',
          criteriaAssetRegex: 'regex',
          criteriaReleaseType: 'releaseType',
        };
        const unmatched: string[] = [];
        for (const problem of error.fieldErrors) {
          const key = keys[fieldName(problem.id).replace(/^PARAMETER /, '')];
          if (key) {
            mapped[key] = problem.message;
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
    if (!policy) {
      return;
    }
    await api.delete(`${POLICIES}${path`/${policy.name}`}`);
    toast.success(t('cleanup.toasts.deleted', { name: policy.name }));
    await invalidate(POLICIES);
    leave(LIST_PAGE);
  };

  const all = t('cleanup.formats.all');
  const formatOptions = formats.map((format) => ({ value: format.id, label: formatLabel(format.id, all) }));
  const releaseOptions = [
    { value: ANY_RELEASE, label: t('cleanup.form.releaseAny') },
    { value: 'RELEASES', label: t('cleanup.form.releases') },
    { value: 'PRERELEASES', label: t('cleanup.form.prereleases') },
  ];
  const hasDays = criteria.has('lastBlobUpdated') || criteria.has('lastDownloaded') || criteria.has('regex');

  return (
    <SettingsForm
      label={creating ? t('cleanup.form.createTitle') : t('cleanup.form.editTitle', { name: policy.name })}
      onSubmit={() => void save()}
      unsaved={dirty && !readOnly ? t('cleanup.form.unsaved') : null}
      start={
        !creating &&
        canDelete && (
          <Button
            type="button"
            size="3"
            color="red"
            className={settingsButton}
            onClick={() => setDeleting(true)}
          >
            <Trash2 size={16} aria-hidden="true" />
            {t('cleanup.actions.delete')}
          </Button>
        )
      }
      end={
        <>
          <Button asChild variant="outline">
            <Link to={LIST_PAGE}>{t('cleanup.actions.cancel')}</Link>
          </Button>
          {!readOnly && (
            <Button type="submit" loading={saving} disabled={!dirty}>
              {creating ? t('cleanup.actions.create') : t('cleanup.actions.save')}
            </Button>
          )}
        </>
      }
    >
      <UnsavedChangesGuard when={dirty && !leaving && !readOnly} />
      <Text as="p" size="2" className={styles.intro}>
        {t.rich('cleanup.form.intro', {
          policies: <strong key="p">{t('cleanup.form.introPolicies')}</strong>,
          criteria: <strong key="c">{t('cleanup.form.introCriteria')}</strong>,
        })}
      </Text>
      <SettingsSection title={t('cleanup.form.settings')}>
        <SettingsFields>
          <TextField
            label={t('cleanup.form.name')}
            hint={t('cleanup.form.nameHint')}
            placeholder={t('cleanup.form.namePlaceholder')}
            required
            value={values.name}
            onChange={(value) => set('name', value)}
            error={problems.name}
            disabled={readOnly || !creating}
            autoFocus={creating}
            maxLength={255}
          />
          <SelectField
            label={t('cleanup.form.format')}
            hint={t('cleanup.form.formatHint')}
            placeholder={t('cleanup.form.formatPlaceholder')}
            required
            value={values.format}
            onChange={(value) => set('format', value)}
            options={formatOptions}
            error={problems.format}
            disabled={readOnly}
          />
          <TextAreaField
            label={t('cleanup.form.notes')}
            hint={t('cleanup.form.notesHint')}
            value={values.notes}
            onChange={(value) => set('notes', value)}
            error={problems.notes}
            disabled={readOnly}
            rows={4}
          />
        </SettingsFields>
      </SettingsSection>
      {criteria.has('isPrerelease') && (
        <SettingsSection title={t('cleanup.form.release')}>
          <SettingsFields>
            <SelectField
              label={t('cleanup.form.release')}
              hint={t('cleanup.form.releaseText')}
              value={values.releaseType}
              onChange={(value) => set('releaseType', value as PolicyValues['releaseType'])}
              options={releaseOptions}
              error={problems.releaseType}
              disabled={readOnly}
            />
          </SettingsFields>
        </SettingsSection>
      )}
      {values.format !== '' && hasDays && (
        <SettingsSection title={t('cleanup.form.criteria')} description={t('cleanup.form.criteriaText')}>
          <div className={styles.criteria}>
            {criteria.has('lastBlobUpdated') && (
              <Criterion
                title={t('cleanup.form.age')}
                description={t('cleanup.form.ageText', { max: MAX_DAYS })}
                checked={values.ageOn}
                onCheckedChange={(checked) => set('ageOn', checked)}
                disabled={readOnly}
              >
                <RadixTextField.Root
                  className={styles.days}
                  type="number"
                  inputMode="numeric"
                  min={1}
                  value={values.age}
                  placeholder={t('cleanup.form.daysPlaceholder')}
                  aria-label={t('cleanup.form.age')}
                  aria-invalid={problems.age ? true : undefined}
                  disabled={readOnly}
                  onChange={(event) => set('age', event.target.value)}
                />
                <ProblemText message={problems.age} />
              </Criterion>
            )}
            {criteria.has('lastDownloaded') && (
              <Criterion
                title={t('cleanup.form.usage')}
                description={t('cleanup.form.usageText', { max: MAX_DAYS })}
                checked={values.usageOn}
                onCheckedChange={(checked) => set('usageOn', checked)}
                disabled={readOnly}
              >
                <RadixTextField.Root
                  className={styles.days}
                  type="number"
                  inputMode="numeric"
                  min={1}
                  value={values.usage}
                  placeholder={t('cleanup.form.daysPlaceholder')}
                  aria-label={t('cleanup.form.usage')}
                  aria-invalid={problems.usage ? true : undefined}
                  disabled={readOnly}
                  onChange={(event) => set('usage', event.target.value)}
                />
                <ProblemText message={problems.usage} />
              </Criterion>
            )}
            {criteria.has('regex') && (
              <Criterion
                title={t('cleanup.form.regex')}
                description={t('cleanup.form.regexText')}
                checked={values.regexOn}
                onCheckedChange={(checked) => set('regexOn', checked)}
                disabled={readOnly}
              >
                <RadixTextField.Root
                  value={values.regex}
                  placeholder={t('cleanup.form.regexPlaceholder')}
                  aria-label={t('cleanup.form.regex')}
                  aria-invalid={problems.regex ? true : undefined}
                  disabled={readOnly}
                  onChange={(event) => set('regex', event.target.value)}
                />
                <ProblemText message={problems.regex} />
              </Criterion>
            )}
          </div>
          {problems.criteria && (
            <Text as="p" size="2" color="red" role="alert" className={styles.criteriaProblem}>
              {problems.criteria}
            </Text>
          )}
        </SettingsSection>
      )}
      {values.format !== '' && (
        <SettingsSection title={t('cleanup.preview.title')}>
          <PreviewSection values={values} criteria={criteria} valid={Object.keys(live).length === 0} />
        </SettingsSection>
      )}
      <FormError message={formError} />
      {policy && (
        <DeletePolicyDialog open={deleting} onOpenChange={setDeleting} policy={policy} onDelete={remove} />
      )}
    </SettingsForm>
  );
}

export default function CleanupPolicyPage() {
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
  const policy = useApi<Policy>(creating ? null : `${POLICIES}${path`/${name}`}`);
  const formats = useApi<FormatCriteria[]>(FORMATS);
  const all = t('cleanup.formats.all');
  const crumbs = [
    { label: t('cleanup.crumbs.settings'), to: '/settings' },
    { label: t('cleanup.list.title'), to: LIST_PAGE },
    { label: creating ? t('cleanup.form.createCrumb') : (name ?? '') },
  ];
  const loading = formats.isPending || (!creating && policy.isPending);
  const error = formats.error ?? (creating ? null : policy.error);
  return (
    <Page
      className={styles.page}
      title={creating ? t('cleanup.form.createTitle') : t('cleanup.form.editTitle', { name: name ?? '' })}
      description={
        !creating && policy.data
          ? t('cleanup.form.formatLine', { format: formatLabel(policy.data.format, all) })
          : undefined
      }
      breadcrumbs={crumbs}
    >
      {loading ? (
        <Flex justify="center" py="8">
          <Spinner size="3" />
        </Flex>
      ) : error ? (
        <ErrorState error={error} />
      ) : (
        <PolicyForm
          key={name ?? 'create'}
          policy={creating ? undefined : policy.data}
          formats={formats.data ?? []}
          leave={setTarget}
          leaving={target !== null}
        />
      )}
    </Page>
  );
}
