import { Card, Flex, Text } from '@radix-ui/themes';
import { ChartNoAxesColumn } from 'lucide-react';
import { useT } from '../../i18n';
import { Page } from '../../ui';

export function DashboardPage() {
  const t = useT();
  return (
    <Page title={t('dashboard.home.title')} description={t('dashboard.home.description')}>
      <Card size="3">
        <Flex direction="column" align="center" gap="2" py="6" style={{ textAlign: 'center' }}>
          <ChartNoAxesColumn size={32} strokeWidth={1.5} color="var(--gray-9)" aria-hidden="true" />
          <Text as="p" size="3" weight="medium">
            {t('dashboard.home.placeholderTitle')}
          </Text>
          <Text as="p" size="2" color="gray">
            {t('dashboard.home.placeholderText')}
          </Text>
        </Flex>
      </Card>
    </Page>
  );
}
