
import React, { useEffect } from 'react';
import {useActor} from '@xstate/react';

import {
  FormUtils,
  ValidationUtils,
  ExtJS,
  FormFieldsFactory,
} from '@globalart/astrum-ui-plugin';
import {
  AsButton,
  AsH2,
  AsFieldset,
  AsFontAwesomeIcon,
  AsFormGroup,
  AsFormSelect,
  AsStatefulForm,
  AsTextInput,
  AsTile
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

  return <AsStatefulForm
      {...FormUtils.formProps(current, send)}
      onCancel={cancel}
      additionalFooterBtns={isEdit && canDelete &&
          <AsButton variant="tertiary" onClick={confirmDelete} type="button">
            <AsFontAwesomeIcon icon={faTrash}/>
            <span>{UIStrings.SETTINGS.DELETE_BUTTON_LABEL}</span>
          </AsButton>
      }
  >
    <AsTile.Content>
      <AsH2>{LABELS.SECTIONS.SETUP}</AsH2>
      <AsFormGroup label={LABELS.TYPE.LABEL} isRequired>
        <AsFormSelect
            {...FormUtils.selectProps('type', current)}
            value={type}
            disabled={isEdit}
            onChange={setType}
            validatable>
          <option disabled={isTypeSelected} value=""/>
          {Object.values(types)?.map(({id, name}) => <option key={id} value={id}>{name}</option>)}
        </AsFormSelect>
      </AsFormGroup>
      <AsFormGroup label={LABELS.NAME.LABEL} isRequired>
        <AsTextInput
            className="as-text-input--long"
            {...FormUtils.fieldProps('name', current)}
            onChange={FormUtils.handleUpdate('name', send)}
            disabled={isEdit}
        />
      </AsFormGroup>
      <AsFormGroup label={LABELS.DESCRIPTION.LABEL}>
        <AsTextInput
            className="as-text-input--long"
            {...FormUtils.fieldProps('description', current)}
            onChange={FormUtils.handleUpdate('description', send)}
        />
      </AsFormGroup>

      {type && <>
        {FormFieldsFactory.getFields(filteredFields)?.map(({Field, props}) => (
          props.id === 'actions' ?
            <AsFieldset
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
            </AsFieldset> :
            <AsFormGroup
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
            </AsFormGroup>
        ))}
      </>}
    </AsTile.Content>
  </AsStatefulForm>;
}
