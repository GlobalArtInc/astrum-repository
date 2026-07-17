
import React, {useState} from 'react';

import {NxButton, NxButtonBar, NxInfoAlert, useToggle, NxWarningAlert} from '@globalart/react-shared-components';
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
      <NxInfoAlert>
        <strong>{PENDING.STATUS_LABEL}&ensp;</strong>
        {PENDING.TEXT}
        <NxButtonBar>
          <NxButton variant="primary" onClick={() => setShowModal(true)}>
            {PENDING.FINALIZE_BUTTON}
          </NxButton>
        </NxButtonBar>
      </NxInfoAlert>
    }
    {state === 'needsUpgrade' && hasUser && hasPermission &&
      <UpgradeTriggerModal showModal={showModal} setShowModal={setShowModal}/>
    }

    {state === 'versionMismatch' && hasUser && hasPermission && isOpen &&
      <NxWarningAlert onClose={dismiss}>
        <strong>{WARN.LABEL}&ensp;</strong>
        {WARN.TEXT}
      </NxWarningAlert>
    }
    {state === 'astrumNeedsUpgrade1Year' && hasUser && hasPermission &&
      <NxInfoAlert>
        <strong>{PENDING.OLDER_LABEL}&ensp;</strong>
        {PENDING.YEAR_OLD_TEXT}
      </NxInfoAlert>
     }
     {state === 'astrumNeeds18MonthsUpgrade' && hasUser && hasPermission &&
      <NxInfoAlert>
        <strong>{PENDING.OLDER_LABEL}&ensp;</strong>
        {PENDING.TEXT_18_MONTHS}
      </NxInfoAlert>
     }
  </>
  )
}
