

import PropTypes from 'prop-types';
import React from 'react';
import { AsTextLink } from '@globalart/react-shared-components';

const CollapsibleListItem = ({ text, href, isSelected, ...props }) => {
  return (
    <li className="as-navigation-expandable-link__expandable-list__item">
      <AsTextLink
        href={href}
        className={`as-navigation-expandable-link__expandable-list__navigation-link ${isSelected ? 'selected' : ''}`}
        {...props}
      >
        {text}
      </AsTextLink>
    </li>
  );
};

CollapsibleListItem.propTypes = {
  href: PropTypes.string.isRequired,
  isSelected: PropTypes.bool,
  props: PropTypes.any,
  text: PropTypes.oneOfType([PropTypes.string, PropTypes.element]).isRequired
};

export default CollapsibleListItem;
