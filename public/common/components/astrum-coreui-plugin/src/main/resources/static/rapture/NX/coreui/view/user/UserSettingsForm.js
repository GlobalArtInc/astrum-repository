
/*global Ext, NX*/

/**
 * User "Settings" form.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.user.UserSettingsForm', {
  extend: 'NX.view.SettingsForm',
  alias: 'widget.nx-coreui-user-settings-form',
  requires: [
    'NX.Conditions',
    'NX.I18n'
  ],

  api: {
    submit: 'NX.direct.coreui_User.update'
  },

  initComponent: function() {
    var me = this;

    me.settingsFormSuccessMessage = me.settingsFormSuccessMessage || function(data) {
      return NX.I18n.get('User_UserSettingsForm_Update_Success') + Ext.htmlEncode(data['userId']);
    };

    me.editableMarker = NX.I18n.get('User_UserSettingsForm_Update_Error');

    me.editableCondition = me.editableCondition || NX.Conditions.and(
        NX.Conditions.isPermitted('astrum:users:update'),
        NX.Conditions.formHasRecord('nx-coreui-user-settings-form', function(model) {
          return !model.get('external');
        })
    );

    me.items = [
      {
        name: 'userId',
        itemId: 'userId',
        readOnly: true,
        fieldLabel: NX.I18n.get('User_UserSettingsForm_ID_FieldLabel'),
        helpText: NX.I18n.get('User_UserSettingsForm_ID_HelpText')
      },
      { name: 'version', xtype: 'hiddenfield' },
      {
        name: 'firstName',
        fieldLabel: NX.I18n.get('User_UserSettingsForm_FirstName_FieldLabel')
      },
      {
        name: 'lastName',
        fieldLabel: NX.I18n.get('User_UserSettingsForm_LastName_FieldLabel')
      },
      {
        xtype: 'nx-email',
        name: 'email',
        fieldLabel: NX.I18n.get('User_UserSettingsForm_Email_FieldLabel'),
        helpText: NX.I18n.get('User_UserSettingsForm_Email_HelpText')
      },
      {
        xtype: 'combo',
        name: 'status',
        fieldLabel: NX.I18n.get('User_UserSettingsForm_Status_FieldLabel'),
        emptyText: NX.I18n.get('User_UserSettingsForm_Status_EmptyText'),
        allowBlank: false,
        editable: false,
        store: [
          ['active', NX.I18n.get('User_UserSettingsForm_Status_ActiveItem')],
          ['disabled', NX.I18n.get('User_UserSettingsForm_Status_DisabledItem')]
        ],
        queryMode: 'local'
      },
      {
        xtype: 'nx-itemselector',
        name: 'roles',
        itemId: 'roles',
        fieldLabel: NX.I18n.get('User_UserSettingsExternalForm_Roles_FieldLabel'),
        buttons: ['add', 'remove'],
        fromTitle: NX.I18n.get('User_UserSettingsExternalForm_Roles_FromTitle'),
        toTitle: NX.I18n.get('User_UserSettingsExternalForm_Roles_ToTitle'),
        store: 'Role',
        valueField: 'id',
        displayField: 'name',
        delimiter: null
      },
      {
        xtype: 'displayfield',
        name: 'appliedDefaultRole',
        value: me.defaultRoleName(),
        fieldLabel: NX.I18n.get('User_UserSettingsForm_DefaultRole_FieldLabel'),
        labelSeparator: '',
        afterLabelTextTpl: '<div style="font-size:14px;font-weight:normal;font-style:italic;margin-top:9px">' +
          NX.I18n.get('User_UserSettingsForm_DefaultRole_HelpText') + '</div>',
        hidden: !me.defaultRoleName(),
        width: 500,
        margin: '20 0 0', 
      }
    ];

    me.callParent();
  },

  defaultRoleName: function() {
    var role = NX.State.getValue('defaultRole');
    return role && role.name;
  }
});
