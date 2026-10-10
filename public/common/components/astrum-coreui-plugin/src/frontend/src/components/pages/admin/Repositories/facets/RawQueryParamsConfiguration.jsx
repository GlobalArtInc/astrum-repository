
import React, {useState} from 'react';

import {ExtJS, FormUtils} from '@globalart/astrum-ui-plugin';

import {
  AsButton,
  AsCheckbox,
  AsFieldset,
  AsFontAwesomeIcon,
  AsFormGroup,
  AsFormRow,
  AsInfoAlert,
  AsList,
  AsTextInput,
  AsWarningAlert
} from '@globalart/astrum-react-shared-components';

import {faPlusCircle, faTrashAlt} from '@fortawesome/free-solid-svg-icons';

import UIStrings from '../../../../../constants/UIStrings';

const {RAW: {QUERY_PARAMS}} = UIStrings.REPOSITORIES.EDITOR;

export default function RawQueryParamsConfiguration({parentMachine}) {
  if (!ExtJS.state()?.getValue('rawQueryParamsForwardingEnabled')) {
    return null;
  }

  const [parentState, sendParent] = parentMachine;

  const forwardQueryParameters = parentState.context.data?.raw?.forwardQueryParameters ?? false;
  const excludedQueryParameters = parentState.context.data?.raw?.excludedQueryParameters ?? [];
  const [newExcludedParameter, setNewExcludedParameter] = useState('');

  const handleToggle = () => {
    sendParent({
      type: 'UPDATE',
      name: 'raw.forwardQueryParameters',
      value: !forwardQueryParameters
    });
    // Don't clear exclusions - let user preserve their list
  };

  const handleAddExcludedParameter = () => {
    const trimmed = newExcludedParameter.trim();

    if (trimmed === '') {
      return;
    }

    const isDuplicate = excludedQueryParameters.some(
      (param) => param.toLowerCase() === trimmed.toLowerCase()
    );

    if (isDuplicate) {
      return;
    }

    sendParent({
      type: 'UPDATE',
      name: 'raw.excludedQueryParameters',
      value: [...excludedQueryParameters, trimmed]
    });

    setNewExcludedParameter('');
  };

  const handleRemoveExcludedParameter = (index) => {
    const updated = excludedQueryParameters.filter((_, i) => i !== index);
    sendParent({
      type: 'UPDATE',
      name: 'raw.excludedQueryParameters',
      value: updated
    });
  };

  const handleKeyDown = (event) => {
    if (event.key === 'Enter') {
      event.preventDefault();
      handleAddExcludedParameter();
    }
  };

  return (
    <AsFieldset
      label={QUERY_PARAMS.CAPTION}
      sublabel={QUERY_PARAMS.SUBLABEL}
      className="as-form-group-query-params"
    >
      <AsCheckbox
        {...FormUtils.checkboxProps('raw.forwardQueryParameters', parentState)}
        onChange={handleToggle}
        isChecked={forwardQueryParameters}
      >
        {QUERY_PARAMS.CHECKBOX}
      </AsCheckbox>
      <div className="as-sub-label as-query-params-description">
        {forwardQueryParameters ? QUERY_PARAMS.DESCRIPTION_ENABLED : QUERY_PARAMS.DESCRIPTION}
      </div>

      {forwardQueryParameters && (
        <>
          <AsWarningAlert>
            <p><strong>{QUERY_PARAMS.CACHING_WARNING_TITLE}</strong></p>
            <p>{QUERY_PARAMS.CACHING_WARNING_CONTENT}</p>
          </AsWarningAlert>

          <AsInfoAlert>
            <p><strong>{QUERY_PARAMS.EXAMPLES_TITLE}</strong></p>
            <ul>
              {QUERY_PARAMS.EXAMPLES.map((example, index) => (
                <li key={index}>{example}</li>
              ))}
            </ul>

            <p><strong>{QUERY_PARAMS.USE_CASES_TITLE}</strong></p>
            <ul>
              {QUERY_PARAMS.USE_CASES.map((useCase, index) => (
                <li key={index}>{useCase}</li>
              ))}
            </ul>
          </AsInfoAlert>

          <AsFieldset
            label={QUERY_PARAMS.EXCLUSION_LABEL}
            sublabel={QUERY_PARAMS.EXCLUSION_SUBLABEL}
            className="as-query-params-exclusions"
          >
            <AsFormRow>
              <>
                <AsFormGroup label="">
                  <AsTextInput
                    className="as-text-input--long"
                    value={newExcludedParameter}
                    onChange={setNewExcludedParameter}
                    onKeyDown={handleKeyDown}
                    placeholder={QUERY_PARAMS.EXCLUSION_PLACEHOLDER}
                  />
                </AsFormGroup>
                <AsButton
                  variant="icon-only"
                  title={newExcludedParameter.trim() ? QUERY_PARAMS.ADD_EXCLUSION : undefined}
                  onClick={handleAddExcludedParameter}
                  disabled={!newExcludedParameter.trim()}
                  type="button"
                >
                  <AsFontAwesomeIcon icon={faPlusCircle} />
                </AsButton>
              </>
            </AsFormRow>

            {excludedQueryParameters.length > 0 && (
              <div className="as-scrollable as-query-params-exclusion-list">
                <AsList>
                  {excludedQueryParameters.map((param, index) => (
                    <AsList.Item key={index}>
                      <AsList.Text>{param}</AsList.Text>
                      <AsList.Actions>
                        <AsButton
                          variant="icon-only"
                          title={QUERY_PARAMS.REMOVE_EXCLUSION}
                          onClick={() => handleRemoveExcludedParameter(index)}
                          type="button"
                        >
                          <AsFontAwesomeIcon icon={faTrashAlt} />
                        </AsButton>
                      </AsList.Actions>
                    </AsList.Item>
                  ))}
                </AsList>
              </div>
            )}
          </AsFieldset>
        </>
      )}
    </AsFieldset>
  );
}
