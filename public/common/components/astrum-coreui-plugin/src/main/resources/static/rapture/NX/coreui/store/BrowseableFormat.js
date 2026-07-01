
/*global Ext*/

/**
 * BrowseableFormat store.
 *
 * @since 3.2.1
 */
Ext.define('NX.coreui.store.BrowseableFormat', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.BrowseableFormat',

  proxy: {
    type: 'direct',

    api: {
      read: 'NX.direct.coreui_Repository.getBrowseableFormats'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  }
});
