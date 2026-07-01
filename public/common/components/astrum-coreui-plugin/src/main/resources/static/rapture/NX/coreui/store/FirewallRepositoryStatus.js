
/*global Ext, NX*/

/**
 * Firewall repository status store.
 *
 * @since 3.2
 */
Ext.define('NX.coreui.store.FirewallRepositoryStatus', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.FirewallRepositoryStatus',

  loaded: false,

  proxy: {
    type: 'direct',
    paramsAsHash: false,

    api: {
      read: 'NX.direct.firewall_RepositoryStatus.read'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  listeners: {
    beforeload: function() {
      this.loaded = false;
    },
    load: function() {
      this.loaded = true;
    }
  }

});
