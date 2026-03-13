import { Button, Flex } from '@radix-ui/themes';
import { useRouteError } from 'react-router';
import { useT } from '../i18n';
import { ErrorState } from '../ui/ErrorState';
import { AppTheme } from './RootLayout';

export function RouteError() {
  const t = useT();
  const error = useRouteError();
  return (
    <AppTheme>
      <Flex direction="column" align="center" py="9">
        <ErrorState error={error} title={t('shell.crash.title')} />
        <Button onClick={() => window.location.assign('/')}>{t('shell.crash.home')}</Button>
      </Flex>
    </AppTheme>
  );
}
