
import React from 'react';
import {keys, mergeDeepRight} from 'ramda';

import {AsCheckbox} from '@globalart/astrum-react-shared-components';
import FormUtils from '../../../../interface/FormUtils';

import {SUPPORTED_FIELD_TYPES} from '../FormFieldsFactoryConstants';

const Field = ({id, dynamicProps, current, onChange}) => {
  const {attributes: {options}, readOnly} = dynamicProps;
  const {data} = current.context;

  if (readOnly) {
    return keys(data[id]).map(e => e.charAt(0).toUpperCase() + e.slice(1)).join(', ');
  }

  return options.map(option =>
      <AsCheckbox key={option}
                  {...FormUtils.checkboxProps([id, option], current)}
                  onChange={ e => onChange(id, mergeDeepRight(data[id], {[option]: e}))}>
        {option.charAt(0).toUpperCase() + option.slice(1)}
      </AsCheckbox>
  );
};

export default {
  types: SUPPORTED_FIELD_TYPES.SET_OF_CHECKBOXES,
  component: Field,
};
