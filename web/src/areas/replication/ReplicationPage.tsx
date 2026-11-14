import { useT } from '../../i18n';
import { Page, QueryTabs } from '../../ui';
import { RemotesPanel } from './RemotesPanel';
import { ReplicationsPanel } from './ReplicationsPanel';

export default function ReplicationPage() {
  const t = useT();
  return (
    <Page
      title={t('replication.page.title')}
      description={t('replication.page.description')}
      breadcrumbs={[
        { label: t('replication.crumbs.settings'), to: '/settings' },
        { label: t('replication.page.title') },
      ]}
    >
      <QueryTabs
        tabs={[
          { value: 'remotes', label: t('replication.tabs.remotes'), content: <RemotesPanel /> },
          {
            value: 'replications',
            label: t('replication.tabs.replications'),
            content: <ReplicationsPanel />,
          },
        ]}
      />
    </Page>
  );
}
