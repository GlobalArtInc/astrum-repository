
import React from 'react';
import PropTypes from 'prop-types';
import classNames from "classnames";

import {NxFontAwesomeIcon, NxP, NxH3} from "@globalart/react-shared-components";
import {faInfoCircle} from '@fortawesome/free-solid-svg-icons';

import './HelpTile.scss';
/**
 * @since 3.29
 */
export default function HelpTile({className, header, body, ...attrs}) {
  return <div className={classNames('as-help-tile', 'nx-tile', className)} {...attrs}>
    <div className="nx-tile-content">
      <NxH3><NxFontAwesomeIcon icon={faInfoCircle}/><span>{header}</span></NxH3>
      <NxP>{body}</NxP>
    </div>
  </div>;
}

HelpTile.propTypes = {
  className: PropTypes.string,
  header: PropTypes.string,
  body: PropTypes.oneOfType([PropTypes.string, PropTypes.element]),
};
