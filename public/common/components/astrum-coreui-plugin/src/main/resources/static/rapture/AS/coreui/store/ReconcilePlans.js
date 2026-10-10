
/*global Ext, AS*/

/**
 * Reconciliation plan summary store.
 */

Ext.define('AS.coreui.store.ReconcilePlans', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.ReconcilePlans',
  proxy: {
    $configStrict: false,
    type: 'rest',
    url: 'service/rest/v1/plan/summary',
    actionMethods: {
      read: 'GET'
    }
  }
});
