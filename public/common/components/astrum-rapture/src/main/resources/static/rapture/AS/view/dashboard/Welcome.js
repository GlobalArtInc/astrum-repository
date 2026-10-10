
/*global Ext, AS*/

/**
 * Welcome dashboard.
 *
 * @since 3.0
 */
Ext.define('AS.view.dashboard.Welcome', {
  extend: 'Ext.container.Container',
  alias: 'widget.as-dashboard-welcome',
  requires: [
    'AS.Icons'
  ],

  cls: 'as-iframe-full as-browse-welcome-page',
  width: '100%'
});
