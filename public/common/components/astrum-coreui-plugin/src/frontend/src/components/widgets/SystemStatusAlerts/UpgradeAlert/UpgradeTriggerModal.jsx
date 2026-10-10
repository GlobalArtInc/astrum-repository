

import React from 'react';
import {AsModal, AsH2, AsButton, AsButtonBar, AsFooter, AsFieldset, AsCheckbox, useToggle} from '@globalart/astrum-react-shared-components';
import {useMachine} from '@xstate/react';

import UIStrings from '../../../../constants/UIStrings';
import UpgradeTriggerMachine from './UpgradeTriggerMachine';
import './UpgradeTriggerModal.scss';

const {TRIGGER_MODAL: { TITLE, DESCRIPTION, CHECKBOX_LABEL, CANCEL, CONTINUE}} = UIStrings;

export default function UpgradeTriggerModal({showModal, setShowModal}) {
  const [state, send] = useMachine(UpgradeTriggerMachine, {devTools: true});
  const [isChecked, onCheck] = useToggle(false);

  function saveModal() {
    send('SAVE_MODAL');
    setShowModal(false);
  }

  return <>
    {showModal && 
    <AsModal variant="narrow" className="upgrade-trigger-modal"
        onCancel={() => setShowModal(false)} aria-labelledby="zero-downtime-trigger-upgrade-modal">
      <header className="as-modal-header">
        <AsH2 id="zero-downtime-trigger-upgrade-modal">{TITLE}</AsH2>
      </header>
      <section className="zdt-trigger-description">
        {DESCRIPTION}
      </section>
      <section className="trigger-upgrade-fieldset">
        <AsFieldset label="Acknowledge">
          <AsCheckbox onChange={onCheck} isChecked={isChecked} className="upgrade-trigger-checkbox">
            {CHECKBOX_LABEL}
          </AsCheckbox>
        </AsFieldset>
      </section>
      <AsFooter>
        <AsButtonBar>
          <AsButton onClick={() => setShowModal(false)}>{CANCEL}</AsButton>
          <AsButton variant="primary" disabled={!isChecked} onClick={() => saveModal()}>{CONTINUE}</AsButton>
        </AsButtonBar>
      </AsFooter>
    </AsModal>
    }
  </>
}
