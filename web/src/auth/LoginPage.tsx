import { Button, Callout, Heading, Text, TextField } from '@radix-ui/themes';
import { CircleAlert, Info } from 'lucide-react';
import { type FormEvent, useEffect, useId, useRef, useState } from 'react';
import { Link, Navigate, useNavigate, useSearchParams } from 'react-router';
import { ApiError } from '../api';
import { useT } from '../i18n';
import { LanguageMenu } from '../shell/HeaderMenus';
import { Mark, Wordmark } from '../shell/Logo';
import { describeError } from '../ui/ErrorState';
import { useDocumentTitle } from '../ui/hooks';
import styles from './LoginPage.module.css';
import { safeReturnTo } from './returnTo';
import { SsoButton, SsoFailure } from './SsoButton';
import { type SecondFactorMethod, type SignInAnswer, secondStep, signIn, takeSessionEnded } from './session';
import { useRefreshSession, useSession } from './state';

const THROTTLE_SECONDS = 15;
const CODE_MAX_LENGTH = 64;
const EXPIRED_CODE = 'sign-in-expired';
const WRONG_CODE = 'second-factor-invalid';
const TOTP_CODE = /^\d{6}$/;

export function signInError(t: ReturnType<typeof useT>, error: unknown): string {
  if (error instanceof ApiError && error.status === 401) {
    return t('auth.login.wrong');
  }
  if (error instanceof ApiError && error.status === 429) {
    return t('auth.login.tooMany');
  }
  return describeError(t, error);
}

function challengeOf(answer: SignInAnswer): SecondFactorMethod[] | null {
  const methods = answer.secondFactor?.methods;
  return Array.isArray(methods) && methods.length > 0 ? methods : null;
}

function codeValue(mode: SecondFactorMethod, code: string): string {
  return mode === 'totp' ? code.replace(/\s+/g, '') : code;
}

function codeReady(mode: SecondFactorMethod, code: string): boolean {
  return mode === 'totp' ? TOTP_CODE.test(codeValue(mode, code)) : code.trim() !== '';
}

