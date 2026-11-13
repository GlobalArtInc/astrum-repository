import { Flex, SegmentedControl, Text } from '@radix-ui/themes';
import { useState } from 'react';
import { formatBytes, formatNumber, useT } from '../../../i18n';
import { DownloadsChart } from '../../dashboard/DownloadsChart';
import { USAGE_PERIODS, type UsageHistoryPoint, type UsagePeriod, useUsageHistory } from '../api';

function isPeriod(value: string): value is `${UsagePeriod}` {
  return USAGE_PERIODS.some((period) => String(period) === value);
}

export function UsageHistory({ name }: { name: string }) {
  const t = useT();
  const [days, setDays] = useState<UsagePeriod>(30);
  const query = useUsageHistory(name, days);
  const points: UsageHistoryPoint[] | undefined = query.data;
  return (
    <Flex direction="column" gap="3">
      <SegmentedControl.Root
        size="1"
        value={String(days)}
        onValueChange={(next) => {
          if (isPeriod(next)) {
            setDays(Number(next) as UsagePeriod);
          }
        }}
        aria-label={t('repositories.usage.period')}
      >
        {USAGE_PERIODS.map((period) => (
          <SegmentedControl.Item key={period} value={String(period)}>
            {t('repositories.usage.days', { count: period })}
          </SegmentedControl.Item>
        ))}
      </SegmentedControl.Root>
      {query.isError && !points ? (
        <Text as="p" size="2" color="gray">
          {t('repositories.usage.error')}
        </Text>
      ) : !points ? (
        <Text as="p" size="2" color="gray">
          {t('repositories.usage.loading')}
        </Text>
      ) : points.length === 0 ? (
        <Text as="p" size="2" color="gray">
          {t('repositories.usage.empty')}
        </Text>
      ) : (
        <>
          <Text as="div" size="2" weight="medium">
            {t('repositories.usage.bytes')}
          </Text>
          <DownloadsChart
            days={points}
            value={(point) => point.bytes}
            format={formatBytes}
            ariaLabel={t('repositories.usage.bytes')}
          />
          <Text as="div" size="2" weight="medium">
            {t('repositories.usage.downloads')}
          </Text>
          <DownloadsChart
            days={points}
            value={(point) => point.downloads}
            format={formatNumber}
            ariaLabel={t('repositories.usage.downloads')}
          />
        </>
      )}
    </Flex>
  );
}
