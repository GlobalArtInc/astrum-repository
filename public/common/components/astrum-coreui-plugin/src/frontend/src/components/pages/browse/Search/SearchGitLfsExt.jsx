

import React from 'react';

import SearchFeatureExt from './SearchFeatureExt';
import UIStrings from '../../../../constants/UIStrings';

export default function SearchGitLfsExt() {
  return (
    <SearchFeatureExt
      title={UIStrings.SEARCH.GITLFS.MENU.text}
      icon={UIStrings.SEARCH.GITLFS.MENU.icon}
      filter={{
        id: 'gitlfs',
        name: 'Git LFS',
        text: UIStrings.SEARCH.GITLFS.MENU.text,
        description: UIStrings.SEARCH.GITLFS.MENU.description,
        readOnly: true,
        criterias: [
          { id: 'format', value: 'gitlfs', hidden: true },
          { id: 'name.raw' }
        ]
      }}
    />
  );
}
