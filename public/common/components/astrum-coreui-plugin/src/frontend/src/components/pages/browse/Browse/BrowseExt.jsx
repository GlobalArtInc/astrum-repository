

import { ExtJsContainer } from "../../../widgets/ExtJsContainer/ExtJsContainer";
import React from "react";
import { ExtJS } from '@globalart/astrum-ui-plugin/src/frontend/src';
import Browse from './Browse';
import UIStrings from '../../../../constants/UIStrings';

export default function BrowseExt() {
   const shouldShowReactVersion = ExtJS.useState(() => ExtJS.state().getValue('astrum.react.browse'));

   if (shouldShowReactVersion) {
       return <Browse />
   } else {
       return (
           <ExtJsContainer
             title={UIStrings.BROWSE.MENU.text}
             icon={UIStrings.BROWSE.MENU.icon}
             className="as-browse-tree"
             extView="AS.coreui.view.browse.ComponentAssetTreeFeature"
          />);
   }
}
