
import {assign} from 'xstate';
import Axios from 'axios';

import {FormUtils, ExtJS} from '@globalart/astrum-ui-plugin';

import {canDeleteCertificate, URLS} from './SslCertificatesHelper';
const {sslCertificatesUrl, singleSslCertificatesUrl} = URLS;

import UIStrings from '../../../../constants/UIStrings';

const {
  SSL_CERTIFICATES: {MESSAGES: LABELS}
} = UIStrings;

export default FormUtils.buildFormMachine({
  id: 'SslCertificatesDetailsMachine',
}).withConfig({
  actions: {
    validate: assign({
      validationErrors: () => {{}},
    }),
    setData: assign(({pristineData: {id}}, event) => {
      const certificate = event.data?.data?.find(it => it.id === id);
      return {
        data: certificate || {},
        loadError: !certificate ? UIStrings.ERROR.NOT_FOUND_ERROR(id) : null,
      };
    }),
    onDeleteError: (_, event) => ExtJS.showErrorMessage(event.data?.response?.data || event.data?.message),
    logDeleteSuccess: ({data: {subjectCommonName}}) => ExtJS.showSuccessMessage(LABELS.DELETE_SUCCESS(subjectCommonName)),
    setDirtyFlag: () => {
      // No-op: SSL Certificates form does not track dirty state
    },
    clearDirtyFlag: () => {
      // No-op: SSL Certificates form does not track dirty state
    }
  },
  guards: {
    canDelete: ({data}) => data.id && canDeleteCertificate(),
  },
  services: {
    fetchData: async () => Axios.get(sslCertificatesUrl),
    confirmDelete: ({data: {subjectCommonName}}) => ExtJS.requestConfirmation({
      title: LABELS.CONFIRM_DELETE.TITLE,
      message: LABELS.CONFIRM_DELETE.MESSAGE(subjectCommonName),
      yesButtonText: LABELS.CONFIRM_DELETE.YES,
      noButtonText: LABELS.CONFIRM_DELETE.NO
    }),
    delete: ({data: {id}}) => Axios.delete(singleSslCertificatesUrl(id)),
  },
});
