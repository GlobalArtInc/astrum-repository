
/*global Ext*/

/**
 * Refresh button.
 *
 * @since 3.0
 */
Ext.define('NX.view.header.Refresh', {
  extend: 'Ext.button.Button',
  alias: 'widget.nx-header-refresh',
  requires: [
    'NX.I18n'
  ],

  iconCls: 'x-fa fa-sync',

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      tooltip: NX.I18n.get('Header_Refresh_Tooltip'),
      ariaLabel: NX.I18n.get('Header_Refresh_Tooltip')
    });

    this.callParent();
  }
});
