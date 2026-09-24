
/*global Ext, NX*/

/**
 * @since 3.17
 */
Ext.define('NX.onboarding.model.Onboarding', {
  extend: 'Ext.data.Model',
  idProperty: 'type',
  fields: [
    {name: 'type', type: 'string', sortType: 'asUCText'}
  ]
});
