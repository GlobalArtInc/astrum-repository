import { Button, Card, Flex, Heading, Spinner, Text } from '@radix-ui/themes';
import { useState } from 'react';
import { useApi, withQuery } from '../../api';
import { formatNumber, useI18n, useT } from '../../i18n';
import { type Column, DataTable, ErrorState, FileSize, SelectField } from '../../ui';
import { DownloadsChart } from '../dashboard/DownloadsChart';
import { type UsageReport, usageCsvHref, usageUrl } from './api';
import {
  dailyDownloads,
  type RepositoryTotals,
  repositoryTotals,
  USAGE_PERIODS,
  type UsagePeriod,
} from './model';

export interface CustomerUsageProps {
  customerId: string;
}

export function CustomerUsage({ customerId }: CustomerUsageProps) {
  const t = useT();
  const { language } = useI18n();
  const [days, setDays] = useState<UsagePeriod>(30);
  const usage = useApi<UsageReport>(withQuery(usageUrl(customerId), { days }));
  const columns: Column<RepositoryTotals>[] = [
    {
      id: 'repository',
      header: t('customers.usage.repository'),
      primary: true,
      sortValue: (row) => row.repository,
      cell: (row) => <Text size="2">{row.repository}</Text>,
    },
    {
      id: 'downloads',
      header: t('customers.usage.downloads'),
      align: 'right',
      sortValue: (row) => row.downloads,
      cell: (row) => <Text size="2">{formatNumber(language, row.downloads)}</Text>,
    },
    {
      id: 'bytes',
      header: t('customers.usage.traffic'),
      align: 'right',
      sortValue: (row) => row.bytes,
      cell: (row) => (
        <Text size="2">
          <FileSize bytes={row.bytes} />
        </Text>
      ),
    },
  ];
  return (
    <Card>
      <Flex justify="between" align="end" gap="3" wrap="wrap" mb="3">
        <Heading as="h2" size="3" weight="medium">
          {t('customers.detail.usage')}
        </Heading>
        <Flex gap="3" align="end" wrap="wrap">
          <SelectField
            label={t('customers.usage.period')}
            value={String(days)}
            onChange={(value) => setDays(USAGE_PERIODS.find((period) => String(period) === value) ?? 30)}
            options={USAGE_PERIODS.map((period) => ({
              value: String(period),
              label: t('customers.period.days', { count: period }),
            }))}
          />
          <Button asChild variant="outline">
            <a href={usageCsvHref(customerId, days)} download>
              {t('customers.usage.csv')}
            </a>
          </Button>
        </Flex>
      </Flex>
      {usage.error ? (
        <ErrorState error={usage.error} onRetry={() => void usage.refetch()} />
      ) : usage.data === undefined ? (
        <Flex justify="center" py="6">
          <Spinner />
        </Flex>
      ) : (
        <Flex direction="column" gap="4">
          <Flex gap="6" wrap="wrap">
            <div>
              <Text as="div" size="2" color="gray">
                {t('customers.usage.downloads')}
              </Text>
              <Text as="div" size="5" weight="medium">
                {formatNumber(language, usage.data.totals.downloads)}
              </Text>
            </div>
            <div>
              <Text as="div" size="2" color="gray">
                {t('customers.usage.traffic')}
              </Text>
              <Text as="div" size="5" weight="medium">
                <FileSize bytes={usage.data.totals.bytes} />
              </Text>
            </div>
          </Flex>
          <DownloadsChart days={dailyDownloads(usage.data.daily, days, new Date())} />
          <DataTable
            label={t('customers.usage.tableLabel')}
            columns={columns}
            rows={repositoryTotals(usage.data.daily)}
            getRowId={(row) => row.repository}
            empty={
              <Text as="p" size="2" color="gray">
                {t('customers.usage.empty')}
              </Text>
            }
          />
        </Flex>
      )}
    </Card>
  );
}
