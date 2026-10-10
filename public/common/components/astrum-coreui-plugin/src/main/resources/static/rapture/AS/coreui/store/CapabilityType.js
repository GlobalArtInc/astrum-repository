
/*global Ext, AS*/

/**
 * Capability type store.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.store.CapabilityType', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.CapabilityType',

  proxy: {
    type: 'direct',
    paramsAsHash: false,
    api: {
      read: 'AS.direct.capability_Capability.readTypes'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'name', direction: 'ASC' }

});
