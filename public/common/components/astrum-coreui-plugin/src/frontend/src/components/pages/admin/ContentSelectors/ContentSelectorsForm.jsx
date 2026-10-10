
import React from 'react';
import {useActor} from '@xstate/react';
import {faTrash} from '@fortawesome/free-solid-svg-icons';

import {FormUtils, ExtJS} from '@globalart/astrum-ui-plugin';
import {
  AsButton,
  AsFontAwesomeIcon,
  AsFormGroup,
  AsP,
  AsReadOnly,
  AsStatefulForm,
  AsTextInput
} from '@globalart/astrum-react-shared-components';

import {Section} from '@globalart/astrum-ui-plugin';

import ContentSelectorsPreview from './ContentSelectorsPreview';
import UIStrings from '../../../../constants/UIStrings';
import { isEmpty } from 'ramda';
import { DeleteConfirmationModal } from '@globalart/astrum-ui-plugin';

export default function ContentSelectorsForm({service, onDone}) {
  const stateMachine = useActor(service);
  const [state, send] = stateMachine;

  const {pristineData, data, loadError, showDeleteModal} = state.context;
  const hasData = data && !isEmpty(data);
  const canDelete = ExtJS.checkPermission('astrum:selectors:delete');
  const isEdit = Boolean(pristineData.name);

  function cancel() {
    onDone();
  }

  function confirmDelete() {
    send({type: 'CONFIRM_DELETE'});
  }
  
  return <>
    <Section className="as-content-selectors-form">
      <AsStatefulForm
          {...FormUtils.formProps(state, send)}
          onCancel={cancel}
          additionalFooterBtns={isEdit && canDelete &&
              <AsButton type="button" variant="tertiary" onClick={confirmDelete} data-analytics-id="as-content-selector-delete">
                <AsFontAwesomeIcon icon={faTrash}/>
                <span>{UIStrings.SETTINGS.DELETE_BUTTON_LABEL}</span>
              </AsButton>
          }
      >
        {hasData && !Boolean(pristineData.name) &&
            <AsFormGroup label={UIStrings.CONTENT_SELECTORS.NAME_LABEL} isRequired={!pristineData.name}>
              <AsTextInput
                  className="as-text-input--long"
                  {...FormUtils.fieldProps('name', state)}
                  disabled={pristineData.name}
                  onChange={FormUtils.handleUpdate('name', send)}/>
            </AsFormGroup>}
        {hasData && <>
          <AsReadOnly>
            <AsReadOnly.Label>{UIStrings.CONTENT_SELECTORS.TYPE_LABEL}</AsReadOnly.Label>
            <AsReadOnly.Data>{data.type?.toUpperCase()}</AsReadOnly.Data>
          </AsReadOnly>
          <AsFormGroup label={UIStrings.CONTENT_SELECTORS.DESCRIPTION_LABEL}>
            <AsTextInput
                className="as-text-input--long"
                {...FormUtils.fieldProps('description', state)}
                onChange={FormUtils.handleUpdate('description', send)}/>
          </AsFormGroup>
          <AsFormGroup label={UIStrings.CONTENT_SELECTORS.EXPRESSION_LABEL}
                       sublabel={UIStrings.CONTENT_SELECTORS.EXPRESSION_DESCRIPTION}
                       isRequired>
            <AsTextInput
                type="textarea"
                className="as-text-input--long"
                {...FormUtils.fieldProps('expression', state)}
                onChange={FormUtils.handleUpdate('expression', send)}
            />
          </AsFormGroup>

          <h4>{UIStrings.CONTENT_SELECTORS.EXPRESSION_EXAMPLES}</h4>
          <AsP>
            {UIStrings.CONTENT_SELECTORS.RAW_EXPRESSION_EXAMPLE_LABEL}:
            {' '}<code className="as-code">format == "raw"</code>
          </AsP>
          <AsP>
            {UIStrings.CONTENT_SELECTORS.MULTI_EXPRESSIONS_EXAMPLE_LABEL}:
            {' '}<code className="as-code">format == "maven2" and path =^ "/org"</code>
          </AsP>
        </>}
      </AsStatefulForm>
    </Section>
    {!loadError && <ContentSelectorsPreview type={data?.type} expression={data?.expression}/>}

    <DeleteConfirmationModal
      open={showDeleteModal}
      onClose={() => send('HIDE_DELETE_MODAL')}
      onConfirm={() => send('DELETE')}
      entityType="content selector"
      loading={state.matches('delete')}
    />
  </>;
}
