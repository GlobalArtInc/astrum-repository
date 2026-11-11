import { Button, Callout, Flex, Text } from '@radix-ui/themes';
import { Info } from 'lucide-react';
import { useState } from 'react';
import { Link } from 'react-router';
import { INTERNAL, useApi } from '../api';
import { SECURITY_PATH } from '../auth/session';
import { useSession } from '../auth/state';
import { formatDate, useI18n, useT } from '../i18n';
import styles from './SecurityBanner.module.css';

export const DISMISSED_KEY = 'astrum.securityBanner.dismissed';
const STATUS_PATH = `${INTERNAL}/ui/user/two-factor`;
const LOW_RECOVERY_CODES = 2;

export interface TwoFactorStatus {
  enrolled: boolean;
  enrollBy?: string | null;
  recoveryCodesLeft: number;
  restricted: boolean;
}

type BannerKind = 'enroll' | 'codes';

function bannerOf(status: TwoFactorStatus | undefined): BannerKind | null {
  if (!status || status.restricted) {
    return null;
  }
  if (status.enrollBy) {
    return 'enroll';
  }
  if (status.enrolled && status.recoveryCodesLeft <= LOW_RECOVERY_CODES) {
    return 'codes';
  }
  return null;
}

function readDismissed(): string | null {
  try {
    return window.sessionStorage.getItem(DISMISSED_KEY);
  } catch {
    return null;
  }
}

function rememberDismissed(kind: BannerKind): void {
  try {
    window.sessionStorage.setItem(DISMISSED_KEY, kind);
  } catch {}
}

export function SecurityBanner() {
  const t = useT();
  const { language } = useI18n();
  const { user, restricted } = useSession();
  const status = useApi<TwoFactorStatus>(user && !restricted ? STATUS_PATH : null, {
    staleTime: Number.POSITIVE_INFINITY,
    retry: false,
  });
  const [dismissed, setDismissed] = useState(readDismissed);
  const data = status.data;
  const kind = bannerOf(data);
  if (!data || !kind || dismissed === kind) {
    return null;
  }
  const dismiss = () => {
    rememberDismissed(kind);
    setDismissed(kind);
  };
  const message =
    kind === 'enroll'
      ? t('shell.securityBanner.enroll', { date: formatDate(language, data.enrollBy ?? '') })
      : t('shell.securityBanner.codes', { count: data.recoveryCodesLeft });
  return (
    <Callout.Root color="amber" variant="surface" className={styles.banner}>
      <Callout.Icon>
        <Info size={16} />
      </Callout.Icon>
      <Callout.Text>
        <Flex align="center" justify="between" gap="3" wrap="wrap">
          <Text size="2">{message}</Text>
          <Flex align="center" gap="3">
            <Link to={SECURITY_PATH} className={styles.link}>
              {t('shell.securityBanner.open')}
            </Link>
            <Button variant="ghost" size="1" onClick={dismiss}>
              {t('shell.securityBanner.dismiss')}
            </Button>
          </Flex>
        </Flex>
      </Callout.Text>
    </Callout.Root>
  );
}
