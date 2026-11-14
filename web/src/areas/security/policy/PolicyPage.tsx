import { Button, Card, Flex, Spinner, Text } from '@radix-ui/themes';
import type { FormEvent, ReactNode } from 'react';
import { useNavigate, useParams } from 'react-router';
import { api, path, useApi, useInvalidateApi } from '../../../api';
import { useCan } from '../../../auth/state';
import { type Translate, useT } from '../../../i18n';
import {
  describeError,
  ErrorState,
  type Errors,
  FormError,
  MultiSelectField,
  NumberField,
  Page,
  SelectField,
  TextAreaField,
  TextField,
  UnsavedChangesGuard,
  useConfirm,
  useForm,
  useToast,
} from '../../../ui';
import { POLICIES, TRUSTED_KEYS } from '../api';
import styles from '../shared/Form.module.css';
import { explainRefusal } from '../shared/messages';
import { ListEditor } from '../ssrf/ListEditor';
import {
  blocksEntered,
  type Choice,
  COOLDOWN_MAX_DAYS,
  COOLDOWN_MIN_DAYS,
  cooldownPatternProblem,
  cooldownProblems,
  EMPTY_FORM,
  formOf,
  LICENSE_LIST_OPTIONS,
  MALICIOUS_OPTIONS,
  MODE_OPTIONS,
  NONE,
  POLICIES_PAGE,
  type Policy,
  type PolicyForm,
  policyHref,
  requestOf,
  SEVERITY_OPTIONS,
  type TrustedKey,
  UNKNOWN_LICENSE_OPTIONS,
  UNSCANNED_OPTIONS,
} from './model';
import { POLICY_CREATE, POLICY_DELETE, POLICY_READ, POLICY_UPDATE } from './permissions';

interface RuleBlockProps {
  title: string;
  text: string;
  children: ReactNode;
}

function RuleBlock({ title, text, children }: RuleBlockProps) {
  return (
    <Card size="3">
      <div className={styles.card}>
        <Text as="div" size="3" weight="medium">
          {title}
        </Text>
        <Text as="p" size="2" color="gray">
          {text}
        </Text>
        {children}
      </div>
    </Card>
  );
}

function choices<V extends string>(
  t: Translate,
  options: Array<Choice<V>>,
): Array<{ value: string; label: string }> {
  return options.map((option) => ({ value: option.value, label: t(option.label) }));
}

function noneable(value: string): string {
  return value === '' ? NONE : value;
}

function unnoned(value: string): string {
  return value === NONE ? '' : value;
}

function problemsOf(t: Translate, values: PolicyForm, creating: boolean): Errors<PolicyForm> {
  const problems: Errors<PolicyForm> = {};
  if (creating && values.name.trim() === '') {
    problems.name = t('security.form.required');
  }
  const cooldown = cooldownProblems(values);
  if (cooldown.minAgeDays) {
    problems.minAgeDays = t(cooldown.minAgeDays, { min: COOLDOWN_MIN_DAYS, max: COOLDOWN_MAX_DAYS });
  }
  if (cooldown.exclude) {
    problems.exclude = t(cooldown.exclude);
  }
  return problems;
}

