
import React from 'react';
import {AsReadOnly} from '@globalart/react-shared-components';
import {isNil, isEmpty} from 'ramda';

/**
 * @param {string} label
 * @param value [required]
 * @param item [optional]
 * @return {ReactNode}
 */
export default function ReadOnlyField({label, value, item}) {
  const valid = !isNil(value) && !isEmpty(value);

  if (!valid) {
    return null;
  }

  const labelAndData = (
      <>
        <AsReadOnly.Label>{label}</AsReadOnly.Label>
        <AsReadOnly.Data>{value}</AsReadOnly.Data>
      </>
  );

  return (
      item ? (<AsReadOnly.Item>{labelAndData}</AsReadOnly.Item>) : labelAndData
  );
}
