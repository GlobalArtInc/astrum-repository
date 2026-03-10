import { Button, Flex, Heading, Text } from '@radix-ui/themes';
import { CircleAlert, RotateCw } from 'lucide-react';
import { ApiError } from '../api';
import { type Translate, useT } from '../i18n';

export function describeError(t: Translate, error: unknown): string {
  if (error instanceof ApiError) {
    if (error.isNetwork) {
      return t('ui.error.network');
    }
    if (error.status === 401) {
      return t('ui.error.unauthorized');
    }
    if (error.status === 403) {
      return t('ui.error.forbidden');
    }
    if (error.status === 404) {
      return t('ui.error.notFound');
    }
    if (error.status >= 500) {
      return t('ui.error.server', { detail: error.message });
    }
    return error.message;
  }
  return error instanceof Error ? error.message : t('ui.error.unknown');
}

export interface ErrorStateProps {
  error: unknown;
  title?: string;
  onRetry?: () => void;
}

export function ErrorState({ error, title, onRetry }: ErrorStateProps) {
  const t = useT();
  return (
    <Flex
      role="alert"
      direction="column"
      align="center"
      justify="center"
      gap="2"
      py="8"
      px="4"
      style={{ textAlign: 'center' }}
    >
      <CircleAlert size={32} strokeWidth={1.5} color="var(--red-9)" aria-hidden="true" />
      <Heading as="h2" size="3" weight="medium">
        {title ?? t('ui.error.title')}
      </Heading>
      <Text as="p" size="2" color="gray" style={{ maxWidth: '36rem', overflowWrap: 'anywhere' }}>
        {describeError(t, error)}
      </Text>
      {onRetry && (
        <Button variant="soft" mt="2" onClick={onRetry}>
          <RotateCw size={16} aria-hidden="true" />
          {t('ui.error.retry')}
        </Button>
      )}
    </Flex>
  );
}
