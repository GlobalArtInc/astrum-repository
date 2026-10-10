
/*global Ext, AS*/

/**
 * Task type store.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.store.TaskType', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.TaskType',

  proxy: {
    type: 'direct',
    paramsAsHash: false,

    api: {
      read: 'AS.direct.coreui_Task.readTypes'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'name', direction: 'ASC' }
});
