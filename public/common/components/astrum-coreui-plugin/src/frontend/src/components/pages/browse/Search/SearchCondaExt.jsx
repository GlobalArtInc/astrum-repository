

import React from 'react';

import SearchFeatureExt from './SearchFeatureExt';
import UIStrings from '../../../../constants/UIStrings';

export default function SearchCondaExt() {
  return (
    <SearchFeatureExt
      title={UIStrings.SEARCH.CONDA.MENU.text}
      icon={UIStrings.SEARCH.CONDA.MENU.icon}
      filter={{
        id: 'conda',
        name: 'Conda',
        text: UIStrings.SEARCH.CONDA.MENU.text,
        description: UIStrings.SEARCH.CONDA.MENU.description,
        readOnly: true,
        criterias: [
          {id: 'format', value: 'conda', hidden: true},
          {id: 'name.raw'},
          {id: 'version'}
        ]
      }}
    />
  );
}
