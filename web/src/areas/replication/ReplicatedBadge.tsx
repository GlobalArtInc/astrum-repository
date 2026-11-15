import { Badge } from '@radix-ui/themes';
import { Link } from 'react-router';
import { useApi } from '../../api';
import { useCan } from '../../auth/state';
import { useT } from '../../i18n';
import { READ, REPLICATIONS, REPLICATIONS_ADDRESS, type Replication } from './api';

interface ReplicatedBadgeProps {
  repository: string;
}

export function ReplicatedBadge({ repository }: ReplicatedBadgeProps) {
  const t = useT();
  const allowed = useCan(READ);
  const replications = useApi<Replication[]>(allowed ? REPLICATIONS : null);
  const count = (replications.data ?? []).filter(
    (replication) => replication.enabled && replication.sourceRepository === repository,
  ).length;
  if (!allowed || count === 0) {
    return null;
  }
  return (
    <Link to={REPLICATIONS_ADDRESS}>
      <Badge color="gray" variant="soft" radius="small">
        {t('replication.replicatedTo.badge', { count })}
      </Badge>
    </Link>
  );
}
