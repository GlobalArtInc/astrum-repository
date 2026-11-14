import { Badge, Button, Callout, Card, Code, Dialog, Flex, TextField } from '@radix-ui/themes';
import { ShieldCheck, TriangleAlert } from 'lucide-react';
import { type FormEvent, useId, useRef, useState } from 'react';
import { useNavigate } from 'react-router';
import { ApiError, useApi } from '../../api';
import { type Translate, useT } from '../../i18n';
import { CopyButton, DateTime, describeError, ErrorState, useDocumentTitle, useToast } from '../../ui';
import { PasskeysCard } from './PasskeySection';
import { RecoveryCodesContent } from './RecoveryCodesContent';
import styles from './Security.module.css';
import {
  beginTotp,
  type ConfirmBody,
  confirmTotp,
  fieldMessage,
  LOW_RECOVERY_CODES,
  qrDataUri,
  type RecoveryCodes,
  regenerateRecoveryCodes,
  removeTotp,
  type TotpConfirmation,
  type TotpEnrollment,
  TWO_FACTOR,
  type TwoFactorStatus,
} from './twoFactor';

type FlowMode = 'setUp' | 'regenerate' | 'remove';
type FlowStep = 'password' | 'enroll' | 'codes';

interface FlowText {
  title: string;
  text: string;
  action: string;
}

interface SecurityDialogProps {
  mode: FlowMode;
  onClose(): void;
  onChanged(): Promise<void>;
}

interface AuthenticatorCardProps {
  status: TwoFactorStatus;
  onStart(mode: FlowMode): void;
}

function groupSecret(secret: string): string {
  return secret.match(/.{1,4}/g)?.join(' ') ?? secret;
}

function flowText(mode: FlowMode, t: Translate): FlowText {
  if (mode === 'setUp') {
    return {
      title: t('user.security.setUpTitle'),
      text: t('user.security.setUpText'),
      action: t('user.security.continue'),
    };
  }
  if (mode === 'regenerate') {
    return {
      title: t('user.security.newCodes'),
      text: t('user.security.regenerateText'),
      action: t('user.security.newCodes'),
    };
  }
  return {
    title: t('user.security.removeTitle'),
    text: t('user.security.removeText'),
    action: t('user.security.remove'),
  };
}

