
import classNames from 'classnames';
import React from 'react';

import './ContentBody.scss';

/**
 * @since 3.21
 */
export default function ContentBody({children, className}) {
  const classes = classNames('as-content-body', className);

  return <div className={classes}>
    {children}
  </div>;
}
