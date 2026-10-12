import { Button, Callout, Card, Dialog, Flex, Heading, Separator, Text, TextField } from '@radix-ui/themes';
import { Check, Copy, Package } from 'lucide-react';
import { Tabs } from 'radix-ui';
import { type FormEvent, useEffect, useId, useState } from 'react';
import { Navigate } from 'react-router';
import { ApiError, api } from '../../api';
import { useSession } from '../../auth/state';
import { useT } from '../../i18n';
import { ConfirmDialog, CopyButton, copyText, describeError, useDocumentTitle, useToast } from '../../ui';
import styles from './NugetApiKey.module.css';
import {
  dotnetSnippet,
  type KeySnippet,
  keySnippets,
  NUGET_KEY,
  NUGET_KEY_RESET,
  type RevealedKey,
} from './nugetKey';

type Purpose = 'access' | 'reset';

function CopyCodeButton({ code }: { code: string }) {
  const t = useT();
  const toast = useToast();
  const [copied, setCopied] = useState(false);
  useEffect(() => {
    if (!copied) {
      return;
    }
    const timer = window.setTimeout(() => setCopied(false), 1500);
    return () => window.clearTimeout(timer);
  }, [copied]);
  const copy = async () => {
    try {
      await copyText(code);
      setCopied(true);
    } catch {
      toast.error(t('user.nugetKey.copyFailed'));
    }
  };
  return (
    <Button type="button" variant="surface" color="gray" onClick={copy} className={styles.copy}>
      {copied ? <Check size={16} aria-hidden="true" /> : <Copy size={16} aria-hidden="true" />}
      {copied ? t('user.nugetKey.copied') : t('user.nugetKey.copy')}
    </Button>
  );
}

function SnippetCode({ code }: { code: string }) {
  return (
    <div className={styles.code}>
      <pre className={styles.pre}>
        <code>{code}</code>
      </pre>
      <CopyCodeButton code={code} />
    </div>
  );
}

function SnippetTabs({ snippets }: { snippets: KeySnippet[] }) {
  const t = useT();
  const first = snippets[0]?.id ?? 'dotnet';
  return (
    <Tabs.Root defaultValue={first} className={styles.tabsRoot}>
      <Tabs.List className={styles.tabs} aria-label={t('user.nugetKey.clients')}>
        {snippets.map((snippet) => (
          <Tabs.Trigger key={snippet.id} value={snippet.id} className={styles.tab}>
            {t(snippet.label)}
          </Tabs.Trigger>
        ))}
      </Tabs.List>
      {snippets.map((snippet) => (
        <Tabs.Content key={snippet.id} value={snippet.id} className={styles.tabContent}>
          <SnippetCode code={snippet.code} />
        </Tabs.Content>
      ))}
    </Tabs.Root>
  );
}

interface AuthenticateProps {
  purpose: Purpose | null;
  userId: string;
  onClose(): void;
  onDone(purpose: Purpose, revealed: RevealedKey | null): void;
}

function AuthenticateDialog({ purpose, userId, onClose, onDone }: AuthenticateProps) {
  const t = useT();
  const toast = useToast();
  const passwordId = useId();
  const [password, setPassword] = useState('');
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (purpose === null) {
      setPassword('');
    }
  }, [purpose]);
  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (purpose === null || busy) {
      return;
    }
    setBusy(true);
    try {
      if (purpose === 'access') {
        const revealed = await api.post<RevealedKey>(NUGET_KEY, { password });
        onDone(purpose, revealed);
      } else {
        await api.post(NUGET_KEY_RESET, { password });
        onDone(purpose, null);
      }
    } catch (error) {
      if (error instanceof ApiError && error.status === 400) {
        toast.error(t('user.nugetKey.authFailed'));
      } else if (error instanceof ApiError && error.status === 429) {
        toast.error(describeError(t, error));
      } else {
        toast.error(purpose === 'access' ? t('user.nugetKey.accessFailed') : t('user.nugetKey.resetFailed'));
      }
    } finally {
      setBusy(false);
    }
  };
  return (
    <Dialog.Root
      open={purpose !== null}
      onOpenChange={(open) => {
        if (!open && !busy) {
          onClose();
        }
      }}
    >
      <Dialog.Content maxWidth="480px" className={styles.dialog}>
        <form onSubmit={(event) => void submit(event)} className={styles.authForm}>
          <Dialog.Title mb="0">{t('user.nugetKey.authTitle')}</Dialog.Title>
          <Dialog.Description size="2">
            {purpose === 'reset' ? t('user.nugetKey.authReset') : t('user.nugetKey.authAccess')}
          </Dialog.Description>
          <Text as="p" size="2" weight="medium">
            {userId}
          </Text>
          <TextField.Root
            id={passwordId}
            type="password"
            value={password}
            onChange={(event) => setPassword(event.target.value)}
            placeholder={t('user.nugetKey.password')}
            aria-label={t('user.nugetKey.password')}
            autoComplete="current-password"
            autoFocus
            required
          />
          <Flex gap="3" justify="end" wrap="wrap">
            <Dialog.Close>
              <Button type="button" variant="soft" color="gray" disabled={busy}>
                {t('user.nugetKey.cancel')}
              </Button>
            </Dialog.Close>
            <Button type="submit" loading={busy} disabled={password === ''}>
              {t('user.nugetKey.authenticate')}
            </Button>
          </Flex>
        </form>
      </Dialog.Content>
    </Dialog.Root>
  );
}