function SecurityDialog({ mode, onClose, onChanged }: SecurityDialogProps) {
  const t = useT();
  const toast = useToast();
  const passwordId = useId();
  const codeId = useId();
  const pending = useRef(false);
  const [step, setStep] = useState<FlowStep>('password');
  const [busy, setBusy] = useState(false);
  const [password, setPassword] = useState('');
  const [passwordError, setPasswordError] = useState<string | null>(null);
  const [enrollment, setEnrollment] = useState<TotpEnrollment | null>(null);
  const [code, setCode] = useState('');
  const [codeError, setCodeError] = useState<string | null>(null);
  const [codes, setCodes] = useState<string[]>([]);
  const [formError, setFormError] = useState<string | null>(null);
  const text = flowText(mode, t);

  const guarded = async (action: () => Promise<void>) => {
    if (pending.current) {
      return;
    }
    pending.current = true;
    setBusy(true);
    setFormError(null);
    try {
      await action();
    } finally {
      pending.current = false;
      setBusy(false);
    }
  };

  const reportPassword = (error: unknown) => {
    const message = fieldMessage(error, 'currentPassword');
    setPasswordError(message);
    setFormError(message === null ? describeError(t, error) : null);
  };

  const reportEnrollment = (error: unknown) => {
    const codeMessage = fieldMessage(error, 'code');
    if (codeMessage !== null) {
      setCodeError(codeMessage);
      return;
    }
    if (fieldMessage(error, 'ceremony') !== null) {
      setEnrollment(null);
      setCode('');
      setPassword('');
      setStep('password');
    }
    setFormError(describeError(t, error));
  };

  const submitPassword = (event: FormEvent) => {
    event.preventDefault();
    if (password === '') {
      return;
    }
    void guarded(async () => {
      setPasswordError(null);
      try {
        if (mode === 'setUp') {
          setEnrollment(await beginTotp(password));
          setStep('enroll');
        } else if (mode === 'regenerate') {
          const result: RecoveryCodes = await regenerateRecoveryCodes(password);
          setCodes(result.recoveryCodes);
          setStep('codes');
          await onChanged();
        } else {
          await removeTotp(password);
          await onChanged();
          toast.success(t('user.security.removed'));
          onClose();
        }
      } catch (error) {
        reportPassword(error);
      }
    });
  };

  const submitCode = (event: FormEvent) => {
    event.preventDefault();
    if (enrollment === null || code.trim() === '') {
      return;
    }
    void guarded(async () => {
      setCodeError(null);
      const body: ConfirmBody = {
        ceremony: enrollment.ceremony,
        code: code.trim(),
        currentPassword: password,
      };
      try {
        const result: TotpConfirmation = await confirmTotp(body);
        await onChanged();
        if (result.recoveryCodes) {
          setCodes(result.recoveryCodes);
          setStep('codes');
        } else {
          toast.success(t('user.security.enabled'));
          onClose();
        }
      } catch (error) {
        reportEnrollment(error);
      }
    });
  };

  return (
    <Dialog.Root
      open
      onOpenChange={(open) => {
        if (!open && !busy) {
          onClose();
        }
      }}
    >
      <Dialog.Content maxWidth="480px" className={styles.dialog}>
        {step === 'password' && (
          <form onSubmit={submitPassword} noValidate>
            <Dialog.Title className={styles.dialogTitle}>{text.title}</Dialog.Title>
            <Dialog.Description size="2" color="gray">
              {text.text}
            </Dialog.Description>
            {mode === 'remove' && (
              <Callout.Root color="amber" size="1" className={styles.notice}>
                <Callout.Icon>
                  <TriangleAlert size={16} aria-hidden="true" />
                </Callout.Icon>
                <Callout.Text>{t('user.security.removeWarning')}</Callout.Text>
              </Callout.Root>
            )}
            <label htmlFor={passwordId} className={styles.label}>
              {t('user.security.password')}
            </label>
            <TextField.Root
              id={passwordId}
              name="password"
              type="password"
              value={password}
              autoComplete="current-password"
              autoFocus
              aria-invalid={passwordError ? true : undefined}
              aria-describedby={passwordError ? `${passwordId}-error` : undefined}
              onChange={(event) => setPassword(event.target.value)}
            />
            {passwordError && (
              <p id={`${passwordId}-error`} className={styles.error} role="alert">
                {passwordError}
              </p>
            )}
            {formError && (
              <p className={styles.error} role="alert">
                {formError}
              </p>
            )}
            <Flex gap="3" justify="end" mt="4" wrap="wrap">
              <Dialog.Close>
                <Button type="button" variant="soft" color="gray" disabled={busy}>
                  {t('user.security.cancel')}
                </Button>
              </Dialog.Close>
              <Button type="submit" loading={busy} disabled={password === '' || busy}>
                {text.action}
              </Button>
            </Flex>
          </form>
        )}
        {step === 'enroll' && enrollment && (
          <form onSubmit={submitCode} noValidate>
            <Dialog.Title className={styles.dialogTitle}>{t('user.security.setUpTitle')}</Dialog.Title>
            <Dialog.Description size="2" color="gray">
              {t('user.security.enrollText')}
            </Dialog.Description>
            <img
              className={styles.qr}
              src={qrDataUri(enrollment.qrSvg)}
              alt={t('user.security.qrAlt')}
              width={192}
              height={192}
            />
            <div className={styles.label}>{t('user.security.secret')}</div>
            <div className={styles.secret}>
              <Code size="2" variant="soft" className={styles.secretCode}>
                {groupSecret(enrollment.secret)}
              </Code>
              <CopyButton text={enrollment.secret} label={t('user.security.copySecret')} />
            </div>
            <label htmlFor={codeId} className={styles.label}>
              {t('user.security.code')}
            </label>
            <TextField.Root
              id={codeId}
              name="code"
              value={code}
              inputMode="numeric"
              autoComplete="one-time-code"
              placeholder="123456"
              autoFocus
              aria-invalid={codeError ? true : undefined}
              aria-describedby={`${codeId}-hint`}
              onChange={(event) => setCode(event.target.value)}
            />
            <p id={`${codeId}-hint`} className={styles.hint}>
              {t('user.security.codeHint')}
            </p>
            {codeError && (
              <p className={styles.error} role="alert">
                {codeError}
              </p>
            )}
            {formError && (
              <p className={styles.error} role="alert">
                {formError}
              </p>
            )}
            <Flex gap="3" justify="end" mt="4" wrap="wrap">
              <Dialog.Close>
                <Button type="button" variant="soft" color="gray" disabled={busy}>
                  {t('user.security.cancel')}
                </Button>
              </Dialog.Close>
              <Button type="submit" loading={busy} disabled={code.trim() === '' || busy}>
                {t('user.security.confirm')}
              </Button>
            </Flex>
          </form>
        )}
        {step === 'codes' && <RecoveryCodesContent codes={codes} onDone={onClose} />}
      </Dialog.Content>
    </Dialog.Root>
  );
}

