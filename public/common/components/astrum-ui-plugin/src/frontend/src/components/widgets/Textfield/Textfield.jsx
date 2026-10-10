
import PropTypes from 'prop-types';
import React from 'react';
import {AsTextInput} from '@globalart/astrum-react-shared-components';

/**
 * @since 3.21
 * @deprecated prefer AsTextInput instead
 */
export default function Textfield({id, name, type = "text", onChange, isPristine = false, validatable = true, ...attrs}) {
  function handleChange(value) {
    if (onChange) {
      const target = {
        id: id || name,
        name,
        type,
        value
      };
      onChange({
        currentTarget: target,
        target: target
      });
    }
  }

  const inputAttrs = {
    ...attrs,
    id: id || name,
    name,
    type: type === 'number' ? 'text' : type,
    isPristine,
    validatable,
    onChange: handleChange
  };

  return <AsTextInput {...inputAttrs} />;
}

Textfield.propTypes = {
  id: PropTypes.string,
  name: PropTypes.string,
  value: PropTypes.string,
  validationErrors: PropTypes.oneOfType([PropTypes.string, PropTypes.array])
};
