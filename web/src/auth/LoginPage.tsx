import { Button, Callout, Heading, Text, TextField } from '@radix-ui/themes';
import { CircleAlert, Info } from 'lucide-react';
import { type FormEvent, useId, useRef, useState } from 'react';
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
import { signIn, takeSessionEnded } from './session';
import { useRefreshSession, useSession } from './state';

export function signInError(t: ReturnType<typeof useT>, error: unknown): string {
  if (error instanceof ApiError && error.status === 401) {
    return t('auth.login.wrong');
  }
  if (error instanceof ApiError && error.status === 429) {
    return t('auth.login.tooMany');
  }
  return describeError(t, error);
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
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [ended] = useState(takeSessionEnded);
  const usernameInput = useRef<HTMLInputElement>(null);
  const usernameId = useId();
  const passwordId = useId();
  const errorId = useId();
  useDocumentTitle(t('auth.login.documentTitle'));

  if (user && !busy) {
    return <Navigate to={returnTo} replace />;
  }

  const ready = username.trim() !== '' && password !== '';

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (!ready || busy) {
      return;
    }
    setBusy(true);
    setError(null);
    try {
      await signIn(username.trim(), password);
      await refresh();
      navigate(returnTo, { replace: true });
    } catch (failure) {
      setError(signInError(t, failure));
      setPassword('');
      setBusy(false);
      usernameInput.current?.focus();
    }
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
                {t('auth.login.title')}
              </Heading>
              <Text as="p" size="3" className={styles.subtitle}>
                {t('auth.login.subtitle')}
              </Text>
            </div>
            <form className={styles.form} onSubmit={submit} noValidate>
              {ended && !error && (
                <Callout.Root color="amber" role="status">
                  <Callout.Icon>
                    <Info size={16} />
                  </Callout.Icon>
                  <Callout.Text>{t('auth.login.ended')}</Callout.Text>
                </Callout.Root>
              )}
              {!error && <SsoFailure code={params.get('sso')} />}
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
                  aria-describedby={error ? errorId : undefined}
                  aria-invalid={error ? true : undefined}
                  onChange={(event) => setPassword(event.target.value)}
                />
              </div>
              {error && (
                <div id={errorId} className={styles.error} role="alert">
                  <CircleAlert size={16} aria-hidden="true" />
                  <Text size="2">{error}</Text>
                </div>
              )}
              <Button type="submit" size="3" className={styles.submit} disabled={!ready} loading={busy}>
                {t('auth.login.submit')}
              </Button>
              {state.oidcEnabled && <SsoButton returnTo={returnTo} />}
              {state.adminPasswordFile && (
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
