
import React, {Fragment} from 'react';
import {useActor} from '@xstate/react';

import {
  AsButton,
  AsH2,
  AsReadOnly,
  AsLoadWrapper,
  AsInfoAlert,
  AsFooter,
  AsButtonBar,
  AsTile,
} from '@globalart/astrum-react-shared-components';
import {FormFieldsFactory} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

const {PRIVILEGES: {FORM: LABELS}} = UIStrings;

export default function PrivilegesReadOnly({service, onDone}) {
  const [current, send] = useActor(service);

  const {
    data,
    types = {},
    loadError
  } = current.context;

  const {type, name, description, readOnly:isDefaultPrivilege = true} = data;
  const isLoading = current.matches('loading');
  const fields = types[type]?.formFields || [];

  const cancel = () => onDone();

  const retry = () => send({type: 'RETRY'});

  return <AsLoadWrapper loading={isLoading} error={loadError} retryHandler={retry}>
    <AsTile.Content>
      <AsInfoAlert>
        {isDefaultPrivilege ? LABELS.DEFAULT_PRIVILEGE_WARNING : UIStrings.SETTINGS.READ_ONLY.WARNING}
      </AsInfoAlert>
      <AsH2>{LABELS.SECTIONS.SETUP}</AsH2>
      <AsReadOnly>
        <AsReadOnly.Label>{LABELS.TYPE.LABEL}</AsReadOnly.Label>
        <AsReadOnly.Data>{types[type]?.name}</AsReadOnly.Data>
        <AsReadOnly.Label>{LABELS.NAME.LABEL}</AsReadOnly.Label>
        <AsReadOnly.Data>{name}</AsReadOnly.Data>
        {description && <>
          <AsReadOnly.Label>{LABELS.DESCRIPTION.LABEL}</AsReadOnly.Label>
          <AsReadOnly.Data>{description}</AsReadOnly.Data>
        </>}
        {FormFieldsFactory.getFields(fields)?.map(({Field, props}) => (
            <Fragment key={props.id}>
              <AsReadOnly.Label>{props.label}</AsReadOnly.Label>
              <AsReadOnly.Data>
                <Field
                    id={props.id}
                    dynamicProps={{...props, readOnly: true}}
                    current={current}
                />
              </AsReadOnly.Data>
            </Fragment>
        ))}
      </AsReadOnly>
    </AsTile.Content>
    <AsFooter>
      <AsButtonBar>
        <AsButton type="button" onClick={cancel}>{UIStrings.SETTINGS.CANCEL_BUTTON_LABEL}</AsButton>
      </AsButtonBar>
    </AsFooter>
  </AsLoadWrapper>;
}