function PolicyEditor({ saved }: { saved: Policy | null }) {
  const t = useT();
  const toast = useToast();
  const confirm = useConfirm();
  const navigate = useNavigate();
  const invalidate = useInvalidateApi();
  const creating = saved === null;
  const savedForm = saved === null ? EMPTY_FORM : formOf(saved);
  const canWrite = useCan(creating ? POLICY_CREATE : POLICY_UPDATE);
  const canDelete = useCan(POLICY_DELETE);
  const keys = useApi<TrustedKey[]>(TRUSTED_KEYS);
  const form = useForm<PolicyForm>({
    initial: savedForm,
    validate: (values) => problemsOf(t, values, creating),
    onSubmit: (values) =>
      explainRefusal(t, async () => {
        if (creating) {
          await api.post(POLICIES, requestOf(values, true));
          toast.success(t('security.policy.created', { name: values.name }));
          await invalidate(POLICIES);
          navigate(policyHref(values.name), { replace: true });
          return;
        }
        await api.put(`${POLICIES}${path`/${values.name}`}`, requestOf(values, false));
        toast.success(t('security.policy.updated', { name: values.name }));
        await invalidate(POLICIES);
      }),
  });
  const { values } = form;
  const keyOptions = (keys.data ?? []).map((key) => ({ value: key.name, label: key.name }));
  const submit = async (event: FormEvent) => {
    event.preventDefault();
    const valid = Object.keys(problemsOf(t, values, creating)).length === 0;
    if (valid && blocksEntered(savedForm, values).length > 0) {
      const agreed = await confirm({
        title: t('security.policy.blockTitle'),
        description: t('security.policy.blockText'),
        confirmLabel: t('security.policy.blockConfirm'),
        danger: true,
      });
      if (!agreed) {
        return;
      }
    }
    await form.submit();
  };
  const remove = async () => {
    if (saved === null) {
      return;
    }
    const agreed = await confirm({
      title: t('security.policy.deleteTitle', { name: saved.name }),
      description: t('security.policy.deleteText'),
      confirmLabel: t('security.form.delete'),
      danger: true,
    });
    if (!agreed) {
      return;
    }
    try {
      await api.delete(`${POLICIES}${path`/${saved.name}`}`);
      toast.success(t('security.policy.deleted', { name: saved.name }));
      await invalidate(POLICIES);
      navigate(POLICIES_PAGE);
    } catch (error) {
      toast.error(t('security.policy.deleteRefused'), describeError(t, error));
    }
  };
  const disabled = !canWrite;
  return (
    <form className={styles.form} onSubmit={submit} noValidate>
      <Card size="3">
        <div className={styles.card}>
          <TextField
            label={t('security.policy.name')}
            {...form.field('name')}
            required
            disabled={disabled || !creating}
            hint={t('security.policy.nameHint')}
          />
          <TextAreaField label={t('security.policy.notes')} {...form.field('notes')} disabled={disabled} />
        </div>
      </Card>
      <RuleBlock title={t('security.policy.vulnerabilities')} text={t('security.policy.vulnerabilitiesText')}>
        <SelectField
          label={t('security.policy.mode')}
          {...form.field('vulnerabilityMode')}
          options={choices(t, MODE_OPTIONS)}
          disabled={disabled}
        />
        <SelectField
          label={t('security.policy.severity')}
          name="severity"
          value={noneable(values.severity)}
          onChange={(value) => form.set('severity', unnoned(value))}
          options={[{ value: NONE, label: t('security.policy.none') }, ...choices(t, SEVERITY_OPTIONS)]}
          hint={t('security.policy.severityHint')}
          disabled={disabled}
        />
        <SelectField
          label={t('security.policy.malicious')}
          {...form.field('malicious')}
          options={choices(t, MALICIOUS_OPTIONS)}
          disabled={disabled}
        />
        <SelectField
          label={t('security.policy.unscanned')}
          {...form.field('unscanned')}
          options={choices(t, UNSCANNED_OPTIONS)}
          hint={t('security.policy.unscannedHint')}
          disabled={disabled}
        />
        <SelectField
          label={t('security.policy.notify')}
          name="notify"
          value={noneable(values.notify)}
          onChange={(value) => form.set('notify', unnoned(value))}
          options={[{ value: NONE, label: t('security.policy.none') }, ...choices(t, SEVERITY_OPTIONS)]}
          hint={t('security.policy.notifyHint')}
          disabled={disabled}
        />
      </RuleBlock>
      <RuleBlock title={t('security.policy.licenses')} text={t('security.policy.licensesText')}>
        <SelectField
          label={t('security.policy.mode')}
          {...form.field('licenseMode')}
          options={choices(t, MODE_OPTIONS)}
          disabled={disabled}
        />
        <SelectField
          label={t('security.policy.licenseList')}
          name="licenseList"
          value={noneable(values.licenseList)}
          onChange={(value) => form.set('licenseList', unnoned(value))}
          options={[{ value: NONE, label: t('security.policy.none') }, ...choices(t, LICENSE_LIST_OPTIONS)]}
          disabled={disabled}
        />
        <TextAreaField
          label={t('security.policy.licenseIds')}
          {...form.field('licenseIds')}
          hint={t('security.policy.licenseIdsHint')}
          monospace
          disabled={disabled}
        />
        <SelectField
          label={t('security.policy.unknownLicense')}
          {...form.field('unknownLicense')}
          options={choices(t, UNKNOWN_LICENSE_OPTIONS)}
          disabled={disabled}
        />
      </RuleBlock>
      <RuleBlock title={t('security.policy.signatures')} text={t('security.policy.signaturesText')}>
        <SelectField
          label={t('security.policy.mode')}
          {...form.field('signatureMode')}
          options={choices(t, MODE_OPTIONS)}
          disabled={disabled}
        />
        <MultiSelectField
          label={t('security.policy.signatureKeys')}
          {...form.field('signatureKeys')}
          options={keyOptions}
          hint={t('security.policy.signatureKeysHint')}
          disabled={disabled}
        />
      </RuleBlock>
      <RuleBlock title={t('security.policy.cooldown')} text={t('security.policy.cooldownText')}>
        <SelectField
          label={t('security.policy.mode')}
          {...form.field('cooldownMode')}
          options={choices(t, MODE_OPTIONS)}
          disabled={disabled}
        />
        {values.cooldownMode !== 'off' && (
          <NumberField
            label={t('security.policy.cooldownMinAge')}
            {...form.field('minAgeDays')}
            min={COOLDOWN_MIN_DAYS}
            max={COOLDOWN_MAX_DAYS}
            step={1}
            required
            hint={t('security.policy.cooldownMinAgeHint', { min: COOLDOWN_MIN_DAYS, max: COOLDOWN_MAX_DAYS })}
            disabled={disabled}
          />
        )}
        <ListEditor
          label={t('security.policy.cooldownExclude')}
          hint={t('security.policy.cooldownExcludeHint')}
          placeholder="@globalart/*"
          values={values.exclude}
          onChange={(exclude) => form.set('exclude', exclude)}
          check={(value) => {
            const problem = cooldownPatternProblem(value, values.exclude);
            return problem === null ? undefined : t(problem);
          }}
          error={form.errors.exclude}
          disabled={disabled}
          emptyText={t('security.policy.cooldownExcludeEmpty')}
        />
      </RuleBlock>
      <FormError message={form.formError} />
      <div className={styles.actions}>
        <Button type="submit" loading={form.submitting} disabled={disabled || !form.dirty}>
          {t('security.form.save')}
        </Button>
        <Button type="button" variant="soft" color="gray" disabled={!form.dirty} onClick={() => form.reset()}>
          {t('security.form.discard')}
        </Button>
        {saved !== null && canDelete && (
          <Button type="button" variant="soft" color="red" onClick={() => void remove()}>
            {t('security.form.delete')}
          </Button>
        )}
      </div>
      <UnsavedChangesGuard when={form.dirty} />
    </form>
  );
}

export default function PolicyPage() {
  const t = useT();
  const { name } = useParams();
  const canRead = useCan(POLICY_READ);
  const policy = useApi<Policy>(name === undefined || !canRead ? null : `${POLICIES}${path`/${name}`}`);
  const title = name === undefined ? t('security.policy.createTitle') : (name ?? '');
  return (
    <Page
      title={title}
      description={t('security.policy.intro')}
      breadcrumbs={[
        { label: t('security.crumbs.settings'), to: '/settings' },
        { label: t('security.policy.title'), to: POLICIES_PAGE },
        { label: title },
      ]}
    >
      {name === undefined ? (
        <PolicyEditor saved={null} />
      ) : policy.isPending ? (
        <Flex justify="center" py="8">
          <Spinner size="3" />
        </Flex>
      ) : policy.error ? (
        <ErrorState error={policy.error} onRetry={() => void policy.refetch()} />
      ) : policy.data ? (
        <PolicyEditor saved={policy.data} />
      ) : null}
    </Page>
  );
}
