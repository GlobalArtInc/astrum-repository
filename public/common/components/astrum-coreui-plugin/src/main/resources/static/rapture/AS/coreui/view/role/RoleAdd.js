
/*global Ext, AS*/

/**
 * Add role window.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.role.RoleAdd', {
  extend: 'AS.view.AddPanel',
  alias: 'widget.as-coreui-role-add',
  requires: [
    'AS.Conditions',
    'AS.I18n'
  ],

  defaultFocus: 'id',

  initComponent: function() {
    var me = this;

    me.settingsForm = {
      xtype: 'as-coreui-role-settings-form',
      api: {
        submit: 'AS.direct.coreui_Role.create'
      },
      settingsFormSuccessMessage: function(data) {
        return AS.I18n.get('Role_RoleAdd_Create_Success') + Ext.htmlEncode(data['name']);
      },
      editableCondition: AS.Conditions.isPermitted('astrum:roles:create'),
      editableMarker: AS.I18n.get('Role_RoleAdd_Create_Error'),
      source: me.source,

      buttons: [
        { text: AS.I18n.get('Role_RoleList_New_Button'), action: 'add', formBind: true, ui: 'as-primary' },
        { text: AS.I18n.get('Add_Cancel_Button'), action: 'back' }
      ]
    };

    me.callParent();

    me.down('#id').setReadOnly(false);
  }

});
