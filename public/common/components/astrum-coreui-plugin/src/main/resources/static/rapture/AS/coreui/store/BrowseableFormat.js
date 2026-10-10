
/*global Ext*/

/**
 * BrowseableFormat store.
 *
 * @since 3.2.1
 */
Ext.define('AS.coreui.store.BrowseableFormat', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.BrowseableFormat',

  proxy: {
    type: 'direct',

    api: {
      read: 'AS.direct.coreui_Repository.getBrowseableFormats'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  }
});
