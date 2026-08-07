import React from 'react';
import {Button, Dialog, Flex, IconButton, ScrollArea} from '@radix-ui/themes';
import {X} from 'lucide-react';

import './MobileFilterDrawer.scss';

export interface MobileFilterDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  onClearAll?: () => void;
  clearAllLabel?: string;
  children?: React.ReactNode;
}

export function MobileFilterDrawer({
  isOpen,
  onClose,
  title,
  onClearAll,
  clearAllLabel = 'Clear all',
  children
}: MobileFilterDrawerProps) {
  const handleOpenChange = (open: boolean) => {
    if (!open) {
      onClose();
    }
  };

  return (
    <Dialog.Root open={isOpen} onOpenChange={handleOpenChange}>
      <Dialog.Content className="mobile-filter-drawer" aria-describedby={undefined}>
        <Flex align="center" justify="between" gap="3" className="mobile-filter-drawer__header">
          <Dialog.Title size="4" mb="0">
            {title}
          </Dialog.Title>
          <Flex align="center" gap="3">
            {onClearAll && (
              <Button type="button" variant="ghost" size="2" onClick={onClearAll}>
                {clearAllLabel}
              </Button>
            )}
            <Dialog.Close>
              <IconButton type="button" variant="ghost" size="2" aria-label="Close">
                <X size={18} />
              </IconButton>
            </Dialog.Close>
          </Flex>
        </Flex>
        <ScrollArea type="auto" scrollbars="vertical" className="mobile-filter-drawer__body">
          <div className="mobile-filter-drawer__content">{children}</div>
        </ScrollArea>
      </Dialog.Content>
    </Dialog.Root>
  );
}

export default MobileFilterDrawer;
