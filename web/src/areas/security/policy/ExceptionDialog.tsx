import { Button, Dialog, Flex, TextField as RadixTextField } from '@radix-ui/themes';
import { api, useApi, useInvalidateApi } from '../../../api';
import { useCan, useSession } from '../../../auth/state';
import { type Translate, useT } from '../../../i18n';
import {
  type Errors,
  FieldShell,
  type Form,
  FormError,
  SelectField,
  TextAreaField,
  TextField,
  useForm,
  useToast,
} from '../../../ui';
import { POLICIES, POLICY_EXCEPTIONS } from '../api';
import styles from '../shared/Form.module.css';
import { explainRefusal } from '../shared/messages';
import {
  customExpiryProblem,
  EMPTY_EXCEPTION_FORM,
  EXPIRY_OPTIONS,
  type ExceptionForm,
  type Expiry,
  RULE_OPTIONS,
  type Rule,
  requestOf,
  SUBJECT_OPTIONS,
  type Subject,
  subjectsOf,
  targetFilled,
} from './exceptions';
import { type Choice, NONE, type Policy } from './model';
import { POLICY_READ } from './permissions';

interface ExceptionDialogProps {
  open: boolean;
  onOpenChange(open: boolean): void;
}

function choices<V extends string>(
  t: Translate,
  options: Array<Choice<V>>,
): Array<{ value: string; label: string }> {
  return options.map((option) => ({ value: option.value, label: t(option.label) }));
}

function problemsOf(t: Translate, values: ExceptionForm, now: number): Errors<ExceptionForm> {
  const problems: Errors<ExceptionForm> = {};
  if (values.reason.trim() === '') {
    problems.reason = t('security.form.required');
  }
  if (!targetFilled(values)) {
    problems[values.subject === 'component' ? 'format' : values.subject] = t(
      'security.exceptions.targetRequired',
    );
  }
  const expiry = customExpiryProblem(values, now);
  if (expiry !== null) {
    problems.customExpiry = t(expiry);
  }
  return problems;
}

interface ExceptionFieldsProps {
  form: Form<ExceptionForm>;
}

function ExceptionFields({ form }: ExceptionFieldsProps) {
  const t = useT();
  const { state } = useSession();
  const canReadPolicies = useCan(POLICY_READ);
  const policies = useApi<Policy[]>(canReadPolicies ? POLICIES : null);
  const { values } = form;
  const policyOptions = [
    { value: NONE, label: t('security.exceptions.allPolicies') },
    ...(policies.data ?? []).map((policy) => ({ value: policy.name, label: policy.name })),
  ];
  const formatOptions = [
    { value: NONE, label: t('security.exceptions.anyFormat') },
    ...state.formats.map((format) => ({ value: format.id, label: format.id })),
  ];
  const subjects = subjectsOf(values.rule);
  const setRule = (rule: Rule) => {
    form.set('rule', rule);
    if (!subjectsOf(rule).includes(values.subject)) {
      form.set('subject', subjectsOf(rule)[0] as Subject);
    }
  };
  return (
    <>
      <SelectField
        label={t('security.exceptions.rule')}
        name="rule"
        value={values.rule}
        onChange={(value) => setRule(value as Rule)}
        options={choices(t, RULE_OPTIONS)}
      />
      <SelectField
        label={t('security.exceptions.policy')}
        name="policy"
        value={values.policy === '' ? NONE : values.policy}
        onChange={(value) => form.set('policy', value === NONE ? '' : value)}
        options={policyOptions}
      />
      <SelectField
        label={t('security.exceptions.subject')}
        name="subject"
        value={values.subject}
        onChange={(value) => form.set('subject', value as Subject)}
        options={choices(
          t,
          SUBJECT_OPTIONS.filter((option) => subjects.includes(option.value)),
        )}
      />
      {values.subject === 'advisory' && (
        <TextField
          label={t('security.exceptions.advisory')}
          {...form.field('advisory')}
          hint={t('security.exceptions.advisoryHint')}
          required
        />
      )}
      {values.subject === 'license' && (
        <TextField label={t('security.exceptions.license')} {...form.field('license')} required />
      )}
      {values.subject === 'digest' && (
        <TextField
          label={t('security.exceptions.digest')}
          {...form.field('digest')}
          hint={t('security.exceptions.digestHint')}
          required
        />
      )}
      {values.subject === 'component' && (
        <>
          <SelectField
            label={t('security.exceptions.format')}
            name="format"
            value={values.format === '' ? NONE : values.format}
            onChange={(value) => form.set('format', value === NONE ? '' : value)}
            options={formatOptions}
            error={form.errors.format}
          />
          <TextField label={t('security.exceptions.namespace')} {...form.field('namespace')} />
          <TextField label={t('security.exceptions.componentName')} {...form.field('name')} />
          <TextField label={t('security.exceptions.version')} {...form.field('version')} />
        </>
      )}
      <TextAreaField label={t('security.exceptions.reason')} {...form.field('reason')} required />
      <SelectField
        label={t('security.exceptions.expiry')}
        name="expiry"
        value={values.expiry}
        onChange={(value) => form.set('expiry', value as Expiry)}
        options={choices(t, EXPIRY_OPTIONS)}
        hint={t('security.exceptions.expiryHint')}
      />
      {values.expiry === 'custom' && (
        <FieldShell
          id="exception-custom-expiry"
          label={t('security.exceptions.customExpiry')}
          error={form.errors.customExpiry}
          required
        >
          <RadixTextField.Root
            id="exception-custom-expiry"
            type="datetime-local"
            value={values.customExpiry}
            onChange={(event) => form.set('customExpiry', event.target.value)}
            aria-invalid={form.errors.customExpiry ? true : undefined}
          />
        </FieldShell>
      )}
    </>
  );
}

function ExceptionDialogForm({ onDone }: { onDone(): void }) {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const form = useForm<ExceptionForm>({
    initial: EMPTY_EXCEPTION_FORM,
    validate: (values) => problemsOf(t, values, Date.now()),
    onSubmit: (values) =>
      explainRefusal(t, async () => {
        await api.post(POLICY_EXCEPTIONS, requestOf(values, Date.now()));
        toast.success(t('security.exceptions.created'));
        await invalidate(POLICY_EXCEPTIONS);
        onDone();
      }),
  });
  return (
    <form onSubmit={form.submit} noValidate>
      <Flex direction="column" gap="4">
        <div className={styles.card}>
          <ExceptionFields form={form} />
        </div>
        <FormError message={form.formError} />
        <Flex gap="3" justify="end" wrap="wrap">
          <Dialog.Close>
            <Button type="button" variant="soft" color="gray">
              {t('security.form.cancel')}
            </Button>
          </Dialog.Close>
          <Button type="submit" loading={form.submitting}>
            {t('security.exceptions.add')}
          </Button>
        </Flex>
      </Flex>
    </form>
  );
}

export function ExceptionDialog({ open, onOpenChange }: ExceptionDialogProps) {
  const t = useT();
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Content maxWidth="520px">
        <Dialog.Title>{t('security.exceptions.createTitle')}</Dialog.Title>
        <Dialog.Description size="2" color="gray">
          {t('security.exceptions.intro')}
        </Dialog.Description>
        {open && <ExceptionDialogForm onDone={() => onOpenChange(false)} />}
      </Dialog.Content>
    </Dialog.Root>
  );
}
