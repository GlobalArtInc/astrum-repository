
import React from 'react';

import {Detail, Master, MasterDetail} from '@globalart/astrum-ui-plugin';

import LdapServersList from './LdapServersList';
import LdapServersDetails from './LdapServersDetails';

export default function LdapServers() {
  return <MasterDetail path="admin/security/ldap">
    <Master>
      <LdapServersList/>
    </Master>
    <Detail>
      <LdapServersDetails/>
    </Detail>
  </MasterDetail>;
}
