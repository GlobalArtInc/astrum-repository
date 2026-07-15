

import React from "react";

import SearchFeatureExt from "./SearchFeatureExt";
import UIStrings from "../../../../constants/UIStrings";

export default function SearchAptExt() {
  return (
    <SearchFeatureExt
      title={UIStrings.SEARCH.APT.MENU.text}
      icon={UIStrings.SEARCH.APT.MENU.icon}
      filter={{
        id: "apt",
        name: "Apt",
        ...UIStrings.SEARCH.APT.MENU,
        readOnly: true,
        criterias: [
          { id: "format", value: "apt", hidden: true },
          { id: "name.raw" },
          { id: "version" },
        ],
      }}
    />
  );
}
