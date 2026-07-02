
/*global Ext, NX*/

/**
 * Ssl Certificate store.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.store.SslCertificate', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.SslCertificate',

  proxy: {
    type: 'direct',
    paramsAsHash: false,

    api: {
      read: 'NX.direct.ssl_TrustStore.read'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'subjectCommonName', direction: 'ASC' }
});