interface KeyDialogProps {
  revealed: RevealedKey | null;
  baseUrl: string;
  userId: string;
  onClose(): void;
}

function KeyDialog({ revealed, baseUrl, userId, onClose }: KeyDialogProps) {
  const t = useT();
  const key = revealed?.apiKey ?? '';
  const warningId = useId();
  return (
    <Dialog.Root
      open={revealed !== null}
      onOpenChange={(open) => {
        if (!open) {
          onClose();
        }
      }}
    >
      <Dialog.Content maxWidth="520px" className={styles.dialog} aria-describedby={warningId}>
        <Dialog.Title>{t('user.nugetKey.dialogTitle')}</Dialog.Title>
        <Flex direction="column" gap="4">
          <Callout.Root color="amber" variant="soft" className={styles.warning}>
            <Callout.Text id={warningId}>{t('user.nugetKey.keepSecret')}</Callout.Text>
          </Callout.Root>
          <div className={styles.secretBlock}>
            <Text as="label" size="1" color="gray">
              {t('user.nugetKey.yourKey')}
            </Text>
            <div className={styles.secretRow}>
              <code className={styles.secret} data-testid="nuget-api-key">
                {key}
              </code>
              <CopyButton text={key} label={t('user.nugetKey.copyToClipboard')} />
            </div>
          </div>
          <Separator size="4" />
          <Heading as="h3" size="3">
            {t('user.nugetKey.commandTitle')}
          </Heading>
          <SnippetTabs
            snippets={[
              {
                id: 'dotnet',
                label: 'user.nugetKey.tabs.dotnetShort',
                code: dotnetSnippet(baseUrl, userId, key),
              },
            ]}
          />
          <Flex justify="end">
            <Dialog.Close>
              <Button variant="soft" color="gray">
                {t('user.nugetKey.close')}
              </Button>
            </Dialog.Close>
          </Flex>
        </Flex>
      </Dialog.Content>
    </Dialog.Root>
  );
}

export default function NugetApiKeyPage() {
  const t = useT();
  const toast = useToast();
  const { user, state } = useSession();
  const [purpose, setPurpose] = useState<Purpose | null>(null);
  const [confirmReset, setConfirmReset] = useState(false);
  const [revealed, setRevealed] = useState<RevealedKey | null>(null);
  const [key, setKey] = useState<string | null>(null);
  useDocumentTitle(t('user.nugetKey.title'));
  if (!state.nugetApiKeyRealm || !user) {
    return <Navigate to="/" replace />;
  }
  const done = (finished: Purpose, result: RevealedKey | null) => {
    setPurpose(null);
    if (finished === 'access' && result) {
      setKey(result.apiKey);
      setRevealed(result);
    } else {
      setKey(null);
      toast.success(t('user.nugetKey.resetDone'));
    }
  };
  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <Package size={24} className={styles.icon} aria-hidden="true" />
        <div className={styles.heading}>
          <Heading as="h1" size="5" weight="bold" className={styles.title}>
            {t('user.nugetKey.title')}
          </Heading>
          <Text as="span" size="2" color="gray">
            {t('user.nugetKey.subtitle')}
          </Text>
        </div>
      </header>
      <Card size="1" className={styles.card}>
        <div className={styles.cardBody}>
          <Heading as="h2" size="3" weight="bold">
            {t('user.nugetKey.yourKey')}
          </Heading>
          <Text as="p" size="2" color="gray">
            {t('user.nugetKey.intro')}
          </Text>
          <Flex gap="2" wrap="wrap" className={styles.actions}>
            <Button onClick={() => setPurpose('access')}>{t('user.nugetKey.access')}</Button>
            <Button color="red" variant="soft" onClick={() => setConfirmReset(true)}>
              {t('user.nugetKey.reset')}
            </Button>
          </Flex>
        </div>
      </Card>
      <Card size="1" className={styles.card}>
        <div className={styles.cardBody}>
          <Heading as="h2" size="3" weight="bold" className={styles.usageTitle}>
            {t('user.nugetKey.usage')}
          </Heading>
          <SnippetTabs snippets={keySnippets(state.baseUrl, user.userId, key)} />
        </div>
      </Card>
      <ConfirmDialog
        open={confirmReset}
        onOpenChange={setConfirmReset}
        title={t('user.nugetKey.resetTitle')}
        description={t('user.nugetKey.resetText')}
        confirmLabel={t('user.nugetKey.resetConfirm')}
        cancelLabel={t('user.nugetKey.cancel')}
        danger
        onConfirm={() => setPurpose('reset')}
      />
      <AuthenticateDialog
        purpose={purpose}
        userId={user.userId}
        onClose={() => setPurpose(null)}
        onDone={done}
      />
      <KeyDialog
        revealed={revealed}
        baseUrl={state.baseUrl}
        userId={user.userId}
        onClose={() => setRevealed(null)}
      />
    </div>
  );
}
