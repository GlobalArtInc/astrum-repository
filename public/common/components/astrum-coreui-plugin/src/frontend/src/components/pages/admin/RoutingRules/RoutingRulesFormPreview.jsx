
import React from 'react';
import {AsButton, AsTextInput} from '@globalart/astrum-react-shared-components';

import UIStrings from '../../../../constants/UIStrings';

const {ROUTING_RULES} = UIStrings;

export default function RoutingRulesPreview({value, onChange, onTest}) {
  function handleEnter(event) {
    if (event === 'Enter') {
      onTest(event);
    }
  }

  return <div className="as-form-group">
    <label htmlFor="path" className="as-label"><span className="as-label__text">{ROUTING_RULES.PATH_LABEL}</span></label>
    <div className="as-sub-label">{ROUTING_RULES.PATH_DESCRIPTION}</div>
    <div className="as-form-row">
      <div className="as-form-group">
        {/* Ensure the button is at the correct height and prepend with a / */}
        <label className="as-label as-path">
          <AsTextInput
            className="as-text-input--long as-path-input"
            name="path"
            value={value}
            onChange={onChange}
            onKeyPress={handleEnter}
            validatable={false}
            isPristine={true}
          />
        </label>
      </div>
      <div className="as-btn-bar">
        <AsButton variant="primary" onClick={onTest}>{ROUTING_RULES.TEST_BUTTON}</AsButton>
      </div>
    </div>
  </div>;
}
