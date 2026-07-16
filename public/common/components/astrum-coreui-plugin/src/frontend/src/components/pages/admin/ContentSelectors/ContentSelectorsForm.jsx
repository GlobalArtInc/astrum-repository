
import React from 'react';
import {useActor} from '@xstate/react';
import {faTrash} from '@fortawesome/free-solid-svg-icons';

import {FormUtils, ExtJS} from '@globalart/astrum-ui-plugin';
import {
  NxButton,
  NxFontAwesomeIcon,
  NxFormGroup,
  NxP,
  NxReadOnly,
  NxStatefulForm,
  NxTextInput
} from '@globalart/react-shared-components';

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
      <NxStatefulForm
          {...FormUtils.formProps(state, send)}
          onCancel={cancel}
          additionalFooterBtns={isEdit && canDelete &&
              <NxButton type="button" variant="tertiary" onClick={confirmDelete} data-analytics-id="as-content-selector-delete">
                <NxFontAwesomeIcon icon={faTrash}/>
                <span>{UIStrings.SETTINGS.DELETE_BUTTON_LABEL}</span>
              </NxButton>
          }
      >
        {hasData && !Boolean(pristineData.name) &&
            <NxFormGroup label={UIStrings.CONTENT_SELECTORS.NAME_LABEL} isRequired={!pristineData.name}>
              <NxTextInput
                  className="nx-text-input--long"
                  {...FormUtils.fieldProps('name', state)}
                  disabled={pristineData.name}
                  onChange={FormUtils.handleUpdate('name', send)}/>
            </NxFormGroup>}
        {hasData && <>
          <NxReadOnly>
            <NxReadOnly.Label>{UIStrings.CONTENT_SELECTORS.TYPE_LABEL}</NxReadOnly.Label>
            <NxReadOnly.Data>{data.type?.toUpperCase()}</NxReadOnly.Data>
          </NxReadOnly>
          <NxFormGroup label={UIStrings.CONTENT_SELECTORS.DESCRIPTION_LABEL}>
            <NxTextInput
                className="nx-text-input--long"
                {...FormUtils.fieldProps('description', state)}
                onChange={FormUtils.handleUpdate('description', send)}/>
          </NxFormGroup>
          <NxFormGroup label={UIStrings.CONTENT_SELECTORS.EXPRESSION_LABEL}
                       sublabel={UIStrings.CONTENT_SELECTORS.EXPRESSION_DESCRIPTION}
                       isRequired>
            <NxTextInput
                type="textarea"
                className="nx-text-input--long"
                {...FormUtils.fieldProps('expression', state)}
                onChange={FormUtils.handleUpdate('expression', send)}
            />
          </NxFormGroup>

          <h4>{UIStrings.CONTENT_SELECTORS.EXPRESSION_EXAMPLES}</h4>
          <NxP>
            {UIStrings.CONTENT_SELECTORS.RAW_EXPRESSION_EXAMPLE_LABEL}:
            {' '}<code className="nx-code">format == "raw"</code>
          </NxP>
          <NxP>
            {UIStrings.CONTENT_SELECTORS.MULTI_EXPRESSIONS_EXAMPLE_LABEL}:
            {' '}<code className="nx-code">format == "maven2" and path =^ "/org"</code>
          </NxP>
        </>}
      </NxStatefulForm>
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
