

import React from 'react';
import {faTrash} from '@fortawesome/free-solid-svg-icons';

import {
  NxButton,
  NxButtonBar,
  NxFontAwesomeIcon,
  NxFormGroup,
  NxFormRow,
  NxTextInput
} from '@globalart/react-shared-components';

import UIStrings from '../../../../constants/UIStrings';

const {ROUTING_RULES} = UIStrings;

export default function RoutingRuleMatcherRow({label, showRemoveButton, onRemove, ...props}) {
  return <NxFormRow className="routing-rule-matcher">
    {/*Use an empty label to avoid error with no label being provided*/}
    <NxFormGroup label={label}>
      <NxTextInput
          className="nx-text-input--long"
          label={label}
          {...props} />
    </NxFormGroup>
    {showRemoveButton &&
        <NxButtonBar>
          <NxButton
              type="button"
              title={ROUTING_RULES.FORM.DELETE_MATCHER_BUTTON}
              onClick={onRemove}>
            <NxFontAwesomeIcon icon={faTrash}/>
          </NxButton>
        </NxButtonBar>}
  </NxFormRow>;
}
