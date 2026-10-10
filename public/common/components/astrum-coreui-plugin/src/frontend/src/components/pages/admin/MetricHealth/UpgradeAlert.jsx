
import React, {useState} from 'react';

import {AsButton, AsButtonBar, AsInfoAlert, useToggle, AsWarningAlert} from '@globalart/react-shared-components';
import {ExtJS} from '@globalart/astrum-ui-plugin';
import {UpgradeAlertFunctions} from '../../../widgets/SystemStatusAlerts/UpgradeAlert/UpgradeAlertHelper';
import UIStrings from '../../../../constants/UIStrings';
import UpgradeTriggerModal from '../../../widgets/SystemStatusAlerts/UpgradeAlert/UpgradeTriggerModal';
const {UPGRADE_ALERT: {PENDING, WARN}} = UIStrings;

export default function UpgradeAlert() {

  const hasUser = ExtJS.useState(UpgradeAlertFunctions.hasUser);
  const state = ExtJS.useState(UpgradeAlertFunctions.currentState);
  const hasPermission = ExtJS.usePermission(UpgradeAlertFunctions.checkPermissions);
  const [isOpen, dismiss] = useToggle(true);
  const [showModal, setShowModal] = useState(false);

  return (
  <>
    {state === 'needsUpgrade' && hasUser && hasPermission &&
      <AsInfoAlert>
        <strong>{PENDING.STATUS_LABEL}&ensp;</strong>
        {PENDING.TEXT}
        <AsButtonBar>
          <AsButton variant="primary" onClick={() => setShowModal(true)}>
            {PENDING.FINALIZE_BUTTON}
          </AsButton>
        </AsButtonBar>
      </AsInfoAlert>
    }
    {state === 'needsUpgrade' && hasUser && hasPermission &&
      <UpgradeTriggerModal showModal={showModal} setShowModal={setShowModal}/>
    }

    {state === 'versionMismatch' && hasUser && hasPermission && isOpen &&
      <AsWarningAlert onClose={dismiss}>
        <strong>{WARN.LABEL}&ensp;</strong>
        {WARN.TEXT}
      </AsWarningAlert>
    }
    {state === 'astrumNeedsUpgrade1Year' && hasUser && hasPermission &&
      <AsInfoAlert>
        <strong>{PENDING.OLDER_LABEL}&ensp;</strong>
        {PENDING.YEAR_OLD_TEXT}
      </AsInfoAlert>
     }
     {state === 'astrumNeeds18MonthsUpgrade' && hasUser && hasPermission &&
      <AsInfoAlert>
        <strong>{PENDING.OLDER_LABEL}&ensp;</strong>
        {PENDING.TEXT_18_MONTHS}
      </AsInfoAlert>
     }
  </>
  )
}
