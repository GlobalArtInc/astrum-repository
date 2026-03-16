import { AlertDialog, Button, Flex } from '@radix-ui/themes';
import { useQueryClient } from '@tanstack/react-query';
import { useEffect, useRef, useState } from 'react';
import { useLocation, useNavigate } from 'react-router';
import { onUnauthorized, type UiState } from '../api';
import { useT } from '../i18n';
import { LOGIN_PATH, loginPath } from './returnTo';
import { markSessionEnded, takeExpectedSignOut } from './session';
import { STATE_KEY, useSession } from './state';

type Reason = 'expired' | 'required';

export function SessionWatcher() {
  const t = useT();
  const navigate = useNavigate();
  const location = useLocation();
  const client = useQueryClient();
  const { user } = useSession();
  const [reason, setReason] = useState<Reason | null>(null);
  const signedIn = useRef(user !== null);
  const userId = user?.userId ?? null;
  const previous = useRef(userId);

  useEffect(() => {
    signedIn.current = userId !== null;
    if (previous.current !== null && userId === null && !takeExpectedSignOut()) {
      markSessionEnded();
      setReason('expired');
    }
    if (userId !== null) {
      setReason(null);
    }
    previous.current = userId;
  }, [userId]);

  useEffect(
    () =>
      onUnauthorized(() => {
        const current = client.getQueryData<UiState>(STATE_KEY);
        if (current?.user) {
          previous.current = null;
          markSessionEnded();
          client.setQueryData<UiState>(STATE_KEY, { ...current, user: null, permissions: [] });
        }
        setReason(signedIn.current ? 'expired' : 'required');
        void client.invalidateQueries({ queryKey: STATE_KEY });
      }),
    [client],
  );

  const open = reason !== null && location.pathname !== LOGIN_PATH;
  const signIn = () => {
    setReason(null);
    navigate(loginPath(location.pathname + location.search + location.hash));
  };

  return (
    <AlertDialog.Root open={open} onOpenChange={(next) => !next && setReason(null)}>
      <AlertDialog.Content maxWidth="440px">
        <AlertDialog.Title>
          {reason === 'expired' ? t('auth.expired.title') : t('auth.required.title')}
        </AlertDialog.Title>
        <AlertDialog.Description size="2" color="gray">
          {reason === 'expired' ? t('auth.expired.description') : t('auth.required.description')}
        </AlertDialog.Description>
        <Flex gap="3" mt="4" justify="end" wrap="wrap">
          <AlertDialog.Cancel>
            <Button variant="soft" color="gray" onClick={() => window.location.reload()}>
              {t('auth.expired.reload')}
            </Button>
          </AlertDialog.Cancel>
          <AlertDialog.Action>
            <Button onClick={signIn}>{t('auth.expired.signIn')}</Button>
          </AlertDialog.Action>
        </Flex>
      </AlertDialog.Content>
    </AlertDialog.Root>
  );
}
