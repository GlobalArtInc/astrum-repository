
/*global Ext, NX*/

/**
 * Capability type store.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.store.CapabilityType', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.CapabilityType',

  proxy: {
    type: 'direct',
    paramsAsHash: false,
    api: {
      read: 'NX.direct.capability_Capability.readTypes'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'name', direction: 'ASC' }

});
