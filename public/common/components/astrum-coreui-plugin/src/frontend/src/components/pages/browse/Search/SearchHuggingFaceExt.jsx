

import React from 'react';

import SearchFeatureExt from './SearchFeatureExt';
import UIStrings from '../../../../constants/UIStrings';

export default function SearchHuggingFaceExt() {
  return (
    <SearchFeatureExt
      title={UIStrings.SEARCH.HUGGING_FACE.MENU.text}
      icon={UIStrings.SEARCH.HUGGING_FACE.MENU.icon}
      filter={{
        id: 'huggingface',
        name: 'huggingFace',
        text: UIStrings.SEARCH.HUGGING_FACE.MENU.text,
        description: UIStrings.SEARCH.HUGGING_FACE.MENU.description,
        readOnly: true,
        criterias: [
          {id: 'format', value: 'huggingface', hidden: true},
          {id: 'name.raw'},
          {id: 'version'}
        ]
      }}
    />
  );
}
