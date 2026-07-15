

import React from "react";

import SearchFeatureExt from './SearchFeatureExt';
import UIStrings from "../../../../constants/UIStrings";

export default function SearchCargoExt() {
  return (
    <SearchFeatureExt
      title={UIStrings.SEARCH.CARGO.MENU.text}
      icon={UIStrings.SEARCH.CARGO.MENU.icon}
      filter={{
        id: 'cargo',
        name: 'Cargo',
        text: UIStrings.SEARCH.CARGO.MENU.text,
        description: UIStrings.SEARCH.CARGO.MENU.description,
        readOnly: true,
        criterias: [
          { id: 'format', value: 'cargo', hidden: true },
          { id: 'name.raw' },
          { id: 'version' }
        ]
      }}
    />
  );
}
