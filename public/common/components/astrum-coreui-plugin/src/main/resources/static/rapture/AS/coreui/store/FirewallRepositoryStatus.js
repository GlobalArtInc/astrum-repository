
/*global Ext, AS*/

/**
 * Firewall repository status store.
 *
 * @since 3.2
 */
Ext.define('AS.coreui.store.FirewallRepositoryStatus', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.FirewallRepositoryStatus',

  loaded: false,

  proxy: {
    type: 'direct',
    paramsAsHash: false,

    api: {
      read: 'AS.direct.firewall_RepositoryStatus.read'
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
