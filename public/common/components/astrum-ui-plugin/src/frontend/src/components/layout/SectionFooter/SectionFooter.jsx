
import React from 'react';
import classNames from 'classnames';

/**
 * @since 3.24
 *
 * This component must be used within a Section component.
 */
export default function SectionFooter({className, children, ...rest}) {
  return <div className={classNames("as-footer", className)} {...rest}>{children}</div>;
}
