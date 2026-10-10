
import React from 'react';
import {useActor} from '@xstate/react';
import {
  AsH2,
  AsTile,
  AsButton,
  AsButtonBar,
  AsFieldset,
  AsFormRow,
  AsModal,
  AsTooltip,
} from '@globalart/astrum-react-shared-components';

import UIStrings from '../../../../constants/UIStrings';

import {ExtJS} from '@globalart/astrum-ui-plugin';
const {
  USERS: {TOKEN: LABELS},
} = UIStrings;

import ConfirmAdminPasswordForm from './ConfirmAdminPasswordForm';

export default function UsersToken({service}) {
  const [current, send] = useActor(service);
  const activeCapabilities =
    ExtJS.state().getValue('capabilityActiveTypes') || [];
  const isCapabilityActive = activeCapabilities.includes('usertoken');
  const confirmingAdminPassword = current.matches(
    'resetToken.confirmAdminPassword'
  );
  const canReset = ExtJS.checkPermission('astrum:usertoken-settings:update');

  const reset = () => {
    if (canReset) {
      send({type: 'RESET_TOKEN'});
    }
  };

  return (
    <>
      <AsTile.Header>
        <AsTile.HeaderTitle>
          <AsH2>{LABELS.LABEL}</AsH2>
        </AsTile.HeaderTitle>
      </AsTile.Header>
      <AsTile.Content>
        <AsFieldset
          label={
            isCapabilityActive ? LABELS.RESET_USER_TOKEN : LABELS.USER_TOKEN
          }
          sublabel={
            isCapabilityActive
              ? LABELS.ACTIVE_FEATURE
              : LABELS.REQUIRE_ENABLE_FEATURE
          }
          isRequired
        >
          <AsFormRow>
            {isCapabilityActive && (
              <AsTooltip
                title={!canReset && UIStrings.PERMISSION_ERROR}
                placement="bottom"
              >
                <AsButtonBar>
                  <AsButton
                    type="button"
                    variant="tertiary"
                    onClick={reset}
                    className={!canReset ? 'disabled' : ''}
                  >
                    {LABELS.RESET_USER_TOKEN}
                  </AsButton>
                </AsButtonBar>
              </AsTooltip>
            )}
          </AsFormRow>
        </AsFieldset>
      </AsTile.Content>
      {confirmingAdminPassword && (
        <AsModal aria-labelledby="modal-form-header" variant="narrow">
          <ConfirmAdminPasswordForm
            actor={current.children.confirmAdminPasswordMachine}
            title={LABELS.RESET_USER_TOKEN}
            text={LABELS.TEXT}
            confirmLabel={LABELS.AUTHENTICATE}
          />
        </AsModal>
      )}
    </>
  );
}
