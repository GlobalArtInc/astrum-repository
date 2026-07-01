
/*global Ext, NX*/

/**
 * Capability store.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.store.Capability', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.Capability',

  proxy: {
    type: 'direct',
    paramsAsHash: false,
    api: {
      read: 'NX.direct.capability_Capability.read'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'typeName', direction: 'ASC' }

});
