
/*global Ext*/

/**
 * Sign-out button.
 *
 * @since 3.0
 */
Ext.define('NX.view.header.SignOut', {
  extend: 'Ext.button.Button',
  alias: 'widget.nx-header-signout',
  requires: [
    'NX.I18n'
  ],

  iconCls: 'x-fa fa-sign-out-alt',

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      text: NX.I18n.get('Header_SignOut_Text'),
      tooltip: NX.I18n.get('Header_SignOut_Tooltip'),
      hidden: true
    });

    this.callParent();
  }

});
