

import React from "react";

import SearchFeatureExt from './SearchFeatureExt';
import UIStrings from "../../../../constants/UIStrings";

export default function SearchCocoapodsExt() {
  return (
    <SearchFeatureExt
      title={UIStrings.SEARCH.COCOAPODS.MENU.text}
      icon={UIStrings.SEARCH.COCOAPODS.MENU.icon}
      filter={{
        id: 'cocoapods',
        name: 'Cocoapods',
        text: UIStrings.SEARCH.COCOAPODS.MENU.text,
        description: UIStrings.SEARCH.COCOAPODS.MENU.description,
        readOnly: true,
        criterias: [
          {id: 'format', value: 'cocoapods', hidden: true},
          {id: 'name.raw'},
          {id: 'version'}
        ]
      }}
    />
  );
}
