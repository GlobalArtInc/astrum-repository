import { Button, Callout, Dialog, Flex, TextField } from '@radix-ui/themes';
import { CircleAlert, CircleCheck, KeyRound, Lock, User } from 'lucide-react';
import { type FormEvent, useId, useState } from 'react';
import { api, withQuery } from '../../../api';
import { useT } from '../../../i18n';
import { LDAP_VERIFY_LOGIN } from '../api';
import { failureText } from '../shared/errors';
import layout from '../shared/Layout.module.css';
import styles from './Dialogs.module.css';
import { base64 } from './model';

export function PasswordDialog({
  text,
  onVerify,
  onClose,
}: {
  text: string;
  onVerify(password: string): void;
  onClose(): void;
}) {
  const t = useT();
  const id = useId();
  const [password, setPassword] = useState('');
  const submit = (event: FormEvent) => {
    event.preventDefault();
    if (password !== '') {
      onVerify(password);
    }
  };
  return (
    <Dialog.Root
      open
      onOpenChange={(open) => {
        if (!open) {
          onClose();
        }
      }}
    >
      <Dialog.Content maxWidth="450px">
        <form onSubmit={submit}>
          <Dialog.Title>
            <Flex align="center" gap="2">
              <Lock size={20} aria-hidden="true" />
              {t('identity.ldap.passwordTitle')}
            </Flex>
          </Dialog.Title>
          <Dialog.Description size="2">{text}</Dialog.Description>
          <div className={styles.field}>
            <label htmlFor={id} className={styles.label}>
              <Lock size={14} aria-hidden="true" />
              {t('identity.ldap.password')}
            </label>
            <TextField.Root
              id={id}
              type="password"
              autoFocus
              autoComplete="off"
              value={password}
              placeholder={t('identity.ldap.passwordPlaceholder')}
              onChange={(event) => setPassword(event.target.value)}
            />
          </div>
          <Flex gap="3" mt="4" justify="end" wrap="wrap">
            <Dialog.Close>
              <Button type="button" size="3" variant="outline" color="gray" className={layout.neutralButton}>
                {t('identity.ldap.cancel')}
              </Button>
            </Dialog.Close>
            <Button type="submit" size="3" className={layout.bigButton} disabled={password === ''}>
              {t('identity.ldap.verify')}
            </Button>
          </Flex>
        </form>
      </Dialog.Content>
    </Dialog.Root>
  );
}

type Outcome = { ok: true; url: string } | { ok: false; message: string };

export function TestLoginDialog({
  server,
  url,
  existingName,
  onClose,
}: {
  server: object;
  url: string;
  existingName?: string;
  onClose(): void;
}) {
  const t = useT();
  const userId = useId();
  const passwordId = useId();
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [busy, setBusy] = useState(false);
  const [outcome, setOutcome] = useState<Outcome | null>(null);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (username.trim() === '' || password === '') {
      return;
    }
    setBusy(true);
    setOutcome(null);
    try {
      await api.post(withQuery(LDAP_VERIFY_LOGIN, { existingServerName: existingName }), {
        base64Username: base64(username.trim()),
        base64Password: base64(password),
        ldapServer: server,
      });
      setOutcome({ ok: true, url });
    } catch (failure) {
      setOutcome({ ok: false, message: failureText(t, failure) });
    } finally {
      setBusy(false);
    }
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
      <Dialog.Content maxWidth="450px">
        <form onSubmit={submit}>
          <Dialog.Title>
            <Flex align="center" gap="2">
              <KeyRound size={20} aria-hidden="true" />
              {t('identity.ldap.loginTitle')}
            </Flex>
          </Dialog.Title>
          <Dialog.Description size="2">{t('identity.ldap.loginText')}</Dialog.Description>
          <div className={styles.field}>
            <label htmlFor={userId} className={styles.label}>
              <User size={14} aria-hidden="true" />
              {t('identity.ldap.username')}
            </label>
            <TextField.Root
              id={userId}
              autoFocus
              autoComplete="off"
              value={username}
              placeholder={t('identity.ldap.loginUsernamePlaceholder')}
              onChange={(event) => setUsername(event.target.value)}
            />
          </div>
          <div className={styles.field}>
            <label htmlFor={passwordId} className={styles.label}>
              <Lock size={14} aria-hidden="true" />
              {t('identity.ldap.password')}
            </label>
            <TextField.Root
              id={passwordId}
              type="password"
              autoComplete="off"
              value={password}
              placeholder={t('identity.ldap.loginPasswordPlaceholder')}
              onChange={(event) => setPassword(event.target.value)}
            />
          </div>
          {outcome && (
            <Callout.Root
              className={styles.outcome}
              color={outcome.ok ? 'green' : 'red'}
              role={outcome.ok ? 'status' : 'alert'}
            >
              <Callout.Icon>
                {outcome.ok ? (
                  <CircleCheck size={16} aria-hidden="true" />
                ) : (
                  <CircleAlert size={16} aria-hidden="true" />
                )}
              </Callout.Icon>
              <Callout.Text>
                {outcome.ok ? t('identity.ldap.loginOk', { url: outcome.url }) : outcome.message}
              </Callout.Text>
            </Callout.Root>
          )}
          <Flex gap="3" mt="5" justify="end" wrap="wrap">
            <Dialog.Close>
              <Button
                type="button"
                size="3"
                variant="outline"
                color="gray"
                className={layout.neutralButton}
                disabled={busy}
              >
                {t('identity.ldap.close')}
              </Button>
            </Dialog.Close>
            <Button
              type="submit"
              size="3"
              loading={busy}
              disabled={username.trim() === '' || password === ''}
            >
              {t('identity.ldap.testLogin')}
            </Button>
          </Flex>
        </form>
      </Dialog.Content>
    </Dialog.Root>
  );
}
