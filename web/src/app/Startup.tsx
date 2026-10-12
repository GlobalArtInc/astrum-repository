import { Button, Flex, Spinner } from '@radix-ui/themes';
import { useT } from '../i18n';
import { ErrorState } from '../ui/ErrorState';
import { AppTheme } from './RootLayout';

export function Starting() {
  const t = useT();
  return (
    <AppTheme>
      <Flex align="center" justify="center" style={{ minHeight: '100dvh' }} role="status" aria-live="polite">
        <Spinner size="3" />
        <span className="visually-hidden">{t('shell.startup.loading')}</span>
      </Flex>
    </AppTheme>
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
