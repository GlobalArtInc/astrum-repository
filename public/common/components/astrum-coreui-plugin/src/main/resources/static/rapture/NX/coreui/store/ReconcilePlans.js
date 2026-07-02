
/*global Ext, NX*/

/**
 * Reconciliation plan summary store.
 */

Ext.define('NX.coreui.store.ReconcilePlans', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.ReconcilePlans',
  proxy: {
    $configStrict: false,
    type: 'rest',
    url: 'service/rest/v1/plan/summary',
    actionMethods: {
      read: 'GET'
    }
  }
});
