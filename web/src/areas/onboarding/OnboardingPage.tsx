import {
  Button,
  Callout,
  Card,
  CheckboxCards,
  Flex,
  Heading,
  RadioCards,
  Spinner,
  Text,
} from '@radix-ui/themes';
import { useQueryClient } from '@tanstack/react-query';
import { CircleCheck, CircleMinus, CircleX, Globe, Info, Lock } from 'lucide-react';
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
import { createPath, REPOSITORIES } from '../repositories/api';
import { useBlobStoreChoices } from '../repositories/components/useBlobStoreChoices';
import styles from './Onboarding.module.css';
import { availablePresets, plan } from './presets';
import { OFFERED_KEY, sessionMarker } from './watcher';

const ACCOUNT_PASSWORD = `${INTERNAL}/ui/user/password`;
const USERS = `${V1}/security/users`;
const ANONYMOUS = `${V1}/security/anonymous`;
const ADMIN = 'admin';
const MIN_PASSWORD = 8;

type Step = 'password' | 'anonymous' | 'repositories' | 'done';
const STEPS: Step[] = ['password', 'anonymous', 'repositories', 'done'];

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
    repositories: t('onboarding.steps.repositories'),
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

interface Outcome {
  name: string;
  status: 'created' | 'exists' | 'failed';
  message?: string;
}

function OutcomeLine({ outcome }: { outcome: Outcome }) {
  const t = useT();
  const Icon =
    outcome.status === 'created' ? CircleCheck : outcome.status === 'exists' ? CircleMinus : CircleX;
  const color =
    outcome.status === 'created'
      ? 'var(--green-9)'
      : outcome.status === 'exists'
        ? 'var(--gray-9)'
        : 'var(--red-9)';
  const text =
    outcome.status === 'created'
      ? t('onboarding.repositories.created')
      : outcome.status === 'exists'
        ? t('onboarding.repositories.exists')
        : t('onboarding.repositories.failed', { message: outcome.message ?? '' });
  return (
    <li className={styles.outcome}>
      <Icon size={16} color={color} aria-hidden="true" />
      <Text size="2" className={styles.repositoryName}>
        {outcome.name}
      </Text>
      <Text size="2" color={outcome.status === 'failed' ? 'red' : 'gray'}>
        {text}
      </Text>
    </li>
  );
}

function RepositoriesStep({ onDone }: { onDone(): void }) {
  const t = useT();
  const client = useQueryClient();
  const { state } = useSession();
  const presets = availablePresets(state.formats);
  const stores = useBlobStoreChoices();
  const existing = useApi<{ name: string }[]>(REPOSITORIES);
  const [chosen, setChosen] = useState<string[]>([]);
  const [busy, setBusy] = useState(false);
  const [outcomes, setOutcomes] = useState<Outcome[] | null>(null);
  const blobStore =
    stores.choices?.find((store) => store.name === 'default')?.name ?? stores.choices?.[0]?.name ?? 'default';

  const create = async () => {
    setBusy(true);
    const known = new Set((existing.data ?? []).map((repository) => repository.name.toLowerCase()));
    const results: Outcome[] = [];
    const planned = plan(
      presets.filter((preset) => chosen.includes(preset.id)),
      state.formats,
      blobStore,
    );
    for (const repository of planned) {
      if (known.has(repository.name.toLowerCase())) {
        results.push({ name: repository.name, status: 'exists' });
      } else {
        try {
          await api.post(createPath(repository.apiFormat, 'hosted'), repository.body);
          results.push({ name: repository.name, status: 'created' });
        } catch (failure) {
          results.push({
            name: repository.name,
            status: 'failed',
            message: failure instanceof Error ? failure.message : String(failure),
          });
        }
      }
      setOutcomes([...results]);
    }
    await client.invalidateQueries();
    setBusy(false);
  };

  return (
    <div className={styles.body}>
      <Flex direction="column" gap="1">
        <Heading as="h2" size="5" id="onboarding-repositories">
          {t('onboarding.repositories.title')}
        </Heading>
        <Text size="2" color="gray">
          {t('onboarding.repositories.intro')}
        </Text>
      </Flex>
      {outcomes === null ? (
        existing.isPending || !stores.ready ? (
          <Flex justify="center" py="4">
            <Spinner />
          </Flex>
        ) : existing.error ? (
          <ErrorState error={existing.error} onRetry={() => void existing.refetch()} />
        ) : (
          <CheckboxCards.Root
            value={chosen}
            onValueChange={setChosen}
            columns={{ initial: '1', sm: '2' }}
            aria-labelledby="onboarding-repositories"
          >
            {presets.map((preset) => (
              <CheckboxCards.Item key={preset.id} value={preset.id}>
                <Flex direction="column" gap="1" width="100%">
                  <Text weight="bold">{preset.label}</Text>
                  <ul className={styles.repositories}>
                    {preset.repositories.map((repository) => (
                      <li key={repository.name}>
                        <Text size="1" color="gray" className={styles.repositoryName}>
                          {repository.name}
                        </Text>
                      </li>
                    ))}
                  </ul>
                </Flex>
              </CheckboxCards.Item>
            ))}
          </CheckboxCards.Root>
        )
      ) : (
        <ul className={styles.outcomes} aria-label={t('onboarding.repositories.results')}>
          {outcomes.map((outcome) => (
            <OutcomeLine key={outcome.name} outcome={outcome} />
          ))}
        </ul>
      )}
      <div className={styles.actions}>
        {outcomes === null ? (
          <>
            <Button type="button" variant="ghost" color="gray" onClick={onDone}>
              {t('onboarding.actions.skip')}
            </Button>
            <Button
              type="button"
              loading={busy}
              disabled={chosen.length === 0 || existing.isPending || !stores.ready}
              onClick={() => void create()}
            >
              {t('onboarding.repositories.submit')}
            </Button>
          </>
        ) : (
          <Button type="button" loading={busy} onClick={onDone} className={styles.alone}>
            {t('onboarding.repositories.continue')}
          </Button>
        )}
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
          {t('onboarding.page.lead')}
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
        {step === 'anonymous' && <AnonymousStep onDone={() => setStep('repositories')} onSkip={leave} />}
        {step === 'repositories' && <RepositoriesStep onDone={() => setStep('done')} />}
        {step === 'done' && <DoneStep onFinish={leave} />}
      </Card>
      {step === 'password' && (
        <Text as="p" size="1" color="gray" mt="3">
          {t('onboarding.page.later')}
        </Text>
      )}
    </div>
  );
}
