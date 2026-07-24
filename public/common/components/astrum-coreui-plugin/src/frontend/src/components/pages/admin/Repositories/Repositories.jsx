
import React from 'react';

import {Detail, Master, MasterDetail} from '@globalart/astrum-ui-plugin';

import RepositoriesList from './RepositoriesList';
import RepositoriesForm from './RepositoriesForm';
import RepositoriesContextProvider from './RepositoriesContextProvider';

export default function Repositories() {
  return (
    <RepositoriesContextProvider>
      <MasterDetail path="admin/repository/repositories">
        <Master>
          <RepositoriesList />
        </Master>
        <Detail>
          <RepositoriesForm />
        </Detail>
      </MasterDetail>
    </RepositoriesContextProvider>
  );
}
