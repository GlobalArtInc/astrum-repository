import { Button } from '@radix-ui/themes';
import { ArrowRightLeft } from 'lucide-react';
import { useState } from 'react';
import { useT } from '../../../i18n';
import type { PromotionItem } from './model';
import { PromoteDialog } from './PromoteDialog';
import { usePromotionTargets } from './targets';

export interface PromoteRepositoryProps {
  source: string;
  format: string;
  items: PromotionItem[];
}

export function PromoteRepository({ source, format, items }: PromoteRepositoryProps) {
  const t = useT();
  const [open, setOpen] = useState(false);
  const targets = usePromotionTargets([source]).get(source) ?? [];
  if (items.length === 0 || targets.length === 0) {
    return null;
  }
  return (
    <>
      <Button variant="soft" size="1" onClick={() => setOpen(true)}>
        <ArrowRightLeft size={14} aria-hidden="true" />
        {t('browse.promote.open')}
      </Button>
      <PromoteDialog
        open={open}
        onOpenChange={setOpen}
        source={source}
        format={format}
        items={items}
        targets={targets}
      />
    </>
  );
}
