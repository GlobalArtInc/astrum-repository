

import React from 'react';

import SearchFeatureExt from './SearchFeatureExt';
import UIStrings from '../../../../constants/UIStrings';

export default function SearchRExt() {
  return (
    <SearchFeatureExt
      title={UIStrings.SEARCH.R.MENU.text}
      icon={UIStrings.SEARCH.R.MENU.icon}
      filter={{
        id: 'r',
        name: 'r',
        text: UIStrings.SEARCH.R.MENU.text,
        description: UIStrings.SEARCH.R.MENU.description,
        readOnly: true,
        criterias: [
          {id: 'format', value: 'r', hidden: true},
          {id: 'name.raw'},
          {id: 'version'}
        ]
      }}
    />
  );
}
