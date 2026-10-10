
/*global Ext, AS*/

/**
 * User feature panel.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.user.UserFeature', {
  extend: 'AS.view.drilldown.Drilldown',
  alias: 'widget.as-coreui-user-feature',
  requires: [
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      iconName: 'user-default',

      masters: [
        { xtype: 'as-coreui-user-list' }
      ],

      tabs: [
        { xtype: 'as-coreui-user-settings', title: AS.I18n.get('User_UserFeature_Settings_Title'), weight: 10 }
      ],

      asActions: [
        { xtype: 'button', text: AS.I18n.get('User_UserFeature_Delete_Button'), iconCls: 'x-fa fa-trash', action: 'delete', disabled: true },
        { xtype: 'button', text: AS.I18n.get('User_UserFeature_ChangePasswordItem'), iconCls: 'x-fa fa-key', action: 'setpassword', disabled: true}
      ]
    });

    this.callParent();
  }
});
