import { Button, Callout } from '@radix-ui/themes';
import { CircleAlert } from 'lucide-react';
import { useState } from 'react';
import { useT } from '../i18n';
import styles from './SsoButton.module.css';

export const SSO_LOGIN = '/oidc/login';

const FAILURES: Record<string, string> = {
  expired: 'auth.sso.expired',
  provider: 'auth.sso.provider',
  denied: 'auth.sso.denied',
  failed: 'auth.sso.failed',
};

export function ssoHref(returnTo: string): string {
  const bytes = new TextEncoder().encode(returnTo);
  let binary = '';
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }
  return `${SSO_LOGIN}?hash=${encodeURIComponent(btoa(binary))}`;
}

export function SsoFailure({ code }: { code: string | null }) {
  const t = useT();
  if (code === null) {
    return null;
  }
  return (
    <Callout.Root color="red" role="alert">
      <Callout.Icon>
        <CircleAlert size={16} />
      </Callout.Icon>
      <Callout.Text>{t(FAILURES[code] ?? 'auth.sso.failed')}</Callout.Text>
    </Callout.Root>
  );
}

export function SsoButton({ returnTo }: { returnTo: string }) {
  const t = useT();
  const [going, setGoing] = useState(false);
  return (
    <>
      <div className={styles.divider} aria-hidden="true">
        <span>{t('auth.sso.divider')}</span>
      </div>
      <Button
        type="button"
        size="3"
        variant="outline"
        loading={going}
        onClick={() => {
          setGoing(true);
          window.location.assign(ssoHref(returnTo));
        }}
      >
        {going ? t('auth.sso.loading') : t('auth.sso.button')}
      </Button>
    </>
  );
}
