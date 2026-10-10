

import React from 'react';
import PropTypes from 'prop-types';
import { AsDescriptionList } from '@globalart/react-shared-components';

export default function Information({ information }) {
  // Insert a zero-width space (\u200B) after each period to allow word-breaking at periods.
  const ZERO_WIDTH_SPACE = '.\u200B';

  return (
    <AsDescriptionList>
      {Object.entries(information).map(([name, value]) => {
        const breakingName = name.replace(/\./g, ZERO_WIDTH_SPACE);
        return (
          <AsDescriptionList.Item key={name}>
            <AsDescriptionList.Term className="break-on-period">{breakingName}</AsDescriptionList.Term>
            <AsDescriptionList.Description>{String(value)}</AsDescriptionList.Description>
          </AsDescriptionList.Item>
        );
      })}
    </AsDescriptionList>
  );
}

Information.propTypes = {
  information: PropTypes.object.isRequired
};
