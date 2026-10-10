
/*global Ext*/

/**
 * Feature menu tree store.
 *
 * @since 3.0
 */
Ext.define('AS.store.FeatureMenu', {
  extend: 'Ext.data.TreeStore',
  model: 'AS.model.FeatureMenu',

  root: {
    expanded: true,
    text: 'Features'
  },

  proxy: {
    type: 'memory'
  }

});
