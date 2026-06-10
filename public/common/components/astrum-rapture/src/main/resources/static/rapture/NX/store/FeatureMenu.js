
/*global Ext*/

/**
 * Feature menu tree store.
 *
 * @since 3.0
 */
Ext.define('NX.store.FeatureMenu', {
  extend: 'Ext.data.TreeStore',
  model: 'NX.model.FeatureMenu',

  root: {
    expanded: true,
    text: 'Features'
  },

  proxy: {
    type: 'memory'
  }

});
