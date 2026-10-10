
/*global Ext, AS*/

/**
 * Task store.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.store.Task', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.Task',

  proxy: {
    type: 'direct',
    paramsAsHash: false,

    api: {
      read: 'AS.direct.coreui_Task.read'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'name', direction: 'ASC' }
});
