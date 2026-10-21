import { Button, Flex } from '@radix-ui/themes';
import { useT } from '../i18n';
import { Wordmark } from '../shell/Logo';
import { ErrorState } from '../ui/ErrorState';
import { AppTheme } from './RootLayout';

export function Starting() {
  const t = useT();
  return (
    <div className="boot" role="status" aria-label={t('shell.startup.loading')}>
      <span className="boot-logo">
        <Wordmark height={44} />
      </span>
      <span className="boot-status">
        <span className="boot-spinner" aria-hidden="true" />
        {t('shell.startup.progress')}
      </span>
    </div>
  );
}

export function StartupFailed({ error, onRetry }: { error: unknown; onRetry(): void }) {
  const t = useT();
  return (
    <AppTheme>
      <Flex direction="column" align="center" justify="center" style={{ minHeight: '100dvh' }}>
        <ErrorState error={error} title={t('shell.startup.failed')} />
        <Button onClick={onRetry}>{t('ui.error.retry')}</Button>
      </Flex>
    </AppTheme>
  );
}
