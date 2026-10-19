import { Button, Callout, Card, Checkbox, Flex, Heading, Spinner, Text } from '@radix-ui/themes';
import { Bot, CircleAlert } from 'lucide-react';
import { type ReactNode, useState } from 'react';
import { useSearchParams } from 'react-router';
import { ApiError, api, useApi } from '../../api';
import { leaveFor } from '../../auth/browser';
import { useT } from '../../i18n';
import { describeError, useDocumentTitle } from '../../ui';
import styles from './Authorize.module.css';
import { type AuthorizationRequest, requestPath } from './agents';

const PROBLEMS: Record<string, string> = {
  client: 'user.authorize.problemClient',
  redirect: 'user.authorize.problemRedirect',
};

function Problem({ message }: { message: string }) {
  return (
    <Callout.Root color="red" role="alert">
      <Callout.Icon>
        <CircleAlert size={16} />
      </Callout.Icon>
      <Callout.Text>{message}</Callout.Text>
    </Callout.Root>
  );
}

function Consent({ request }: { request: AuthorizationRequest }) {
  const t = useT();
  const [write, setWrite] = useState(false);
  const [busy, setBusy] = useState<'allow' | 'deny' | null>(null);
  const [failure, setFailure] = useState<unknown>(null);
  const answer = async (action: 'approve' | 'deny') => {
    setBusy(action === 'approve' ? 'allow' : 'deny');
    setFailure(null);
    try {
      const reply = await api.post<{ redirect: string }>(
        requestPath(request.id, action),
        action === 'approve' ? { write } : undefined,
      );
      leaveFor(reply.redirect);
    } catch (error) {
      setFailure(error);
      setBusy(null);
    }
  };
  const names = { client: request.clientName, user: request.user, host: request.redirectHost };
  return (
    <Card size="3" className={styles.card}>
      <Flex direction="column" gap="4">
        <Flex align="center" gap="3">
          <Bot size={28} aria-hidden="true" color="var(--accent-11)" />
          <Heading as="h1" size="5">
            {t('user.authorize.heading', names)}
          </Heading>
        </Flex>
        <Text as="p" size="2">
          {t('user.authorize.text', names)}
        </Text>
        <Text as="p" size="2" color="gray">
          {t('user.authorize.read')}
        </Text>
        <Text as="label" size="2" className={styles.write}>
          <Flex gap="2" align="start">
            <Checkbox checked={write} onCheckedChange={(checked) => setWrite(checked === true)} />
            <span>
              <span className={styles.writeLabel}>{t('user.authorize.write')}</span>
              <span className={styles.writeText}>{t('user.authorize.writeText')}</span>
            </span>
          </Flex>
        </Text>
        {failure !== null && <Problem message={describeError(t, failure)} />}
        <Flex gap="3" justify="end">
          <Button
            variant="soft"
            color="gray"
            disabled={busy !== null}
            loading={busy === 'deny'}
            onClick={() => void answer('deny')}
          >
            {t('user.authorize.deny')}
          </Button>
          <Button disabled={busy !== null} loading={busy === 'allow'} onClick={() => void answer('approve')}>
            {t('user.authorize.allow')}
          </Button>
        </Flex>
      </Flex>
    </Card>
  );
}

export default function AuthorizePage() {
  const t = useT();
  const [params] = useSearchParams();
  const id = params.get('request');
  const problem = params.get('problem');
  const request = useApi<AuthorizationRequest>(id ? requestPath(id) : null);
  useDocumentTitle(t('user.authorize.title'));

  let body: ReactNode;
  if (problem) {
    body = <Problem message={t(PROBLEMS[problem] ?? 'user.authorize.missing')} />;
  } else if (!id) {
    body = <Problem message={t('user.authorize.missing')} />;
  } else if (request.error) {
    body = (
      <Problem
        message={
          request.error instanceof ApiError && request.error.status === 404
            ? t('user.authorize.expired')
            : describeError(t, request.error)
        }
      />
    );
  } else if (!request.data) {
    body = (
      <Flex justify="center" py="6">
        <Spinner size="3" />
      </Flex>
    );
  } else {
    body = <Consent request={request.data} />;
  }
  return <div className={styles.page}>{body}</div>;
}
