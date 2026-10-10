
/*global Ext*/

/**
 * Feature menu tree panel.
 *
 * @since 3.0
 */
Ext.define('AS.view.feature.Menu', {
  extend: 'Ext.tree.Panel',
  alias: 'widget.as-feature-menu',

  width: 300,
  ui: 'as-feature-menu',

  stateful: true,
  stateId: 'as-feature-menu-v2',

  store: 'FeatureMenu',
  rootVisible: false,
  sortableColumns: false,
  lines: false,

  ariaRole: 'navigation'
});
