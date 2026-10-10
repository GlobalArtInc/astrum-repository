
/*global Ext, AS*/

/**
 * Capability store.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.store.Capability', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.Capability',

  proxy: {
    type: 'direct',
    paramsAsHash: false,
    api: {
      read: 'AS.direct.capability_Capability.read'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'typeName', direction: 'ASC' }

});
