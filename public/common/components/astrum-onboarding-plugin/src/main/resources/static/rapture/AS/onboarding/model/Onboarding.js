
/*global Ext, AS*/

/**
 * @since 3.17
 */
Ext.define('AS.onboarding.model.Onboarding', {
  extend: 'Ext.data.Model',
  idProperty: 'type',
  fields: [
    {name: 'type', type: 'string', sortType: 'asUCText'}
  ]
});
