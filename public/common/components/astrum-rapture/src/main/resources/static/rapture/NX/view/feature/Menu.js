
/*global Ext*/

/**
 * Feature menu tree panel.
 *
 * @since 3.0
 */
Ext.define('NX.view.feature.Menu', {
  extend: 'Ext.tree.Panel',
  alias: 'widget.nx-feature-menu',

  width: 300,
  ui: 'nx-feature-menu',

  stateful: true,
  stateId: 'nx-feature-menu-v2',

  store: 'FeatureMenu',
  rootVisible: false,
  sortableColumns: false,
  lines: false,

  ariaRole: 'navigation'
});
