
import React from 'react';
import PropTypes from 'prop-types';
import classNames from "classnames";

import {AsFontAwesomeIcon, AsP, AsH3} from "@globalart/react-shared-components";
import {faInfoCircle} from '@fortawesome/free-solid-svg-icons';

import './HelpTile.scss';
/**
 * @since 3.29
 */
export default function HelpTile({className, header, body, ...attrs}) {
  return <div className={classNames('as-help-tile', 'as-tile', className)} {...attrs}>
    <div className="as-tile-content">
      <AsH3><AsFontAwesomeIcon icon={faInfoCircle}/><span>{header}</span></AsH3>
      <AsP>{body}</AsP>
    </div>
  </div>;
}

HelpTile.propTypes = {
  className: PropTypes.string,
  header: PropTypes.string,
  body: PropTypes.oneOfType([PropTypes.string, PropTypes.element]),
};
