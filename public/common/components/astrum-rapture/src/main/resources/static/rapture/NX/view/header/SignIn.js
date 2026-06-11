
/*global Ext*/

/**
 * Sign-in button.
 *
 * @since 3.0
 */
Ext.define('NX.view.header.SignIn', {
  extend: 'Ext.button.Button',
  alias: 'widget.nx-header-signin',
  requires: [
    'NX.I18n'
  ],

  iconCls: 'x-fa fa-sign-in-alt',

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      text: NX.I18n.get('Header_SignIn_Text'),
      tooltip: NX.I18n.get('Header_SignIn_Tooltip')
    });

    this.callParent();
  }

});
