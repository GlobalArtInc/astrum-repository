
import Axios from 'axios';
import {assign} from 'xstate';

import {ListMachineUtils, ExtJS} from '@globalart/astrum-ui-plugin';

import {URLS} from './SslCertificatesHelper';

export default ListMachineUtils.buildListMachine({
  id: 'SslCertificatesListMachine',
  sortableFields: ['subjectCommonName', 'subjectOrganization', 'issuerOrganization', 'fingerprint'],
  sortField: 'subjectCommonName',
}).withConfig({
  actions: {
    filterData: assign({
      data: ({filter, pristineData}, _) => pristineData.filter(item =>
          ListMachineUtils.hasAnyMatches([
              item.subjectCommonName,
              item.subjectOrganization,
              item.issuerOrganization,
              item.fingerprint,
          ], filter)
      )})
  },
  services: {
    fetchData: async () => {
      await ExtJS.waitForPermissions();
      return Axios.get(URLS.sslCertificatesUrl);
    }
  }
});
