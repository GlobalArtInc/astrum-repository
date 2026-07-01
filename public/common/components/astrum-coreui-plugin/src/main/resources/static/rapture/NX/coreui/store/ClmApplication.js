
/*global Ext, NX*/

/**
 * CLM Application store.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.store.ClmApplication', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.Reference',

  proxy: {
    type: 'direct',
    paramsAsHash: false,

    api: {
      read: 'NX.direct.clm_CLM.readApplications'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'name', direction: 'ASC' }
});
