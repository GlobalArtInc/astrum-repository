

import React from 'react';

import SearchFeatureExt from './SearchFeatureExt';
import UIStrings from '../../../../constants/UIStrings';

export default function SearchRawExt() {
  return (
    <SearchFeatureExt
      title={UIStrings.SEARCH.RAW.MENU.text}
      icon={UIStrings.SEARCH.RAW.MENU.icon}
      filter={{
        id: 'raw',
        name: 'Raw',
        text: UIStrings.SEARCH.RAW.MENU.text,
        description: UIStrings.SEARCH.RAW.MENU.description,
        readOnly: true,
        criterias: [
          { id: 'format', value: 'raw', hidden: true },
          { id: 'name.raw' }
        ]
      }}
    />
  );
}
