

import React, {useState} from 'react';
import PropTypes from "prop-types";
import axios from 'axios';

import {
  NxButton,
  NxLoadingSpinner, NxTextLink,
} from '@globalart/react-shared-components';
import {ExtJS} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';
import {UpgradeAlertFunctions} from './UpgradeAlertHelper';

import UpgradeTriggerModal from './UpgradeTriggerModal';
import SystemNotice from '../SystemNotice';

const {UPGRADE_ALERT: {PENDING, PROGRESS, ERROR, COMPLETE, WARN}} = UIStrings;

export default function UpgradeAlert({onClose}) {
  const hasUser = ExtJS.useState(UpgradeAlertFunctions.hasUser);
  const state = ExtJS.useState(UpgradeAlertFunctions.currentState);
  const message = ExtJS.useState(UpgradeAlertFunctions.message);
  const hasPermission = ExtJS.usePermission(UpgradeAlertFunctions.checkPermissions);
  const [showModal, setShowModal] = useState(false);

  if (!hasUser || !hasPermission) {
    return null;
  }

  switch (state) {
    case 'needsUpgrade':
      return (
        <SystemNotice noticeLevel="info" nonDismissable={true} title={PENDING.LABEL}>
          <div className="as-upgrade-alert-needs-upgrade">
              <div>{PENDING.TEXT}</div>

              <NxButton
                  data-analytics-id="as-upgrade-alert-finalize-upgrade-btn"
                  variant="primary"
                  onClick={() => setShowModal(true)}
              >
                {PENDING.FINALIZE_BUTTON}
              </NxButton>
          </div>

          <UpgradeTriggerModal showModal={showModal} setShowModal={setShowModal} />
        </SystemNotice>);
    case 'versionMismatch':
      return (
          <SystemNotice title={WARN.LABEL} noticeLevel="warning" onClose={dismissAlert}>
            {WARN.TEXT}
          </SystemNotice>);
    case 'astrumUpgradeInProgress':
      return (
        <SystemNotice noticeLevel="info" nonDismissable={true}>
          <NxLoadingSpinner><strong>{PROGRESS.LABEL}</strong></NxLoadingSpinner>
        </SystemNotice>);
    case 'astrumUpgradeError':
      return (
          <SystemNotice noticeLevel="error" title={ERROR.LABEL} onClose={onClose}>
            {message} {ERROR.TEXT}{" "}
            <NxTextLink external href={ERROR.CONTACT_SUPPORT.HREF}>
              {ERROR.CONTACT_SUPPORT.TEXT}
            </NxTextLink> for assistance.
          </SystemNotice>);
    case 'astrumUpgradeComplete':
      return (
          <SystemNotice
              noticeLevel="success"
              title={COMPLETE.LABEL}
              onClose={dismissAlert}
          >
            {COMPLETE.TEXT}
          </SystemNotice>);
    default:
      return null;
  }

  function dismissAlert() {
    axios.delete('service/rest/v1/clustered/upgrade-database-schema')
  }
}

UpgradeAlert.propTypes = {
  onClose: PropTypes.func
}
