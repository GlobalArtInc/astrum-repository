

import { ExtJsContainer } from '../../../widgets/ExtJsContainer/ExtJsContainer';
import React from 'react';
import UIStrings from '../../../../constants/UIStrings';

export default function Capabilities() {
 return <ExtJsContainer
     title={UIStrings.CAPABILITIES.MENU.text}
     icon={UIStrings.CAPABILITIES.MENU.icon}
     extView="NX.coreui.view.capability.CapabilityFeature"
 />;
}
