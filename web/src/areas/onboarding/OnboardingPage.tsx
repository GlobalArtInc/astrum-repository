import { Button, Callout, Card, Flex, Heading, RadioCards, Spinner, Text } from '@radix-ui/themes';
import { useQueryClient } from '@tanstack/react-query';
import { CircleCheck, Globe, Info, Lock } from 'lucide-react';
import { useEffect, useState } from 'react';
import { Navigate, useNavigate, useSearchParams } from 'react-router';
import { ApiError, api, INTERNAL, path, useApi, V1 } from '../../api';
import { safeReturnTo } from '../../auth/returnTo';
import { STATE_KEY, useSession } from '../../auth/state';
import { type Translate, useT } from '../../i18n';
import {
  ErrorState,
  FormError,
  PasswordField,
  useDocumentTitle,
  useForm,
  useToast,
  writeStored,
} from '../../ui';
import styles from './Onboarding.module.css';
import { OFFERED_KEY, sessionMarker } from './watcher';

const ACCOUNT_PASSWORD = `${INTERNAL}/ui/user/password`;
const USERS = `${V1}/security/users`;
const ANONYMOUS = `${V1}/security/anonymous`;
const ADMIN = 'admin';
const MIN_PASSWORD = 8;

type Step = 'password' | 'anonymous' | 'done';
const STEPS: Step[] = ['password', 'anonymous', 'done'];

interface Anonymous {
  enabled: boolean;
  userId: string;
  realmName: string;
}

interface PasswordValues {
  currentPassword: string;
  newPassword: string;
  confirm: string;
}

export function validateOnboardingPassword(
  t: Translate,
  values: PasswordValues,
  needsCurrent: boolean,
): Partial<Record<keyof PasswordValues, string>> {
  const errors: Partial<Record<keyof PasswordValues, string>> = {};
  if (needsCurrent && values.currentPassword === '') {
    errors.currentPassword = t('onboarding.password.required');
  }
  if (values.newPassword === '') {
    errors.newPassword = t('onboarding.password.required');
  } else if ([...values.newPassword].length < MIN_PASSWORD) {
    errors.newPassword = t('onboarding.password.rule', { count: MIN_PASSWORD });
  } else if (needsCurrent && values.newPassword === values.currentPassword) {
    errors.newPassword = t('onboarding.password.same');
  }
  if (values.confirm !== values.newPassword) {
    errors.confirm = t('onboarding.password.mismatch');
  }
  return errors;
}

function explain(t: Translate, error: unknown): unknown {
  if (error instanceof ApiError) {
    if (error.status === 429) {
      return new Error(t('onboarding.password.tooMany'));
    }
    if (error.status === 400 && error.fieldErrors.some((problem) => problem.id === 'currentPassword')) {
      return new ApiError(400, t('onboarding.password.wrong'), {
        fieldErrors: [{ id: 'currentPassword', message: t('onboarding.password.wrong') }],
      });
    }
  }
  return error;
}

function StepList({ current }: { current: Step }) {
  const t = useT();
  const labels: Record<Step, string> = {
    password: t('onboarding.steps.password'),
    anonymous: t('onboarding.steps.anonymous'),
    done: t('onboarding.steps.done'),
  };
  const index = STEPS.indexOf(current);
  return (
    <ol className={styles.steps} aria-label={t('onboarding.steps.label')}>
      {STEPS.map((step, position) => (
        <li
          key={step}
          className={styles.step}
          data-state={position < index ? 'done' : position === index ? 'current' : 'next'}
          aria-current={position === index ? 'step' : undefined}
        >
          <span className={styles.number} aria-hidden="true">
            {position < index ? <CircleCheck size={16} /> : position + 1}
          </span>
          <span>{labels[step]}</span>
        </li>
      ))}
    </ol>
  );
}

