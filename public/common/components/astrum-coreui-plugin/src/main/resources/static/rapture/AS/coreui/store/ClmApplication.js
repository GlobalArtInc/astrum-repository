
/*global Ext, AS*/

/**
 * CLM Application store.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.store.ClmApplication', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.Reference',

  proxy: {
    type: 'direct',
    paramsAsHash: false,

    api: {
      read: 'AS.direct.clm_CLM.readApplications'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'name', direction: 'ASC' }
});
