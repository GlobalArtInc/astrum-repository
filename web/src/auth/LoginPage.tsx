import { Button, Callout, Flex, Heading, Link as RadixLink, Text } from '@radix-ui/themes';
import { CircleAlert, Info } from 'lucide-react';
import { type FormEvent, useState } from 'react';
import { Link, Navigate, useNavigate, useSearchParams } from 'react-router';
import { ApiError } from '../api';
import { useT } from '../i18n';
import { LanguageMenu } from '../shell/HeaderMenus';
import { Logo, Sparkle } from '../shell/Logo';
import { describeError } from '../ui/ErrorState';
import { PasswordField, TextField } from '../ui/form';
import { useDocumentTitle } from '../ui/hooks';
import styles from './LoginPage.module.css';
import { safeReturnTo } from './returnTo';
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
  const [missing, setMissing] = useState<{ username?: string; password?: string }>({});
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [ended] = useState(takeSessionEnded);
  useDocumentTitle(t('auth.login.documentTitle'));

  if (user && !busy) {
    return <Navigate to={returnTo} replace />;
  }

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    const problems = {
      username: username.trim() === '' ? t('auth.login.usernameRequired') : undefined,
      password: password === '' ? t('auth.login.passwordRequired') : undefined,
    };
    setMissing(problems);
    if (problems.username || problems.password) {
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
      setBusy(false);
    }
  };

  return (
    <div className={styles.page}>
      <header className={styles.top}>
        <Link to="/" className={styles.home} aria-label={t('shell.header.home')}>
          <Logo />
        </Link>
        <LanguageMenu />
      </header>
      <main className={styles.center}>
        <div className={styles.card}>
          <Flex direction="column" align="center" gap="2" mb="5">
            <Sparkle size={40} />
            <Heading as="h1" size="6">
              {t('auth.login.title')}
            </Heading>
            <Text size="2" color="gray">
              {t('auth.login.subtitle')}
            </Text>
          </Flex>
          <form className={styles.form} onSubmit={submit} noValidate>
            {ended && !error && (
              <Callout.Root color="amber" role="status">
                <Callout.Icon>
                  <Info size={16} />
                </Callout.Icon>
                <Callout.Text>{t('auth.login.ended')}</Callout.Text>
              </Callout.Root>
            )}
            {error && (
              <Callout.Root color="red" role="alert">
                <Callout.Icon>
                  <CircleAlert size={16} />
                </Callout.Icon>
                <Callout.Text>{error}</Callout.Text>
              </Callout.Root>
            )}
            <TextField
              label={t('auth.login.username')}
              name="username"
              value={username}
              onChange={setUsername}
              autoComplete="username"
              autoFocus
              size="3"
              error={missing.username}
            />
            <PasswordField
              label={t('auth.login.password')}
              name="password"
              value={password}
              onChange={setPassword}
              autoComplete="current-password"
              size="3"
              error={missing.password}
            />
            <Button type="submit" size="3" loading={busy}>
              {t('auth.login.submit')}
            </Button>
            {anonymousAccess && (
              <Text size="2" align="center">
                <RadixLink asChild>
                  <Link to={returnTo}>{t('auth.login.anonymous')}</Link>
                </RadixLink>
              </Text>
            )}
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
        </div>
      </main>
    </div>
  );
}
