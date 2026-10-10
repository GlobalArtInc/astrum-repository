
/*global Ext*/

/**
 * Sign-in button.
 *
 * @since 3.0
 */
Ext.define('AS.view.header.SignIn', {
  extend: 'Ext.button.Button',
  alias: 'widget.as-header-signin',
  requires: [
    'AS.I18n'
  ],

  iconCls: 'x-fa fa-sign-in-alt',

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      text: AS.I18n.get('Header_SignIn_Text'),
      tooltip: AS.I18n.get('Header_SignIn_Tooltip')
    });

    this.callParent();
  }

});
