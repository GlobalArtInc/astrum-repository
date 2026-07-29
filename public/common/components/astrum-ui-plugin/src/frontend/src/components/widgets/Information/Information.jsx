

import React from 'react';
import PropTypes from 'prop-types';
import { NxDescriptionList } from '@globalart/react-shared-components';

export default function Information({ information }) {
  // Insert a zero-width space (\u200B) after each period to allow word-breaking at periods.
  const ZERO_WIDTH_SPACE = '.\u200B';

  return (
    <NxDescriptionList>
      {Object.entries(information).map(([name, value]) => {
        const breakingName = name.replace(/\./g, ZERO_WIDTH_SPACE);
        return (
          <NxDescriptionList.Item key={name}>
            <NxDescriptionList.Term className="break-on-period">{breakingName}</NxDescriptionList.Term>
            <NxDescriptionList.Description>{String(value)}</NxDescriptionList.Description>
          </NxDescriptionList.Item>
        );
      })}
    </NxDescriptionList>
  );
}

Information.propTypes = {
  information: PropTypes.object.isRequired
};
