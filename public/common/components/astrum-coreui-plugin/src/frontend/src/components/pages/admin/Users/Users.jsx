
import React, {useState} from 'react';

import {Detail, Master, MasterDetail} from '@globalart/astrum-ui-plugin';

import UsersList from './UsersList';
import UsersDetails from './UsersDetails';

export default function Users() {
  const [sourceFilter, setSourceFilter] = useState('default');

  return <MasterDetail path="admin/security/users">
    <Master>
      <UsersList initialSourceFilter={sourceFilter} onSourceFilterChange={setSourceFilter}/>
    </Master>
    <Detail>
      <UsersDetails/>
    </Detail>
  </MasterDetail>;
}
