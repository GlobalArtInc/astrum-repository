import { useT } from '../../i18n';
import { Page } from '../../ui';
import { RemotesPanel } from './RemotesPanel';

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
      <RemotesPanel />
    </Page>
  );
}
