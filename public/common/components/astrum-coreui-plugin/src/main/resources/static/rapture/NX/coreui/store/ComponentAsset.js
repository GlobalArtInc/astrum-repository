
/*global Ext, NX*/

/**
 * Asset store.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.store.ComponentAsset', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.Asset',

  proxy: {
    type: 'direct',

    api: {
      read: 'NX.direct.coreui_Component.readComponentAssets'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  remoteFilter: true,
  sorters: { property: 'name', direction: 'ASC' }

});
