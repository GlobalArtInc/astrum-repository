
import React from 'react';
import {AsButton} from '@globalart/astrum-react-shared-components';
import {ExtJS, handleExtJsUnsavedChanges, showUnsavedChangesModal} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';
import SystemNotice from '../SystemNotice';
import { useRouter } from '@uirouter/react';

const {RECOVERY_MODE_ALERT: {LABEL, TEXT, BUTTON}} = UIStrings;
const supportRecoveryIdentifier = 'admin.support.recovery';

export default function RecoveryModeAlert() {
    const user = ExtJS.useUser();
    const isAdmin = user?.administrator;
    const recoveryModeEnabled = ExtJS.useState(() => ExtJS.state().getValue('recovery.mode.enabled'));
    const router = useRouter();

    const onClick = async () => {
      if (ExtJS.isExtJsRendered()) {
        const menuCtrl =
          window.Ext && Ext.getApplication && Ext.getApplication().getController
            ? Ext.getApplication().getController("Menu")
            : null;
        handleExtJsUnsavedChanges(menuCtrl, () => {
          router.stateService.go(supportRecoveryIdentifier);
        });
      } else {
        const hasReactDirty = window.dirty && window.dirty.length > 0;
        if (hasReactDirty) {
          const confirm = await showUnsavedChangesModal();
          if (confirm) {
            window.dirty = [];
            router.stateService.go(supportRecoveryIdentifier);
          }
        } else {
          router.stateService.go(supportRecoveryIdentifier);
        }
      }
    };

    if (!isAdmin || !recoveryModeEnabled) {
        return null;
    }

    return (
      <SystemNotice noticeLevel="info" title={LABEL} nonDismissable={true}>
        <div className="as-recovery-mode-alert">
          <div>{TEXT}</div>

          <AsButton
              onClick={onClick}
              data-analytics-id="as-recovery-mode-alert-btn"
              variant="primary"
          >
              {BUTTON}
          </AsButton>
        </div>
      </SystemNotice>
    );
}
