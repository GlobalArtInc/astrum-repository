import { Badge, Button, Flex } from '@radix-ui/themes';
import { Circle, Database, DatabaseZap, FolderTree, Power, Trash2 } from 'lucide-react';
import type { ReactNode } from 'react';
import { Link } from 'react-router';
import type { RepositoryType } from '../../../api';
import { useT } from '../../../i18n';
import { formatName } from '../../../ui';
import { browseAddress, type UiRepository } from '../api';
import styles from './RepositoryHeader.module.css';
import { canInvalidate, canRebuild } from './useRepositoryActions';

const TYPE_LABELS: Record<RepositoryType, string> = {
  hosted: 'ui.repositoryType.hosted',
  proxy: 'ui.repositoryType.proxy',
  group: 'ui.repositoryType.group',
};

export function OnlineDot({ online }: { online: boolean }) {
  return (
    <Circle
      size={8}
      aria-hidden="true"
      fill="currentColor"
      className={online ? styles.dotOnline : styles.dotOffline}
    />
  );
}

export function RepositoryBadges({ repository, online }: { repository: UiRepository; online: boolean }) {
  const t = useT();
  return (
    <span className={styles.badges}>
      <Badge color="gray" variant="soft" radius="small" className={styles.badge}>
        <OnlineDot online={online} />
        {online ? t('repositories.status.online') : t('repositories.status.offline')}
      </Badge>
      <Badge color="gray" variant="soft" radius="small" className={styles.badge}>
        {t(TYPE_LABELS[repository.type])}
      </Badge>
      <Badge color="gray" variant="soft" radius="small" className={styles.badge}>
        {formatName(repository.format)}
      </Badge>
    </span>
  );
}

export function RepositoryButtons({
  repository,
  online,
  busy,
  onRebuild,
  onInvalidate,
  onToggleOnline,
  onDelete,
  extra,
  tone = 'gray',
}: {
  repository: UiRepository;
  online: boolean;
  busy?: boolean;
  tone?: 'gray' | 'accent';
  onRebuild(): void;
  onInvalidate(): void;
  onToggleOnline(): void;
  onDelete(): void;
  extra?: ReactNode;
}) {
  const t = useT();
  const flags = repository.permissions;
  const color = tone === 'gray' ? 'gray' : undefined;
  return (
    <Flex gap="2" wrap="wrap" align="center" justify="end">
      {(flags.browse || flags.read) && (
        <Button asChild variant="soft" color={color}>
          <Link to={browseAddress(repository.name)}>
            <FolderTree size={16} aria-hidden="true" />
            {t('repositories.edit.browse')}
          </Link>
        </Button>
      )}
      {canRebuild(repository) && (
        <Button variant="soft" color={color} onClick={onRebuild}>
          <Database size={16} aria-hidden="true" />
          {t('repositories.actions.rebuild')}
        </Button>
      )}
      {canInvalidate(repository) && (
        <Button variant="soft" color={color} onClick={onInvalidate}>
          <DatabaseZap size={16} aria-hidden="true" />
          {t('repositories.actions.invalidate')}
        </Button>
      )}
      {flags.adminEdit && (
        <Button variant="soft" color={color} disabled={busy} onClick={onToggleOnline}>
          <Power size={16} aria-hidden="true" />
          {online ? t('repositories.actions.takeOffline') : t('repositories.actions.bringOnline')}
        </Button>
      )}
      {flags.adminDelete && (
        <Button variant="soft" color="red" onClick={onDelete}>
          <Trash2 size={16} aria-hidden="true" />
          {t('repositories.actions.delete')}
        </Button>
      )}
      {extra}
    </Flex>
  );
}
