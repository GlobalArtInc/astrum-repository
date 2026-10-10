

import { ExtJsContainer } from "../../../widgets/ExtJsContainer/ExtJsContainer";
import React from "react";
import { ExtJS } from '@globalart/astrum-ui-plugin/src/frontend/src';
import Repositories from "./Repositories";
import UIStrings from '../../../../constants/UIStrings';

export default function RepositoriesExt() {
   const shouldShowReactVersion = ExtJS.useState(() => ExtJS.state().getValue('astrum.react.repositories'));

   if (shouldShowReactVersion) {
       return <Repositories />
   } else {
       return <ExtJsContainer
           title={UIStrings.REPOSITORIES.MENU.text}
           icon={UIStrings.REPOSITORIES.MENU.icon}
           className="as-feature-content"
           extView="AS.coreui.view.repository.RepositoryFeature"
       />;
   }
}
