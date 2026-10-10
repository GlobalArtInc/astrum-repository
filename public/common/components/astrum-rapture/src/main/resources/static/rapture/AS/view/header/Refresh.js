
/*global Ext*/

/**
 * Refresh button.
 *
 * @since 3.0
 */
Ext.define('AS.view.header.Refresh', {
  extend: 'Ext.button.Button',
  alias: 'widget.as-header-refresh',
  requires: [
    'AS.I18n'
  ],

  iconCls: 'x-fa fa-sync',

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      tooltip: AS.I18n.get('Header_Refresh_Tooltip'),
      ariaLabel: AS.I18n.get('Header_Refresh_Tooltip')
    });

    this.callParent();
  }
});
