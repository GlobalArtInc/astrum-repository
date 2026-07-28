
import React from 'react';
import classNames from 'classnames';

/**
 * @since 3.22
 */
export default function Section({className, children, ...rest}) {
  return <div className={classNames("as-section", "nx-tile", className)} {...rest}>
    <div className="nx-tile-content">
      {children}
    </div>
  </div>;

}
