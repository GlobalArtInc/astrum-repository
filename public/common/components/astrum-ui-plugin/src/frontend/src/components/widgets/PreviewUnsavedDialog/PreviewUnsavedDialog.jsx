

import React, { useLayoutEffect, useState } from 'react';
import { AlertDialog, Button, Flex } from '@radix-ui/themes';
import UIStrings from '../../../constants/UIStrings';

/**
 * Astrum One unsaved changes dialog for Preview UI router navigation.
 *
 * Registers window.showPreviewUnsavedDialog so the shared router guard
 * (createRouter.js) can prompt with a Radix dialog on preview.* routes.
 * Mounted once per app root (coreui and cloudui).
 */
let resolveUnsavedDialog = null;
let setUnsavedDialogOpen = () => {};

export default function PreviewUnsavedDialog() {
  const [open, setOpen] = useState(false);

  // useLayoutEffect so window.showPreviewUnsavedDialog exists before paint; UI-Router
  // onBefore can run in the same turn as a nav click and would otherwise fall back to
  // the classic modal.
  useLayoutEffect(() => {
    setUnsavedDialogOpen = setOpen;
    window.showPreviewUnsavedDialog = () => {
      return new Promise((resolve) => {
        resolveUnsavedDialog = resolve;
        setOpen(true);
      });
    };
    return () => {
      delete window.showPreviewUnsavedDialog;
    };
  }, []);

  const handleLeave = () => {
    if (resolveUnsavedDialog) {
      resolveUnsavedDialog(true);
      resolveUnsavedDialog = null;
    }
    // Don't call setOpen(false) - AlertDialog.Action closes automatically
  };

  const handleStay = () => {
    if (resolveUnsavedDialog) {
      resolveUnsavedDialog(false);
      resolveUnsavedDialog = null;
    }
    // Don't call setOpen(false) - AlertDialog.Cancel closes automatically
  };

  return (
    <AlertDialog.Root open={open} onOpenChange={setOpen}>
      <AlertDialog.Content maxWidth="450px">
        <AlertDialog.Title>{UIStrings.UNSAVED_CHANGES.TITLE}</AlertDialog.Title>
        <AlertDialog.Description size="2">
          {UIStrings.UNSAVED_CHANGES.LEAVE_TEXT}
        </AlertDialog.Description>
        <Flex gap="3" mt="4" justify="end">
          <AlertDialog.Cancel>
            <Button variant="soft" color="gray" onClick={handleStay}>{UIStrings.UNSAVED_CHANGES.STAY}</Button>
          </AlertDialog.Cancel>
          <AlertDialog.Action>
            <Button variant="solid" color="red" onClick={handleLeave}>{UIStrings.UNSAVED_CHANGES.LEAVE}</Button>
          </AlertDialog.Action>
        </Flex>
      </AlertDialog.Content>
    </AlertDialog.Root>
  );
}
