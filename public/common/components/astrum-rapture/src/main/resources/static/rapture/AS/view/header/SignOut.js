
/*global Ext*/

/**
 * Sign-out button.
 *
 * @since 3.0
 */
Ext.define('AS.view.header.SignOut', {
  extend: 'Ext.button.Button',
  alias: 'widget.as-header-signout',
  requires: [
    'AS.I18n'
  ],

  iconCls: 'x-fa fa-sign-out-alt',

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      text: AS.I18n.get('Header_SignOut_Text'),
      tooltip: AS.I18n.get('Header_SignOut_Tooltip'),
      hidden: true
    });

    this.callParent();
  }

});
