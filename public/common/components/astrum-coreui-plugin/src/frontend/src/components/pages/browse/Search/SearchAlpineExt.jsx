

import React from 'react';

import SearchFeatureExt from './SearchFeatureExt';
import UIStrings from '../../../../constants/UIStrings';

export default function SearchAlpineExt() {
  return (
    <SearchFeatureExt
      title={UIStrings.SEARCH.ALPINE.MENU.text}
      icon={UIStrings.SEARCH.ALPINE.MENU.icon}
      filter={{
        id: 'alpine',
        name: 'Alpine',
        ...UIStrings.SEARCH.ALPINE.MENU,
        readOnly: true,
        criterias: [
          { id: 'format', value: 'alpine', hidden: true },
          { id: 'name.raw' },
          { id: 'version' },
        ],
      }}
    />
  );
}
