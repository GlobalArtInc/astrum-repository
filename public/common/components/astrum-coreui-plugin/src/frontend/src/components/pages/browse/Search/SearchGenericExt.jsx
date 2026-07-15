

import React from "react";
import SearchFeatureExt from "./SearchFeatureExt";
import UIStrings from '../../../../constants/UIStrings';

export default function SearchGenericExt() {
  return <SearchFeatureExt
      title={UIStrings.SEARCH.KEYWORD.MENU.text}
      icon={UIStrings.SEARCH.KEYWORD.MENU.icon} />;
}
