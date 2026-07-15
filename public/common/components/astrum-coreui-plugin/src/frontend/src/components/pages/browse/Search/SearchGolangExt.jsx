

import React from 'react';

import SearchFeatureExt from './SearchFeatureExt';
import UIStrings from '../../../../constants/UIStrings';

export default function SearchGolangExt() {
  return (
    <SearchFeatureExt
      title={UIStrings.SEARCH.GOLANG.MENU.text}
      icon={UIStrings.SEARCH.GOLANG.MENU.icon}
      filter={{
        id: 'go',
        name: 'Go',
        text: UIStrings.SEARCH.GOLANG.MENU.text,
        description: UIStrings.SEARCH.GOLANG.MENU.description,
        readOnly: true,
        criterias: [
          { id: 'format', value: 'go', hidden: true },
          { id: 'name.raw' },
          { id: 'version' },
          { id: 'keyword' }
        ]
      }}
    />
  );
}
