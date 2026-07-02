
/*global Ext, NX*/

/**
 * User sources store.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.store.UserSource', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.Reference',

  proxy: {
    type: 'direct',
    paramsAsHash: false,

    api: {
      read: 'NX.direct.coreui_User.readSources'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  }

});
