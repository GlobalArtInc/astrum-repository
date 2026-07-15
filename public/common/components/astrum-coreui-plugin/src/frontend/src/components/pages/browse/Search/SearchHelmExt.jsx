

import React from 'react';

import SearchFeatureExt from './SearchFeatureExt';
import UIStrings from '../../../../constants/UIStrings';

export default function SearchHelmExt() {
  return (
    <SearchFeatureExt
      title={UIStrings.SEARCH.HELM.MENU.text}
      icon={UIStrings.SEARCH.HELM.MENU.icon}
      filter={{
        id: 'helm',
        name: 'helm',
        text: UIStrings.SEARCH.HELM.MENU.text,
        description: UIStrings.SEARCH.HELM.MENU.description,
        readOnly: true,
        criterias: [
          {id: 'format', value: 'helm', hidden: true},
          {id: 'name.raw'},
          {id: 'version'}
        ]
      }}
    />
  );
}
