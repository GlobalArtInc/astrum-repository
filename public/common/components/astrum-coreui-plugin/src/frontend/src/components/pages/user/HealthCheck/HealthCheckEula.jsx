
import React from 'react';
import PropTypes from 'prop-types';
import {faHandsHelping} from "@fortawesome/free-solid-svg-icons";
import {ExtJS} from '@globalart/astrum-ui-plugin';
import {AsButton, AsFontAwesomeIcon} from '@globalart/astrum-react-shared-components';

import UIStrings from '../../../../constants/UIStrings';
import {useMachine} from "@xstate/react";
import HealthCheckEulaMachine from "./HealthCheckEulaMachine";

export default function HealthCheckEula(props) {
  const [state, send] = useMachine(HealthCheckEulaMachine, {
    actions: {
      eulaAccepted: () => props.onAccept(),
      eulaDeclined: () => props.onDecline()
    },
    devTools: true
  });

  const iframe = `<iframe style="width: 100%" src=${ExtJS.urlOf('/static/healthcheck-tos.html')}></iframe>`;

  function handleEulaAccept() {
    send({type: 'ACCEPT'});
  }

  function handleEulaDecline() {
    send({type: 'DECLINE'});
  }

  return <div>
    <header className="as-modal-header">
      <h2 className="as-h2">
        <AsFontAwesomeIcon icon={faHandsHelping} fixedWidth/>
        <span>{UIStrings.HEALTHCHECK_EULA.HEADER}</span>
      </h2>
    </header>

    <div className="as-modal-content">
      <p dangerouslySetInnerHTML={{__html: iframe}} />
    </div>

    <footer className="as-footer">
      <AsButton variant="primary" onClick={handleEulaAccept}>
        {UIStrings.HEALTHCHECK_EULA.BUTTONS.ACCEPT}
      </AsButton>
      <AsButton onClick={handleEulaDecline}>
        {UIStrings.HEALTHCHECK_EULA.BUTTONS.DECLINE}
      </AsButton>
    </footer>
  </div>
}

HealthCheckEula.propTypes = {
  onAccept: PropTypes.func,
  onDecline: PropTypes.func
}
