
import React from 'react';
import {useActor} from '@xstate/react';
import {
  NxH2,
  NxTile,
  NxButton,
  NxButtonBar,
  NxFieldset,
  NxFormRow,
  NxModal,
  NxTooltip,
} from '@globalart/react-shared-components';

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
      <NxTile.Header>
        <NxTile.HeaderTitle>
          <NxH2>{LABELS.LABEL}</NxH2>
        </NxTile.HeaderTitle>
      </NxTile.Header>
      <NxTile.Content>
        <NxFieldset
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
          <NxFormRow>
            {isCapabilityActive && (
              <NxTooltip
                title={!canReset && UIStrings.PERMISSION_ERROR}
                placement="bottom"
              >
                <NxButtonBar>
                  <NxButton
                    type="button"
                    variant="tertiary"
                    onClick={reset}
                    className={!canReset ? 'disabled' : ''}
                  >
                    {LABELS.RESET_USER_TOKEN}
                  </NxButton>
                </NxButtonBar>
              </NxTooltip>
            )}
          </NxFormRow>
        </NxFieldset>
      </NxTile.Content>
      {confirmingAdminPassword && (
        <NxModal aria-labelledby="modal-form-header" variant="narrow">
          <ConfirmAdminPasswordForm
            actor={current.children.confirmAdminPasswordMachine}
            title={LABELS.RESET_USER_TOKEN}
            text={LABELS.TEXT}
            confirmLabel={LABELS.AUTHENTICATE}
          />
        </NxModal>
      )}
    </>
  );
}
