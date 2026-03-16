import { Button, Flex, Heading, Text } from '@radix-ui/themes';
import { ShieldOff } from 'lucide-react';
import type { ReactNode } from 'react';
import { Link, Navigate, useLocation } from 'react-router';
import { useT } from '../i18n';
import { useDocumentTitle } from '../ui/hooks';
import type { Need } from './permissions';
import { loginPath } from './returnTo';
import { useSession } from './state';

export function NoAccess() {
  const t = useT();
  useDocumentTitle(t('auth.noAccess.title'));
  return (
    <Flex direction="column" align="center" gap="3" py="9" px="4" style={{ textAlign: 'center' }}>
      <ShieldOff size={40} strokeWidth={1.5} color="var(--gray-9)" aria-hidden="true" />
      <Heading as="h1" size="6">
        {t('auth.noAccess.title')}
      </Heading>
      <Text as="p" size="2" color="gray" style={{ maxWidth: '36rem' }}>
        {t('auth.noAccess.description')}
      </Text>
      <Button asChild mt="2" variant="soft">
        <Link to="/">{t('auth.noAccess.home')}</Link>
      </Button>
    </Flex>
  );
}

export function Guard({
  need,
  requiresUser = false,
  children,
}: {
  need?: Need;
  requiresUser?: boolean;
  children: ReactNode;
}) {
  const { user, can } = useSession();
  const location = useLocation();
  const allowed = can(need);
  if (!user && (requiresUser || !allowed)) {
    return <Navigate to={loginPath(location.pathname + location.search + location.hash)} replace />;
  }
  if (!allowed) {
    return <NoAccess />;
  }
  return children;
}
