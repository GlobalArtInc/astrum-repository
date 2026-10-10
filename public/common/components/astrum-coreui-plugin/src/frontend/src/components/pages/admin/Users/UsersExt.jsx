

import React from "react";
import { ExtJS } from '@globalart/astrum-ui-plugin/src/frontend/src';
import { ExtJsContainer } from "../../../widgets/ExtJsContainer/ExtJsContainer";
import Users from "./Users";
import UIStrings from '../../../../constants/UIStrings';

export default function UsersExt() {
   const shouldShowReactVersion = ExtJS.useState(() => ExtJS.state().getValue('astrum.react.users', false));

   if (shouldShowReactVersion) {
       return <Users />
   } else {
       return <ExtJsContainer
           title={UIStrings.USERS.MENU.text}
           icon={UIStrings.USERS.MENU.icon}
           className="as-feature-content"
           extView="AS.coreui.view.user.UserFeature"
       />;
   }
}
