

import React from 'react';
import {faTrash} from '@fortawesome/free-solid-svg-icons';

import {
  AsButton,
  AsButtonBar,
  AsFontAwesomeIcon,
  AsFormGroup,
  AsFormRow,
  AsTextInput
} from '@globalart/react-shared-components';

import UIStrings from '../../../../constants/UIStrings';

const {ROUTING_RULES} = UIStrings;

export default function RoutingRuleMatcherRow({label, showRemoveButton, onRemove, ...props}) {
  return <AsFormRow className="routing-rule-matcher">
    {/*Use an empty label to avoid error with no label being provided*/}
    <AsFormGroup label={label}>
      <AsTextInput
          className="as-text-input--long"
          label={label}
          {...props} />
    </AsFormGroup>
    {showRemoveButton &&
        <AsButtonBar>
          <AsButton
              type="button"
              title={ROUTING_RULES.FORM.DELETE_MATCHER_BUTTON}
              onClick={onRemove}>
            <AsFontAwesomeIcon icon={faTrash}/>
          </AsButton>
        </AsButtonBar>}
  </AsFormRow>;
}
