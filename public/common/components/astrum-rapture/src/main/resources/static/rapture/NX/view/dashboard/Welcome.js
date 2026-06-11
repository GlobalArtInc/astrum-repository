
/*global Ext, NX*/

/**
 * Welcome dashboard.
 *
 * @since 3.0
 */
Ext.define('NX.view.dashboard.Welcome', {
  extend: 'Ext.container.Container',
  alias: 'widget.nx-dashboard-welcome',
  requires: [
    'NX.Icons'
  ],

  cls: 'nx-iframe-full as-browse-welcome-page',
  width: '100%'
});
