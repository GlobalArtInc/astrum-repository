
import React from 'react';

import PropTypes from 'prop-types';
import classNames from 'classnames';
import {NxFontAwesomeIcon} from "@globalart/react-shared-components";

import './PageTitle.scss';

/**
 * @since 3.26
 */
export default function PageTitle({className, icon, text, description, ...rest}) {
  const classes = classNames('nx-page-title', className);

  return <div className={classes} {...rest}>
    <h1 className="nx-h1 nx-feature-name">
      {icon && <NxFontAwesomeIcon icon={icon} className="nx-page-title__page-icon"/>}
      <span>{text}</span>
    </h1>
    {description && <p className="nx-page-title__description nx-feature-description">{description}</p>}
  </div>;
}

PageTitle.propTypes = {
  text: PropTypes.string.isRequired,
  icon: PropTypes.oneOfType([
    PropTypes.object,
    PropTypes.array,
    PropTypes.string
  ]),
  description: PropTypes.string
};
