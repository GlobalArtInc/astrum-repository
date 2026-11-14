import { Button, Flex, Select, Text } from '@radix-ui/themes';
import { useState } from 'react';
import { useApi, withQuery } from '../../api';
import { useT } from '../../i18n';
import { DateTime, ErrorState, Page } from '../../ui';
import { usePagedSearch } from '../search/usePagedSearch';
import { COOLDOWN, type CooldownWait, REPOSITORIES, type Repository } from './api';
import { type ListColumn, ListTable } from './shared/AdminList';
import list from './shared/AdminList.module.css';

const ALL = '_all';

function packageName(wait: CooldownWait): string {
  return wait.namespace ? `${wait.namespace}/${wait.name}` : wait.name;
}

export default function CooldownPage() {
  const t = useT();
  const [chosen, setChosen] = useState(ALL);
  const repositories = useApi<Repository[]>(REPOSITORIES);
  const waits = usePagedSearch<CooldownWait>(
    withQuery(COOLDOWN, { repository: chosen === ALL ? null : chosen }),
    1,
  );
  const columns: ListColumn<CooldownWait>[] = [
    {
      id: 'repository',
      header: t('security.cooldown.repository'),
      cell: (wait) => wait.repository,
    },
    {
      id: 'package',
      header: t('security.cooldown.package'),
      cell: (wait) => packageName(wait),
    },
    {
      id: 'version',
      header: t('security.cooldown.version'),
      cell: (wait) => wait.version,
    },
    {
      id: 'published',
      header: t('security.cooldown.published'),
      cell: (wait) => <DateTime value={wait.publishedAt} />,
    },
    {
      id: 'availableAt',
      header: t('security.cooldown.availableFrom'),
      cell: (wait) => <DateTime value={wait.availableAt} />,
    },
    {
      id: 'state',
      header: t('security.cooldown.state'),
      cell: (wait) => (wait.blocked ? t('security.cooldown.blocked') : t('security.cooldown.reported')),
    },
    {
      id: 'refusals',
      header: t('security.cooldown.refusals'),
      cell: (wait) => wait.refusals,
    },
    {
      id: 'firstRefused',
      header: t('security.cooldown.firstRefused'),
      cell: (wait) => (wait.firstRefusedAt === null ? '—' : <DateTime value={wait.firstRefusedAt} />),
    },
  ];
  return (
    <Page
      title={t('security.cooldown.title')}
      description={t('security.cooldown.intro')}
      breadcrumbs={[
        { label: t('security.crumbs.settings'), to: '/settings' },
        { label: t('security.cooldown.title') },
      ]}
    >
      <Flex direction="column" gap="4">
        <Text as="p" size="2" color="gray">
          {t('security.cooldown.explain')}
        </Text>
        <Select.Root value={chosen} onValueChange={setChosen}>
          <Select.Trigger aria-label={t('security.cooldown.repository')} />
          <Select.Content position="popper">
            <Select.Item value={ALL}>{t('security.cooldown.allRepositories')}</Select.Item>
            {(repositories.data ?? []).map((repository) => (
              <Select.Item key={repository.name} value={repository.name}>
                {repository.name}
              </Select.Item>
            ))}
          </Select.Content>
        </Select.Root>
        {waits.error ? (
          <ErrorState error={waits.error} onRetry={waits.retry} />
        ) : (
          <ListTable
            plain
            label={t('security.cooldown.title')}
            columns={columns}
            rows={waits.items ?? []}
            loading={waits.loading}
            getRowId={(wait) =>
              `${wait.repository}|${wait.format}|${wait.namespace ?? ''}|${wait.name}|${wait.version}`
            }
            empty={<div className={list.count}>{t('security.cooldown.empty')}</div>}
          />
        )}
        {waits.hasMore && (
          <Flex justify="center">
            <Button
              variant="surface"
              color="gray"
              highContrast
              loading={waits.loadingMore}
              onClick={waits.loadMore}
            >
              {t('security.cooldown.more')}
            </Button>
          </Flex>
        )}
      </Flex>
    </Page>
  );
}
