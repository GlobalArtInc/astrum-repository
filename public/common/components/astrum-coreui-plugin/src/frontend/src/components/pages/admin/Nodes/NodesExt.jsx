

import { ExtJsContainer } from "../../../widgets/ExtJsContainer/ExtJsContainer";
import React from "react";
import { ExtJS } from "@globalart/astrum-ui-plugin/src/frontend/src";
import Nodes from "./NodeList";
import FeatureFlags from "../../../../constants/FeatureFlags";
import UIStrings from '../../../../constants/UIStrings';

const { DATASTORE_CLUSTERED_ENABLED } = FeatureFlags;

export default function NodesExt() {
  const shouldShowReactVersion = ExtJS.useState(() =>
    ExtJS.state().getValue(DATASTORE_CLUSTERED_ENABLED)
  );

  if (shouldShowReactVersion) {
    return <Nodes />;
  } else {
    return (
      <ExtJsContainer
        title={UIStrings.NODES.MENU.text}
        icon={UIStrings.NODES.MENU.icon}
        className="as-nodes"
        extView="NX.coreui.view.system.Nodes"
      />
    );
  }
}
