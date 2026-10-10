
/*global Ext, AS*/

/**
 * Reconciliation plan summary model.
 */

Ext.define('AS.coreui.model.ReconcilePlans', {
  extend: 'Ext.data.Model',
  fields: [
    {name: 'planCount', type: 'int'},
    {name: 'blobStoreCount', type: 'int'},
    {name: 'repositoryCount', type: 'int'},
    {name: 'minStartDate', type: 'date', dateFormat: 'c'},
    {name: 'maxEndDate', type: 'date', dateFormat: 'c'},
  ]
});