function PasswordStep({
  passwordFile,
  selfIsAdmin,
  onDone,
  onSkip,
}: {
  passwordFile: string | null;
  selfIsAdmin: boolean;
  onDone(): void;
  onSkip(): void;
}) {
  const t = useT();
  const toast = useToast();
  const form = useForm<PasswordValues>({
    initial: { currentPassword: '', newPassword: '', confirm: '' },
    validate: (values) => validateOnboardingPassword(t, values, selfIsAdmin),
    onSubmit: async (values) => {
      try {
        if (selfIsAdmin) {
          await api.put(ACCOUNT_PASSWORD, {
            currentPassword: values.currentPassword,
            newPassword: values.newPassword,
          });
        } else {
          await api.put(`${USERS}${path`/${ADMIN}/change-password`}`, values.newPassword, {
            headers: { 'Content-Type': 'text/plain; charset=utf-8' },
          });
        }
      } catch (error) {
        throw explain(t, error);
      }
      toast.success(t('onboarding.password.changed'));
      onDone();
    },
  });
  return (
    <form className={styles.body} onSubmit={form.submit} noValidate>
      <Flex direction="column" gap="1">
        <Heading as="h2" size="5">
          {t('onboarding.password.title')}
        </Heading>
        <Text size="2" color="gray">
          {selfIsAdmin ? t('onboarding.password.intro') : t('onboarding.password.introOther')}
        </Text>
      </Flex>
      {passwordFile && (
        <Callout.Root color="gray" variant="surface">
          <Callout.Icon>
            <Info size={16} />
          </Callout.Icon>
          <Callout.Text>
            {t.rich('onboarding.password.file', {
              path: (
                <code key="path" className={styles.path}>
                  {passwordFile}
                </code>
              ),
            })}
          </Callout.Text>
        </Callout.Root>
      )}
      <input type="text" name="username" autoComplete="username" value={ADMIN} hidden readOnly />
      {selfIsAdmin && (
        <PasswordField
          label={t('onboarding.password.current')}
          hint={t('onboarding.password.currentHint')}
          {...form.field('currentPassword')}
          required
          autoFocus
          autoComplete="current-password"
        />
      )}
      <PasswordField
        label={t('onboarding.password.new')}
        hint={t('onboarding.password.hint', { count: MIN_PASSWORD })}
        {...form.field('newPassword')}
        required
        autoFocus={!selfIsAdmin}
        autoComplete="new-password"
      />
      <PasswordField
        label={t('onboarding.password.confirm')}
        {...form.field('confirm')}
        required
        autoComplete="new-password"
      />
      <FormError message={form.formError} />
      <div className={styles.actions}>
        <Button type="button" variant="ghost" color="gray" onClick={onSkip}>
          {t('onboarding.actions.skip')}
        </Button>
        <Button type="submit" loading={form.submitting}>
          {t('onboarding.password.submit')}
        </Button>
      </div>
    </form>
  );
}

