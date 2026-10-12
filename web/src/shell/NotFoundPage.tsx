import { Button, Code, Flex, Heading, Text } from '@radix-ui/themes';
import { MapPinOff } from 'lucide-react';
import { Link, useLocation } from 'react-router';
import { useT } from '../i18n';
import { useDocumentTitle } from '../ui/hooks';

export function NotFoundPage() {
  const t = useT();
  const location = useLocation();
  useDocumentTitle(t('shell.notFound.title'));
  const address = location.pathname + location.search + location.hash;
  return (
    <Flex direction="column" align="center" gap="3" py="9" px="4" style={{ textAlign: 'center' }}>
      <MapPinOff size={40} strokeWidth={1.5} color="var(--gray-9)" aria-hidden="true" />
      <Heading as="h1" size="6">
        {t('shell.notFound.title')}
      </Heading>
      <Text as="p" size="2" color="gray" style={{ maxWidth: '36rem', overflowWrap: 'anywhere' }}>
        {t.rich('shell.notFound.description', { address: <Code key="address">{address}</Code> })}
      </Text>
      <Button asChild mt="2">
        <Link to="/">{t('shell.notFound.home')}</Link>
      </Button>
    </Flex>
  );
}
