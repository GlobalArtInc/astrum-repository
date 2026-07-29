

import PropTypes from 'prop-types';
import React from 'react';
import { NxTextLink } from '@globalart/react-shared-components';

const CollapsibleListItem = ({ text, href, isSelected, ...props }) => {
  return (
    <li className="as-navigation-expandable-link__expandable-list__item">
      <NxTextLink
        href={href}
        className={`as-navigation-expandable-link__expandable-list__navigation-link ${isSelected ? 'selected' : ''}`}
        {...props}
      >
        {text}
      </NxTextLink>
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
