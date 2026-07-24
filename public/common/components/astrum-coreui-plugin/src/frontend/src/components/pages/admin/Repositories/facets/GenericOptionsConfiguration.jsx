
import React from 'react';

import {FormUtils, useSimpleMachine} from '@globalart/astrum-ui-plugin';

import {
  NxFormGroup,
  NxLoadWrapper,
  NxTextInput,
  NxCheckbox,
  NxFieldset,
  NxFormSelect
} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {EDITOR} = UIStrings.REPOSITORIES;

export const ROUTING_RULES_URL = 'service/rest/internal/ui/routing-rules';

export default function GenericOptionsConfiguration({parentMachine}) {
  const {current, retry, isLoading} = useSimpleMachine({
    id: 'GenericOptionsConfigurationMachine',
    url: ROUTING_RULES_URL
  });

  const [currentParent, sendParent] = parentMachine;

  const {data: routingRules, error} = current.context;

  const isNegativeCacheEnabled = currentParent.context.data.negativeCache?.enabled;

  return (
    <>
      <h2 className="nx-h2">{EDITOR.OPTIONS_CAPTION}</h2>
      <NxLoadWrapper loading={isLoading} error={error} retryHandler={retry}>
        <NxFormGroup label={EDITOR.ROUTING_RULE_LABEL} className="as-form-group-routing-rule">
          <NxFormSelect
            {...FormUtils.selectProps('routingRule', currentParent)}
            onChange={FormUtils.handleUpdate('routingRule', sendParent)}
          >
            <option value="">{EDITOR.NONE_OPTION}</option>
            {routingRules?.map(({name}) => (
              <option key={name} value={name}>
                {name}
              </option>
            ))}
          </NxFormSelect>
        </NxFormGroup>

        <NxFieldset label={EDITOR.NEGATIVE_CACHE_LABEL} className="as-form-group-proxy-cache">
          <NxCheckbox
            {...FormUtils.checkboxProps('negativeCache.enabled', currentParent)}
            onChange={FormUtils.handleUpdate('negativeCache.enabled', sendParent)}
          >
            {EDITOR.ENABLED_CHECKBOX_DESCR}
          </NxCheckbox>
        </NxFieldset>

        <NxFormGroup
          label={EDITOR.NEGATIVE_CACHE_TTL_LABEL}
          sublabel={EDITOR.NEGATIVE_CACHE_TTL_SUBLABEL}
          isRequired
          className="as-form-group-negative-cache-ttl"
        >
          <NxTextInput
            {...FormUtils.fieldProps('negativeCache.timeToLive', currentParent)}
            onChange={FormUtils.handleUpdate('negativeCache.timeToLive', sendParent)}
            disabled={!isNegativeCacheEnabled}
            className="nx-text-input--short"
          />
        </NxFormGroup>
      </NxLoadWrapper>
    </>
  );
}
