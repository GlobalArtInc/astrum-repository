
import React from 'react';
import {useMachine} from '@xstate/react';
import {
  NxButton,
  NxH2,
  NxStatefulForm,
  NxTooltip,
  NxStatefulTransferList
} from '@globalart/react-shared-components';
import {FormUtils} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

import RealmsMachine from './RealmsMachine';
import RealmsRemovalModal from './RealmsRemovalModal';

const {REALMS: {CONFIGURATION: LABELS}, SETTINGS} = UIStrings;

export default function RealmsForm() {
  const [state, send] = useMachine(RealmsMachine, {devTools: true});
  const {data, isInvalid, isPristine, validationErrors} = state.context;

  const showLocalRealmRemovalModal = 
    state.matches('showLocalRealmRemovalModal');

  const discard = () => send({type: 'RESET'});
  const checkLocalRealmRemoval = () => send({type: 'CHECK_LOCAL_REALM_REMOVAL'});
  const save = () => send({type: 'SAVE'});
  const closeModal = () => send({type: 'CLOSE'});

  const available =
    data?.available?.map(({ id, name: displayName }) => ({
      id,
      displayName,
    })) || [];

  return (
    <NxStatefulForm
        {...FormUtils.formProps(state, send)}
        validationErrors={validationErrors?.active || validationErrors?.multipleSsoEnabled || FormUtils.saveTooltip({isPristine, isInvalid})}
        additionalFooterBtns={
          <NxTooltip title={FormUtils.discardTooltip({isPristine})}>
            <NxButton
              type="button"
              className={isPristine && 'disabled'}
              onClick={discard}
            >
              {SETTINGS.DISCARD_BUTTON_LABEL}
            </NxButton>
          </NxTooltip>
        }
        onSubmit={checkLocalRealmRemoval}
    >
      <>
        <NxH2>{LABELS.SUB_LABEL}</NxH2>
        <NxStatefulTransferList
            id="realms_select"
            availableItemsLabel={LABELS.AVAILABLE_TITLE}
            selectedItemsLabel={LABELS.SELECTED_TITLE}
            allItems={available}
            selectedItems={data.active || []}
            onChange={FormUtils.handleUpdate('active', send)}
            showMoveAll
            allowReordering
        />
      </>
      {showLocalRealmRemovalModal &&
        <RealmsRemovalModal onClose={closeModal} onConfirm={save} />
      }
    </NxStatefulForm>
  );
}