function AnonymousStep({ onDone, onSkip }: { onDone(): void; onSkip(): void }) {
  const t = useT();
  const client = useQueryClient();
  const settings = useApi<Anonymous>(ANONYMOUS);
  const [choice, setChoice] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const value = choice ?? (settings.data ? (settings.data.enabled ? 'on' : 'off') : 'off');

  const save = async () => {
    if (!settings.data) {
      return;
    }
    setBusy(true);
    setError(null);
    try {
      await api.put(ANONYMOUS, { ...settings.data, enabled: value === 'on' });
      await client.invalidateQueries({ queryKey: STATE_KEY });
      onDone();
    } catch (failure) {
      setError(failure);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className={styles.body}>
      <Flex direction="column" gap="1">
        <Heading as="h2" size="5" id="onboarding-anonymous">
          {t('onboarding.anonymous.title')}
        </Heading>
        <Text size="2" color="gray">
          {t('onboarding.anonymous.intro')}
        </Text>
      </Flex>
      {settings.isPending ? (
        <Flex justify="center" py="4">
          <Spinner />
        </Flex>
      ) : settings.error ? (
        <ErrorState error={settings.error} onRetry={() => void settings.refetch()} />
      ) : (
        <RadioCards.Root
          value={value}
          onValueChange={setChoice}
          columns={{ initial: '1', sm: '2' }}
          aria-labelledby="onboarding-anonymous"
        >
          <RadioCards.Item value="on">
            <Flex direction="column" gap="1" width="100%">
              <Flex gap="2" align="center">
                <Globe size={16} aria-hidden="true" />
                <Text weight="bold">{t('onboarding.anonymous.on')}</Text>
              </Flex>
              <Text size="2" color="gray">
                {t('onboarding.anonymous.onText')}
              </Text>
            </Flex>
          </RadioCards.Item>
          <RadioCards.Item value="off">
            <Flex direction="column" gap="1" width="100%">
              <Flex gap="2" align="center">
                <Lock size={16} aria-hidden="true" />
                <Text weight="bold">{t('onboarding.anonymous.off')}</Text>
              </Flex>
              <Text size="2" color="gray">
                {t('onboarding.anonymous.offText')}
              </Text>
            </Flex>
          </RadioCards.Item>
        </RadioCards.Root>
      )}
      {error !== null && <FormError message={error instanceof Error ? error.message : String(error)} />}
      <div className={styles.actions}>
        <Button type="button" variant="ghost" color="gray" onClick={onSkip}>
          {t('onboarding.actions.skip')}
        </Button>
        <Button type="button" loading={busy} disabled={!settings.data} onClick={() => void save()}>
          {t('onboarding.anonymous.submit')}
        </Button>
      </div>
    </div>
  );
}

function DoneStep({ onFinish }: { onFinish(): void }) {
  const t = useT();
  return (
    <div className={styles.body}>
      <Flex direction="column" align="center" gap="3" py="4" style={{ textAlign: 'center' }}>
        <CircleCheck size={40} color="var(--green-9)" aria-hidden="true" />
        <Heading as="h2" size="5">
          {t('onboarding.done.title')}
        </Heading>
        <Text size="2" color="gray">
          {t('onboarding.done.text')}
        </Text>
        <Button size="3" onClick={onFinish} autoFocus>
          {t('onboarding.done.finish')}
        </Button>
      </Flex>
    </div>
  );
}

export default function OnboardingPage() {
  const t = useT();
  const navigate = useNavigate();
  const [params] = useSearchParams();
  const { state, user } = useSession();
  const [started] = useState(() => state.adminPasswordFile !== null);
  const [step, setStep] = useState<Step>('password');
  const returnTo = safeReturnTo(params.get('returnTo'));
  useDocumentTitle(t('onboarding.page.title'));

  useEffect(() => {
    if (user) {
      writeStored(OFFERED_KEY, sessionMarker(user.userId));
    }
  }, [user]);

  if (!started) {
    return <Navigate to={returnTo} replace />;
  }

  const leave = () => navigate(returnTo, { replace: true });
  const selfIsAdmin = user?.userId === ADMIN && user.source === 'default';

  return (
    <div className={styles.page}>
      <Flex direction="column" gap="1" mb="4">
        <Heading as="h1" size="7">
          {t('onboarding.page.title')}
        </Heading>
        <Text size="2" color="gray">
          {t('onboarding.page.intro')}
        </Text>
      </Flex>
      <StepList current={step} />
      <Card size="3" className={styles.card}>
        {step === 'password' && (
          <PasswordStep
            passwordFile={state.adminPasswordFile}
            selfIsAdmin={selfIsAdmin}
            onDone={() => setStep('anonymous')}
            onSkip={leave}
          />
        )}
        {step === 'anonymous' && <AnonymousStep onDone={() => setStep('done')} onSkip={leave} />}
        {step === 'done' && <DoneStep onFinish={leave} />}
      </Card>
      {step !== 'done' && (
        <Text as="p" size="1" color="gray" mt="3">
          {t('onboarding.page.later')}
        </Text>
      )}
    </div>
  );
}
