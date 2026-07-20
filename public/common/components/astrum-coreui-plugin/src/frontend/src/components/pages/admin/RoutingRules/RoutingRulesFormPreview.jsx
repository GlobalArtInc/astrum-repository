
import React from 'react';
import {NxButton, NxTextInput} from '@globalart/react-shared-components';

import UIStrings from '../../../../constants/UIStrings';

const {ROUTING_RULES} = UIStrings;

export default function RoutingRulesPreview({value, onChange, onTest}) {
  function handleEnter(event) {
    if (event === 'Enter') {
      onTest(event);
    }
  }

  return <div className="nx-form-group">
    <label htmlFor="path" className="nx-label"><span className="nx-label__text">{ROUTING_RULES.PATH_LABEL}</span></label>
    <div className="nx-sub-label">{ROUTING_RULES.PATH_DESCRIPTION}</div>
    <div className="nx-form-row">
      <div className="nx-form-group">
        {/* Ensure the button is at the correct height and prepend with a / */}
        <label className="nx-label as-path">
          <NxTextInput
            className="nx-text-input--long as-path-input"
            name="path"
            value={value}
            onChange={onChange}
            onKeyPress={handleEnter}
            validatable={false}
            isPristine={true}
          />
        </label>
      </div>
      <div className="nx-btn-bar">
        <NxButton variant="primary" onClick={onTest}>{ROUTING_RULES.TEST_BUTTON}</NxButton>
      </div>
    </div>
  </div>;
}
