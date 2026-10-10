

import React from 'react';
import {ExtJsContainer} from "../../../widgets/ExtJsContainer/ExtJsContainer";
import UIStrings from '../../../../constants/UIStrings';

export default function LdapServersExt() {
    return <ExtJsContainer
        title={UIStrings.LDAP_SERVERS.MENU.text}
        icon={UIStrings.LDAP_SERVERS.MENU.icon}
        extView="AS.coreui.view.ldap.LdapServerFeature"
    />;
}
