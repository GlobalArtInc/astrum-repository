
import React from 'react';

import {NxTextInput} from '@globalart/react-shared-components';
import FormUtils from '../../../../interface/FormUtils';

import {SUPPORTED_FIELD_TYPES} from '../FormFieldsFactoryConstants';

const Field = ({id, dynamicProps, current, onChange}) => {
  const {type, initialValue, readOnly, disabled} = dynamicProps;
  const {data} = current.context;
  let props = {disabled};

  if (type === 'password') {
    props = {
      ...props,
      type: 'password',
      autoComplete: 'new-password',
    };
  }

  if (readOnly) {
    return data[id];
  }

  return <NxTextInput
      {...props}
      {...FormUtils.fieldProps(id, current, initialValue || '')}
      onChange={value => onChange(id, value)}
  />;
};

export default {
  types: SUPPORTED_FIELD_TYPES.TEXT,
  component: Field,
};

