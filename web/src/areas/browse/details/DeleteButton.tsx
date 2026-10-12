import { Button } from '@radix-ui/themes';
import { Trash2 } from 'lucide-react';
import { type ReactNode, useState } from 'react';
import { ConfirmDialog } from '../../../ui';

export interface DeleteButtonProps {
  label: string;
  title: string;
  description: ReactNode;
  confirmLabel: string;
  onConfirm(): Promise<void>;
}

export function DeleteButton({ label, title, description, confirmLabel, onConfirm }: DeleteButtonProps) {
  const [open, setOpen] = useState(false);
  return (
    <>
      <Button variant="soft" color="red" onClick={() => setOpen(true)}>
        <Trash2 size={15} aria-hidden="true" />
        {label}
      </Button>
      <ConfirmDialog
        open={open}
        onOpenChange={setOpen}
        title={title}
        description={description}
        confirmLabel={confirmLabel}
        danger
        onConfirm={onConfirm}
      />
    </>
  );
}
