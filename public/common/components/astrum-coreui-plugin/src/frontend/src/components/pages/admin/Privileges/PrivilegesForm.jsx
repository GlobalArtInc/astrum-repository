
import React, { useEffect } from 'react';
import {useActor} from '@xstate/react';

import {
  FormUtils,
  ValidationUtils,
  ExtJS,
  FormFieldsFactory,
} from '@globalart/astrum-ui-plugin';
import {
  NxButton,
  NxH2,
  NxFieldset,
  NxFontAwesomeIcon,
  NxFormGroup,
  NxFormSelect,
  NxStatefulForm,
  NxTextInput,
  NxTile
} from '@globalart/react-shared-components';

import {faTrash} from '@fortawesome/free-solid-svg-icons';

import UIStrings from '../../../../constants/UIStrings';
import { TYPES as TYPE_IDS } from './PrivilegesHelper';

const {PRIVILEGES: {FORM: LABELS}} = UIStrings;

export default function PrivilegesForm({itemId, service, onDone}) {
  const [current, send] = useActor(service);

  const {
    data: {type, readOnly},
    isTouched,
    types = {},
    validationErrors: {actionSelected}
  } = current.context;

  const isCreate = ValidationUtils.isBlank(itemId);
  const isEdit = !isCreate;
  const hasDeletePermissions = ExtJS.checkPermission('astrum:privileges:delete');
  const canDelete = hasDeletePermissions && !readOnly;
  const isTypeSelected = Boolean(type);
  const fields = types[type]?.formFields || [];

  // Filter out format field for repository content selector type and always set format to "*"
  const filteredFields =
    type === TYPE_IDS.REPOSITORY_CONTENT_SELECTOR ? fields.filter(field => field.id !== 'format') : fields;

  const cancel = () => onDone();

  const confirmDelete = () => send({type: 'CONFIRM_DELETE'});

  const setType = privilegeType => {
    // When setting type to repository content selector, ensure format is set to "*"
    if (privilegeType === TYPE_IDS.REPOSITORY_CONTENT_SELECTOR) {
      send({ type: 'UPDATE', data: { format: '*' } });
    }
    send({ type: 'SET_TYPE', privilegeType });
  };

  const onChangeField = (name, value) => send({type: 'UPDATE', data: {[name]: value}});

  // Ensure format is always set to "*" for repository content selector type
  useEffect(() => {
    if (type === TYPE_IDS.REPOSITORY_CONTENT_SELECTOR && current.context.data.format !== '*') {
      send({ type: 'UPDATE', data: { format: '*' } });
    }
  }, [type, current.context.data.format, send]);

  return <NxStatefulForm
      {...FormUtils.formProps(current, send)}
      onCancel={cancel}
      additionalFooterBtns={isEdit && canDelete &&
          <NxButton variant="tertiary" onClick={confirmDelete} type="button">
            <NxFontAwesomeIcon icon={faTrash}/>
            <span>{UIStrings.SETTINGS.DELETE_BUTTON_LABEL}</span>
          </NxButton>
      }
  >
    <NxTile.Content>
      <NxH2>{LABELS.SECTIONS.SETUP}</NxH2>
      <NxFormGroup label={LABELS.TYPE.LABEL} isRequired>
        <NxFormSelect
            {...FormUtils.selectProps('type', current)}
            value={type}
            disabled={isEdit}
            onChange={setType}
            validatable>
          <option disabled={isTypeSelected} value=""/>
          {Object.values(types)?.map(({id, name}) => <option key={id} value={id}>{name}</option>)}
        </NxFormSelect>
      </NxFormGroup>
      <NxFormGroup label={LABELS.NAME.LABEL} isRequired>
        <NxTextInput
            className="nx-text-input--long"
            {...FormUtils.fieldProps('name', current)}
            onChange={FormUtils.handleUpdate('name', send)}
            disabled={isEdit}
        />
      </NxFormGroup>
      <NxFormGroup label={LABELS.DESCRIPTION.LABEL}>
        <NxTextInput
            className="nx-text-input--long"
            {...FormUtils.fieldProps('description', current)}
            onChange={FormUtils.handleUpdate('description', send)}
        />
      </NxFormGroup>

      {type && <>
        {FormFieldsFactory.getFields(filteredFields)?.map(({Field, props}) => (
          props.id === 'actions' ?
            <NxFieldset
                key={props.id}
                label={props.label}
                sublabel={props.helpText}
                isRequired={props.required}
                isPristine={!isTouched.actions}
                validationErrors={actionSelected}
            >
              <Field
                  id={props.id}
                  dynamicProps={props}
                  current={current}
                  onChange={onChangeField}
              />
            </NxFieldset> :
            <NxFormGroup
                key={props.id}
                label={props.label}
                sublabel={props.helpText}
                isRequired={props.required}
            >
              <Field
                  id={props.id}
                  dynamicProps={props}
                  current={current}
                  onChange={onChangeField}
              />
            </NxFormGroup>
        ))}
      </>}
    </NxTile.Content>
  </NxStatefulForm>;
}
