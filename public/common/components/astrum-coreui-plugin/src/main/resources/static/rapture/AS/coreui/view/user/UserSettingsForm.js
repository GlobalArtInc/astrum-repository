
/*global Ext, AS*/

/**
 * User "Settings" form.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.user.UserSettingsForm', {
  extend: 'AS.view.SettingsForm',
  alias: 'widget.as-coreui-user-settings-form',
  requires: [
    'AS.Conditions',
    'AS.I18n'
  ],

  api: {
    submit: 'AS.direct.coreui_User.update'
  },

  initComponent: function() {
    var me = this;

    me.settingsFormSuccessMessage = me.settingsFormSuccessMessage || function(data) {
      return AS.I18n.get('User_UserSettingsForm_Update_Success') + Ext.htmlEncode(data['userId']);
    };

    me.editableMarker = AS.I18n.get('User_UserSettingsForm_Update_Error');

    me.editableCondition = me.editableCondition || AS.Conditions.and(
        AS.Conditions.isPermitted('astrum:users:update'),
        AS.Conditions.formHasRecord('as-coreui-user-settings-form', function(model) {
          return !model.get('external');
        })
    );

    me.items = [
      {
        name: 'userId',
        itemId: 'userId',
        readOnly: true,
        fieldLabel: AS.I18n.get('User_UserSettingsForm_ID_FieldLabel'),
        helpText: AS.I18n.get('User_UserSettingsForm_ID_HelpText')
      },
      { name: 'version', xtype: 'hiddenfield' },
      {
        name: 'firstName',
        fieldLabel: AS.I18n.get('User_UserSettingsForm_FirstName_FieldLabel')
      },
      {
        name: 'lastName',
        fieldLabel: AS.I18n.get('User_UserSettingsForm_LastName_FieldLabel')
      },
      {
        xtype: 'as-email',
        name: 'email',
        fieldLabel: AS.I18n.get('User_UserSettingsForm_Email_FieldLabel'),
        helpText: AS.I18n.get('User_UserSettingsForm_Email_HelpText')
      },
      {
        xtype: 'combo',
        name: 'status',
        fieldLabel: AS.I18n.get('User_UserSettingsForm_Status_FieldLabel'),
        emptyText: AS.I18n.get('User_UserSettingsForm_Status_EmptyText'),
        allowBlank: false,
        editable: false,
        store: [
          ['active', AS.I18n.get('User_UserSettingsForm_Status_ActiveItem')],
          ['disabled', AS.I18n.get('User_UserSettingsForm_Status_DisabledItem')]
        ],
        queryMode: 'local'
      },
      {
        xtype: 'as-itemselector',
        name: 'roles',
        itemId: 'roles',
        fieldLabel: AS.I18n.get('User_UserSettingsExternalForm_Roles_FieldLabel'),
        buttons: ['add', 'remove'],
        fromTitle: AS.I18n.get('User_UserSettingsExternalForm_Roles_FromTitle'),
        toTitle: AS.I18n.get('User_UserSettingsExternalForm_Roles_ToTitle'),
        store: 'Role',
        valueField: 'id',
        displayField: 'name',
        delimiter: null
      },
      {
        xtype: 'displayfield',
        name: 'appliedDefaultRole',
        value: me.defaultRoleName(),
        fieldLabel: AS.I18n.get('User_UserSettingsForm_DefaultRole_FieldLabel'),
        labelSeparator: '',
        afterLabelTextTpl: '<div style="font-size:14px;font-weight:normal;font-style:italic;margin-top:9px">' +
          AS.I18n.get('User_UserSettingsForm_DefaultRole_HelpText') + '</div>',
        hidden: !me.defaultRoleName(),
        width: 500,
        margin: '20 0 0', 
      }
    ];

    me.callParent();
  },

  defaultRoleName: function() {
    var role = AS.State.getValue('defaultRole');
    return role && role.name;
  }
});
