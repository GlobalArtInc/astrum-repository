
/*global Ext, AS*/

/**
 * Add privilege window.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.privilege.PrivilegeAdd', {
  extend: 'AS.view.AddPanel',
  alias: 'widget.as-coreui-privilege-add',
  requires: [
    'AS.I18n'
  ],

  defaultFocus: 'name',

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    Ext.apply(me, {
      settingsForm: {
        xtype: 'as-coreui-privilege-settings-form',
        editableCondition: AS.Conditions.isPermitted('astrum:privileges:create'),
        editableMarker: AS.I18n.get('Privilege_PrivilegeSettingsForm_Update_Error'),

        buttons: [
          {text: AS.I18n.get('Privilege_PrivilegeList_New_Button'), action: 'add', formBind: true, ui: 'as-primary'},
          {text: AS.I18n.get('Add_Cancel_Button'), action: 'back'}
        ]
      }
    });

    me.callParent();

    me.down('#name').setReadOnly(false);
  }

});
