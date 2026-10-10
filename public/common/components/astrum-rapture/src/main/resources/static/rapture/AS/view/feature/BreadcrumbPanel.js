
/*global Ext*/

/**
 * Breadcrumb panel.
 *
 * @since 3.14
 */
Ext.define('AS.view.feature.BreadcrumbPanel', {
  extend: 'Ext.panel.Panel',
  alias: 'widget.as-breadcrumb',
  requires: [ 'AS.view.feature.BreadcrumbViewController' ],

  controller: 'breadcrumb',

  layout: 'hbox',
  itemId: 'breadcrumb',
  ui: 'as-feature-header',
  cls: 'as-feature-header'
});
