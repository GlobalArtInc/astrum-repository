
/*global Ext, NX*/

/**
 * Health Check repository status store.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.store.HealthCheckRepositoryStatus', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.HealthCheckRepositoryStatus',

  loaded: false,

  proxy: {
    type: 'direct',
    paramsAsHash: false,

    api: {
      read: 'NX.direct.healthcheck_Status.read'
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
