
import React from 'react';

import PropTypes from 'prop-types';
import classNames from 'classnames';
import {AsFontAwesomeIcon} from "@globalart/astrum-react-shared-components";

import './PageTitle.scss';

/**
 * @since 3.26
 */
export default function PageTitle({className, icon, text, description, ...rest}) {
  const classes = classNames('as-page-title', className);

  return <div className={classes} {...rest}>
    <h1 className="as-h1 as-feature-name">
      {icon && <AsFontAwesomeIcon icon={icon} className="as-page-title__page-icon"/>}
      <span>{text}</span>
    </h1>
    {description && <p className="as-page-title__description as-feature-description">{description}</p>}
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