export function LoginPage() {
  const t = useT();
  const navigate = useNavigate();
  const [params] = useSearchParams();
  const { user, anonymousAccess, state } = useSession();
  const refresh = useRefreshSession();
  const returnTo = safeReturnTo(params.get('returnTo'));
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [methods, setMethods] = useState<SecondFactorMethod[] | null>(null);
  const [mode, setMode] = useState<SecondFactorMethod>('totp');
  const [code, setCode] = useState('');
  const [expired, setExpired] = useState(false);
  const [lockUntil, setLockUntil] = useState(0);
  const [clock, setClock] = useState(0);
  const locked = Math.max(0, Math.ceil((lockUntil - clock) / 1000));
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [ended] = useState(takeSessionEnded);
  const usernameInput = useRef<HTMLInputElement>(null);
  const codeInput = useRef<HTMLInputElement>(null);
  const usernameId = useId();
  const passwordId = useId();
  const codeId = useId();
  const errorId = useId();
  useDocumentTitle(t('auth.login.documentTitle'));

  useEffect(() => {
    if (lockUntil === 0) {
      return;
    }
    const timer = window.setInterval(() => {
      const now = Date.now();
      setClock(now);
      if (now >= lockUntil) {
        window.clearInterval(timer);
      }
    }, 250);
    return () => window.clearInterval(timer);
  }, [lockUntil]);

  if (user && !busy) {
    return <Navigate to={returnTo} replace />;
  }

  const ready = username.trim() !== '' && password !== '';
  const canSubmit = methods ? codeReady(mode, code) : ready;
  const lockedText = t('auth.secondStep.locked', { count: locked });
  const message = error ?? (methods && locked > 0 ? lockedText : null);

  const lockOn = (failure: unknown) => {
    if (failure instanceof ApiError && failure.status === 429) {
      const now = Date.now();
      setClock(now);
      setLockUntil(now + (failure.retryAfter ?? THROTTLE_SECONDS) * 1000);
    }
  };

  const submitPassword = async (event: FormEvent) => {
    event.preventDefault();
    if (!ready || busy || locked > 0) {
      return;
    }
    setBusy(true);
    setError(null);
    setExpired(false);
    try {
      const answer = await signIn(username.trim(), password);
      const second = challengeOf(answer);
      if (second) {
        setPassword('');
        setMethods(second);
        setMode(second.includes('totp') ? 'totp' : 'recovery');
        setBusy(false);
        return;
      }
      await refresh();
      navigate(returnTo, { replace: true });
    } catch (failure) {
      setError(signInError(t, failure));
      setPassword('');
      setBusy(false);
      lockOn(failure);
      usernameInput.current?.focus();
    }
  };

  const submitCode = async (event: FormEvent) => {
    event.preventDefault();
    if (!methods || !codeReady(mode, code) || busy || locked > 0) {
      return;
    }
    setBusy(true);
    setError(null);
    setExpired(false);
    try {
      await secondStep(mode, codeValue(mode, code));
      await refresh();
      navigate(returnTo, { replace: true });
    } catch (failure) {
      setBusy(false);
      setCode('');
      if (failure instanceof ApiError && failure.code === EXPIRED_CODE) {
        setMethods(null);
        setExpired(true);
        return;
      }
      if (failure instanceof ApiError && failure.code === WRONG_CODE) {
        setError(t('auth.secondStep.wrong'));
      } else if (!(failure instanceof ApiError && failure.status === 429)) {
        setError(describeError(t, failure));
      }
      lockOn(failure);
      codeInput.current?.focus();
    }
  };

  const switchMode = (next: SecondFactorMethod) => {
    setMode(next);
    setCode('');
    setError(null);
  };

  const backToPassword = () => {
    setMethods(null);
    setCode('');
    setError(null);
  };

  return (
    <div className={styles.page}>
      <header className={styles.top}>
        <Link to="/" className={styles.home} aria-label={t('shell.header.home')}>
          <Wordmark height={28} />
        </Link>
        <LanguageMenu />
      </header>
      <main className={styles.center}>
        <div className={styles.card}>
          <div className={styles.content}>
            <div className={styles.intro}>
              <Mark size={44} />
              <Heading as="h1" size="6" className={styles.title}>
                {methods ? t('auth.secondStep.title') : t('auth.login.title')}
              </Heading>
              <Text as="p" size="3" className={styles.subtitle}>
                {methods
                  ? mode === 'totp'
                    ? t('auth.secondStep.totpHint')
                    : t('auth.secondStep.recoveryHint')
                  : t('auth.login.subtitle')}
              </Text>
            </div>
            <form className={styles.form} onSubmit={methods ? submitCode : submitPassword} noValidate>
              {ended && !error && !methods && (
                <Callout.Root color="amber" role="status">
                  <Callout.Icon>
                    <Info size={16} />
                  </Callout.Icon>
                  <Callout.Text>{t('auth.login.ended')}</Callout.Text>
                </Callout.Root>
              )}
              {expired && (
                <Callout.Root color="amber" role="status">
                  <Callout.Icon>
                    <Info size={16} />
                  </Callout.Icon>
                  <Callout.Text>{t('auth.login.secondStepExpired')}</Callout.Text>
                </Callout.Root>
              )}
              {!methods && !error && <SsoFailure code={params.get('sso')} />}
              {methods ? (
                <div className={styles.field}>
                  <label htmlFor={codeId} className={styles.label}>
                    {mode === 'totp' ? t('auth.secondStep.codeLabel') : t('auth.secondStep.recoveryLabel')}
                  </label>
                  <TextField.Root
                    ref={codeInput}
                    id={codeId}
                    size="3"
                    className={styles.input}
                    name="code"
                    value={code}
                    placeholder={
                      mode === 'totp' ? t('auth.secondStep.codeLabel') : t('auth.secondStep.recoveryLabel')
                    }
                    inputMode={mode === 'totp' ? 'numeric' : 'text'}
                    autoComplete={mode === 'totp' ? 'one-time-code' : 'off'}
                    maxLength={CODE_MAX_LENGTH}
                    autoFocus
                    aria-describedby={message ? errorId : undefined}
                    aria-invalid={message ? true : undefined}
                    onChange={(event) => setCode(event.target.value)}
                  />
                </div>
              ) : (
                <>
                  <div className={styles.field}>
                    <label htmlFor={usernameId} className={styles.label}>
                      {t('auth.login.username')}
                    </label>
                    <TextField.Root
                      ref={usernameInput}
                      id={usernameId}
                      size="3"
                      className={styles.input}
                      name="username"
                      value={username}
                      placeholder={t('auth.login.username')}
                      autoComplete="username"
                      autoFocus
                      onChange={(event) => setUsername(event.target.value)}
                    />
                  </div>
                  <div className={styles.field}>
                    <label htmlFor={passwordId} className={styles.label}>
                      {t('auth.login.password')}
                    </label>
                    <TextField.Root
                      id={passwordId}
                      size="3"
                      type="password"
                      className={styles.input}
                      name="password"
                      value={password}
                      placeholder={t('auth.login.password')}
                      autoComplete="current-password"
                      aria-describedby={message ? errorId : undefined}
                      aria-invalid={message ? true : undefined}
                      onChange={(event) => setPassword(event.target.value)}
                    />
                  </div>
                </>
              )}
              {message && (
                <div id={errorId} className={styles.error} role="alert">
                  <CircleAlert size={16} aria-hidden="true" />
                  <Text size="2">{message}</Text>
                </div>
              )}
              <Button
                type="submit"
                size="3"
                className={styles.submit}
                disabled={!canSubmit || locked > 0}
                loading={busy}
              >
                {t('auth.login.submit')}
              </Button>
              {methods && (
                <div className={styles.links}>
                  {mode === 'totp' && methods.includes('recovery') && (
                    <button
                      type="button"
                      className={styles.linkButton}
                      onClick={() => switchMode('recovery')}
                    >
                      {t('auth.secondStep.useRecovery')}
                    </button>
                  )}
                  {mode === 'recovery' && methods.includes('totp') && (
                    <button type="button" className={styles.linkButton} onClick={() => switchMode('totp')}>
                      {t('auth.secondStep.useApp')}
                    </button>
                  )}
                  <button type="button" className={styles.linkButton} onClick={backToPassword}>
                    {t('auth.secondStep.back')}
                  </button>
                </div>
              )}
              {!methods && state.oidcEnabled && <SsoButton returnTo={returnTo} />}
              {!methods && state.adminPasswordFile && (
                <Callout.Root color="gray" variant="surface">
                  <Callout.Icon>
                    <Info size={16} />
                  </Callout.Icon>
                  <Callout.Text>
                    {t.rich('auth.login.adminPassword', {
                      path: (
                        <span key="path" className={styles.path}>
                          {state.adminPasswordFile}
                        </span>
                      ),
                    })}
                  </Callout.Text>
                </Callout.Root>
              )}
            </form>
            {anonymousAccess && (
              <div className={styles.alternative}>
                <Link to={returnTo} className={styles.anonymous}>
                  {t('auth.login.anonymous')}
                </Link>
              </div>
            )}
          </div>
        </div>
      </main>
    </div>
  );
}
