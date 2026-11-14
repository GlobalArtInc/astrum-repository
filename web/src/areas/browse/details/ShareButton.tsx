import { Button } from '@radix-ui/themes';
import { Link2 } from 'lucide-react';
import { useState } from 'react';
import { useSession } from '../../../auth/state';
import { useT } from '../../../i18n';
import { SHARE_CREATE } from '../../security/share-links/model';
import type { RepositoryItem } from '../types';
import { ShareDialog } from './ShareDialog';

export interface ShareButtonProps {
  repository: RepositoryItem;
  path: string;
  prefix: boolean;
}

export function ShareButton({ repository, path, prefix }: ShareButtonProps) {
  const t = useT();
  const { user, can } = useSession();
  const [open, setOpen] = useState(false);
  if (!user || !can(SHARE_CREATE) || repository.type !== 'hosted' || repository.format === 'docker') {
    return null;
  }
  return (
    <>
      <Button variant="soft" color="gray" onClick={() => setOpen(true)}>
        <Link2 size={16} aria-hidden="true" />
        {prefix ? t('browse.share.folderButton') : t('browse.share.button')}
      </Button>
      <ShareDialog
        open={open}
        onOpenChange={setOpen}
        repository={repository.name}
        path={path}
        prefix={prefix}
      />
    </>
  );
}
