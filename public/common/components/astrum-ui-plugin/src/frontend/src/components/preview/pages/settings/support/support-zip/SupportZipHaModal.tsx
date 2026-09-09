

import React, { useCallback } from 'react';
import { Dialog, Flex, Button } from '@radix-ui/themes';

import { SupportZipForm } from './SupportZipForm';
import { NodeInfo, SupportZipParams } from './types';

interface SupportZipHaModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  targetNode: NodeInfo | null;
  allNodes?: boolean;
  params: SupportZipParams;
  onParamChange: (name: keyof SupportZipParams, value: boolean | number) => void;
  onSubmit: () => void;
  disabled?: boolean;
}

export function SupportZipHaModal({
  open,
  onOpenChange,
  targetNode,
  allNodes = false,
  params,
  onParamChange,
  onSubmit,
  disabled = false,
}: SupportZipHaModalProps): React.ReactElement {
  const title = allNodes
    ? 'Generate Support ZIP for all nodes'
    : `Generate Support ZIP for ${targetNode?.hostname || targetNode?.nodeId || 'node'}`;

  const handleCancel = useCallback(() => {
    onOpenChange(false);
  }, [onOpenChange]);

  const handleConfirm = useCallback(() => {
    onSubmit();
  }, [onSubmit]);

  return (
    <Dialog.Root open={open} onOpenChange={(isOpen) => !isOpen && handleCancel()}>
      <Dialog.Content
        aria-describedby={undefined}
        style={{ maxWidth: 640 }}
        data-testid="support-zip-ha-modal"
      >
        <Dialog.Title>{title}</Dialog.Title>

        <Flex direction="column" gap="3" mt="3">
          <SupportZipForm
            params={params}
            onParamChange={onParamChange}
            onSubmit={handleConfirm}
            isHa={false}
            disabled={disabled}
            hideActions
          />
        </Flex>

        <Flex gap="3" mt="4" justify="end">
          <Button
            variant="outline"
            onClick={handleCancel}
            data-testid="support-zip-ha-modal-cancel"
            data-analytics-id="as-support-zip-modal-cancel"
          >
            Cancel
          </Button>
          <Button
            variant="solid"
            onClick={handleConfirm}
            disabled={disabled}
            data-testid="support-zip-ha-modal-confirm"
            data-analytics-id="as-support-zip-modal-confirm"
          >
            Generate
          </Button>
        </Flex>
      </Dialog.Content>
    </Dialog.Root>
  );
}

export default SupportZipHaModal;
