
/*global Ext, AS*/

/**
 * External user "Settings" form.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.user.UserSettingsExternalForm', {
  extend: 'AS.view.SettingsForm',
  alias: 'widget.as-coreui-user-settings-external-form',
  requires: [
    'AS.Conditions',
    'AS.Dialogs',
    'AS.I18n'
  ],

  api: {
    submit: 'AS.direct.coreui_User.updateRoleMappings'
  },

  initComponent: function() {
    var me = this;

    me.settingsFormSuccessMessage = function(data) {
      return AS.I18n.format('User_UserSettingsForm_UpdateRoles_Success', data['userId']);
    };

    me.editableMarker = AS.I18n.get('User_UserSettingsForm_Update_Error');

    me.editableCondition = me.editableCondition || AS.Conditions.and(
        AS.Conditions.isPermitted('astrum:users:update'),
        AS.Conditions.formHasRecord('as-coreui-user-settings-external-form', function(model) {
          return model.get('external');
        })
    );

    me.items = [
      {
        name: 'userId',
        itemId: 'userId',
        readOnly: true,
        fieldLabel: AS.I18n.get('User_UserSettingsForm_ID_FieldLabel'),
        helpText: AS.I18n.get('User_UserSettingsForm_ID_HelpText'),
        allowBlank: true
      },
      { name: 'realm', xtype: 'hiddenfield' },
      {
        name: 'firstName',
        fieldLabel: AS.I18n.get('User_UserSettingsForm_FirstName_FieldLabel'),
        allowBlank: true,
        readOnly: true,
        submitValue: false
      },
      {
        name: 'lastName',
        fieldLabel: AS.I18n.get('User_UserSettingsForm_LastName_FieldLabel'),
        allowBlank: true,
        readOnly: true,
        submitValue: false
      },
      {
        name: 'email',
        fieldLabel: AS.I18n.get('User_UserSettingsForm_Email_FieldLabel'),
        helpText: AS.I18n.get('User_UserSettingsForm_Email_HelpText'),
        allowBlank: true,
        readOnly: true,
        submitValue: false
      },
      {
        xtype: 'combo',
        name: 'status',
        fieldLabel: AS.I18n.get('User_UserSettingsForm_Status_FieldLabel'),
        editable: false,
        store: [
          ['active', AS.I18n.get('User_UserSettingsForm_Status_ActiveItem')],
          ['disabled', AS.I18n.get('User_UserSettingsForm_Status_DisabledItem')]
        ],
        queryMode: 'local',
        allowBlank: true,
        readOnly: true,
        cls: 'as-combo-disabled',
        submitValue: false
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
        delimiter: null,
        allowBlank: true
      },
      {
        xtype: 'textarea',
        name: 'externalRoles',
        itemId: 'externalRoles',
        fieldLabel: AS.I18n.get('User_UserSettingsExternalForm_ExternalRoles_FieldLabel'),
        helpText: AS.I18n.get('User_UserSettingsExternalForm_ExternalRoles_HelpText'),
        allowBlank: true,
        readOnly: true,
        submitValue: false
      }
    ];

    me.callParent();

    Ext.override(me.down('#externalRoles'), {
      /**
       * @override
       * Join external roles using '\n' so they are shown as a role / line.
       */
      setValue: function(value) {
        var displayValue = Ext.isArray(value) ? value.join('\n') : (value || '');
        this.callParent([displayValue]);
      }
    });

    Ext.override(me.down('#roles'), {
      /**
       * @override
       * Block removal of external roles.
       */
      moveRec: function(add, recs) {
        var externalRoles = me.getRecord().get('externalRoles'),
            canRemove = true;

        if (!add && externalRoles) {
          Ext.Array.each(Ext.Array.from(recs), function(roleModel) {
            if (Ext.Array.contains(externalRoles, roleModel.get('id'))) {
              canRemove = false;
              AS.Dialogs.showInfo(
                  AS.I18n.get('User_UserSettingsExternalForm_Remove_Error'),
                  'External mapped role "' + roleModel.get('name')
                      + '" cannot be removed because is assigned to user by external source'
              );
            }
            return canRemove;
          });
        }

        if (canRemove) {
          this.callParent(arguments);
        }
      }
    });
  }

});
