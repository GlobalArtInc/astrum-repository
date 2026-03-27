import { Button, Text } from '@radix-ui/themes';
import { Trash2 } from 'lucide-react';
import { useState } from 'react';
import { useT } from '../../../i18n';
import { ConfirmDialog } from '../../../ui';

export interface DeleteButtonProps {
  label: string;
  title: string;
  description: string;
  repository: string;
  onConfirm(): Promise<void>;
}

export function DeleteButton({ label, title, description, repository, onConfirm }: DeleteButtonProps) {
  const t = useT();
  const [open, setOpen] = useState(false);
  return (
    <>
      <Button variant="outline" color="red" onClick={() => setOpen(true)}>
        <Trash2 size={16} aria-hidden="true" />
        {label}
      </Button>
      <ConfirmDialog
        open={open}
        onOpenChange={setOpen}
        title={title}
        description={
          <>
            {description}
            <Text as="span" size="2" color="gray" style={{ display: 'block', marginTop: 8 }}>
              {t('browse.details.inRepository', { name: repository })}
            </Text>
          </>
        }
        confirmLabel={t('browse.details.delete')}
        danger
        onConfirm={onConfirm}
      />
    </>
  );
}
