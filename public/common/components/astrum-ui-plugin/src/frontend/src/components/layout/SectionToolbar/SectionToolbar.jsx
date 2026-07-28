
import React from 'react';
import classNames from 'classnames';

import './SectionToolbar.scss';

/**
 * @since 3.27
 *
 * This component must be used within a Section component.
 */
export default function SectionToolbar({className, children, ...rest}) {
  return <div className={classNames("as-section-toolbar", className)} {...rest}>{children}</div>;
}
