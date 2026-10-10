

import React from 'react';
import PropTypes from 'prop-types';
import classNames from 'classnames';

export default function FieldWrapper({labelText, descriptionText, id, isOptional, children}) {
  const firstChildProps = React.Children.toArray(children)[0].props;
  const fieldName = firstChildProps.id || firstChildProps.name;
  const classes = classNames('as-label', {
    'as-label--optional': isOptional
  });

  return <div className='as-form-group' id={id}>
    {labelText ? <label htmlFor={fieldName} className={classes}><span className="as-label__text">{labelText}</span></label> : null}
    {descriptionText ? <p className='as-p'>{descriptionText}</p> : null}
    {children}
  </div>;
};

FieldWrapper.propTypes = {
  descriptionText: PropTypes.string,
  isOptional: PropTypes.bool,
  labelText: PropTypes.string.isRequired
};