function AuthenticatorCard({ status, onStart }: AuthenticatorCardProps) {
  const t = useT();
  return (
    <Card size="2" className={styles.card}>
      <h2 className={styles.cardTitle}>{t('user.security.authenticator')}</h2>
      <div className={styles.row}>
        <span className={styles.label}>{t('user.security.status')}</span>
        <Badge size="1" color={status.enrolled ? 'green' : 'gray'}>
          {status.enrolled ? t('user.security.stateSet') : t('user.security.stateNotSet')}
        </Badge>
      </div>
      {status.enrolled && (
        <p className={styles.line}>
          {t('user.security.recoveryCodesLeft', { count: status.recoveryCodesLeft })}
        </p>
      )}
      {status.enrolled && status.recoveryCodesLeft <= LOW_RECOVERY_CODES && (
        <Callout.Root color="amber" size="1" className={styles.notice}>
          <Callout.Icon>
            <TriangleAlert size={16} aria-hidden="true" />
          </Callout.Icon>
          <Callout.Text>{t('user.security.recoveryLow')}</Callout.Text>
        </Callout.Root>
      )}
      <p className={styles.line}>{t('user.security.accessTokens', { count: status.accessTokens })}</p>
      <p className={styles.hint}>{t('user.security.accessTokensNote')}</p>
      {status.required && (
        <Callout.Root color="amber" size="1" className={styles.notice}>
          <Callout.Text>
            {t('user.security.required')}
            {status.enrollBy && (
              <>
                {' '}
                {t.rich('user.security.enrollBy', { date: <DateTime value={status.enrollBy} dateOnly /> })}
              </>
            )}
          </Callout.Text>
        </Callout.Root>
      )}
      {status.restricted && (
        <Callout.Root color="amber" size="1" className={styles.notice}>
          <Callout.Text>{t('user.security.restricted')}</Callout.Text>
        </Callout.Root>
      )}
      <Flex gap="2" wrap="wrap" mt="4">
        {status.enrolled ? (
          <>
            <Button variant="soft" onClick={() => onStart('regenerate')}>
              {t('user.security.newCodes')}
            </Button>
            <Button variant="soft" color="red" onClick={() => onStart('remove')}>
              {t('user.security.remove')}
            </Button>
          </>
        ) : (
          <Button onClick={() => onStart('setUp')}>{t('user.security.setUp')}</Button>
        )}
      </Flex>
    </Card>
  );
}

export default function SecurityPage() {
  const t = useT();
  const navigate = useNavigate();
  const status = useApi<TwoFactorStatus>(TWO_FACTOR);
  const [mode, setMode] = useState<FlowMode | null>(null);
  const freed = useRef(false);
  useDocumentTitle(t('user.security.title'));
  const managed = status.error instanceof ApiError && status.error.status === 400;

  const changed = async () => {
    const wasRestricted = status.data?.restricted === true;
    const latest = await status.refetch();
    if (wasRestricted && latest.data?.restricted === false) {
      freed.current = true;
    }
  };

  const finish = () => {
    if (freed.current) {
      freed.current = false;
      navigate('/');
    }
  };

  const close = () => {
    setMode(null);
    finish();
  };

  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <div className={styles.titleRow}>
          <ShieldCheck size={24} aria-hidden="true" className={styles.icon} />
          <h1 className={styles.title}>{t('user.security.title')}</h1>
        </div>
        <p className={styles.intro}>{t('user.security.intro')}</p>
      </header>
      {managed ? (
        <Callout.Root size="2">
          <Callout.Text>{t('user.security.managed')}</Callout.Text>
        </Callout.Root>
      ) : status.error ? (
        <ErrorState error={status.error} onRetry={() => void status.refetch()} />
      ) : status.data ? (
        <>
          <AuthenticatorCard status={status.data} onStart={setMode} />
          <PasskeysCard status={status.data} onChanged={changed} onClosed={finish} />
        </>
      ) : null}
      {mode !== null && <SecurityDialog key={mode} mode={mode} onClose={close} onChanged={changed} />}
    </div>
  );
}
