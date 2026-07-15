

import React from "react";

import SearchFeatureExt from "./SearchFeatureExt";
import UIStrings from "../../../../constants/UIStrings";

export default function SearchCustomExt() {
  return (
    <SearchFeatureExt
      title={UIStrings.SEARCH.CUSTOM.MENU.text}
      icon={UIStrings.SEARCH.CUSTOM.MENU.icon}
      filter={{
        id: "custom",
        name: "Custom",
        text: UIStrings.SEARCH.CUSTOM.MENU.text,
        description: UIStrings.SEARCH.CUSTOM.MENU.description,
        readOnly: true,
      }}
    />
  );
}
