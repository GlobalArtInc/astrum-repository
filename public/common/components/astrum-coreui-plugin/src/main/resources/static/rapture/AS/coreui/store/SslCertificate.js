
/*global Ext, AS*/

/**
 * Ssl Certificate store.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.store.SslCertificate', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.SslCertificate',

  proxy: {
    type: 'direct',
    paramsAsHash: false,

    api: {
      read: 'AS.direct.ssl_TrustStore.read'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'subjectCommonName', direction: 'ASC' }
});
