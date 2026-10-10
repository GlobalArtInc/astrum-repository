
/*global Ext, AS*/

/**
 * User sources store.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.store.UserSource', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.Reference',

  proxy: {
    type: 'direct',
    paramsAsHash: false,

    api: {
      read: 'AS.direct.coreui_User.readSources'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  }

});
