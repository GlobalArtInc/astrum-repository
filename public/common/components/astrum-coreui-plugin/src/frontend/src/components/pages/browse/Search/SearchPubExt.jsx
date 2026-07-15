

import React from 'react';

import SearchFeatureExt from './SearchFeatureExt';
import UIStrings from '../../../../constants/UIStrings';

/**
 * Search component for Pub package repositories.
 * Provides format-specific search criteria including name and version.
 */
export default function SearchPubExt() {
  return (
    <SearchFeatureExt
      title={UIStrings.SEARCH.PUB.MENU.text}
      icon={UIStrings.SEARCH.PUB.MENU.icon}
      criterias={[]}
      filter={{
        id: 'pub',
        name: 'pub',
        text: UIStrings.SEARCH.PUB.MENU.text,
        description: UIStrings.SEARCH.PUB.MENU.description,
        readOnly: true,
        criterias: [
          { id: 'format', value: 'pub', hidden: true },
          { id: 'name.raw' },
          { id: 'version' }
        ]
      }}
    />
  );
}
