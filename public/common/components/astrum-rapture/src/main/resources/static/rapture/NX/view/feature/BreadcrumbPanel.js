
/*global Ext*/

/**
 * Breadcrumb panel.
 *
 * @since 3.14
 */
Ext.define('NX.view.feature.BreadcrumbPanel', {
  extend: 'Ext.panel.Panel',
  alias: 'widget.nx-breadcrumb',
  requires: [ 'NX.view.feature.BreadcrumbViewController' ],

  controller: 'breadcrumb',

  layout: 'hbox',
  itemId: 'breadcrumb',
  ui: 'nx-feature-header',
  cls: 'nx-feature-header'
});
