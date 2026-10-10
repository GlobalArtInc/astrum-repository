
/*global Ext, AS*/

/**
 * Role "Settings" form.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.role.RoleSettingsForm', {
  extend: 'AS.view.SettingsForm',
  alias: 'widget.as-coreui-role-settings-form',
  requires: [
    'AS.Conditions',
    'AS.coreui.store.Role',
    'AS.I18n',
    'AS.util.Validator'
  ],

  api: {
    submit: 'AS.direct.coreui_Role.update'
  },

  initComponent: function() {
    var me = this,
        idField,
        roleStore = Ext.create('AS.coreui.store.Role');

    roleStore.load();

    me.settingsFormSuccessMessage = me.settingsFormSuccessMessage || function(data) {
      return AS.I18n.get('Role_RoleSettingsForm_Update_Success') + Ext.String.htmlEncode(data['name']);
    };

    me.editableMarker = AS.I18n.get('Role_RoleSettingsForm_Update_Error');

    me.editableCondition = me.editableCondition || AS.Conditions.and(
        AS.Conditions.isPermitted('astrum:roles:update'),
        AS.Conditions.formHasRecord('as-coreui-role-settings-form', function(model) {
          return !model.get('readOnly');
        })
    );

    if (me.source) {
      idField = {
        xtype: 'combo',
        name: 'id',
        itemId: 'id',
        fieldLabel: AS.I18n.get('Role_RoleSettingsForm_MappedRole_FieldLabel'),
        emptyText: AS.I18n.get('Role_RoleSettingsForm_MappedRole_EmptyText'),
        editable: true,
        store: 'RoleBySource',
        queryMode: 'local',
        displayField: 'name',
        valueField: 'id',
        vtype: 'as-trim'
      };
    }
    else {
      idField = {
        name: 'id',
        itemId: 'id',
        readOnly: true,
        fieldLabel: AS.I18n.get('Role_RoleSettingsForm_RoleID_FieldLabel'),
        vtype: 'as-trim'
      };
    }

    me.items = [
      {
        xtype: 'hiddenfield',
        name: 'version'
      },
      {
        xtype: 'hiddenfield',
        name: 'source',
        value: me.source || 'default'
      },
      idField,
      {
        name: 'name',
        fieldLabel: AS.I18n.get('Role_RoleSettingsForm_Name_FieldLabel'),
        transformRawValue: Ext.htmlDecode
      },
      {
        name: 'description',
        allowBlank: true,
        fieldLabel: AS.I18n.get('Role_RoleSettingsForm_Description_FieldLabel'),
        transformRawValue: Ext.htmlDecode
      },
      {
        xtype: 'as-itemselector',
        name: 'privileges',
        itemId: 'privileges',
        fieldLabel: AS.I18n.get('Role_RoleSettingsForm_Privileges_FieldLabel'),
        allowBlank: true,
        buttons: ['add', 'remove'],
        fromTitle: AS.I18n.get('Role_RoleSettingsForm_Privileges_FromTitle'),
        toTitle: AS.I18n.get('Role_RoleSettingsForm_Privileges_ToTitle'),
        store: 'PrivilegeReference',
        valueField: 'id',
        displayField: 'name',
        delimiter: null
      },
      {
        xtype: 'as-itemselector',
        name: 'roles',
        itemId: 'roles',
        fieldLabel: AS.I18n.get('Role_RoleSettingsForm_Roles_FieldLabel'),
        allowBlank: true,
        buttons: ['add', 'remove'],
        fromTitle: AS.I18n.get('Role_RoleSettingsForm_Roles_FromTitle'),
        toTitle: AS.I18n.get('Role_RoleSettingsForm_Roles_ToTitle'),
        store: roleStore,
        valueField: 'id',
        displayField: 'name',
        delimiter: null
      }
    ];

    me.callParent();

    AS.Conditions.formIs(me, function(form) {
      return !form.isDisabled();
    }).on({
      satisfied: function() {
        Ext.each(me.query('as-itemselector'), function(it) {
          it.show();
        });
      },
      unsatisfied: function() {
        Ext.each(me.query('as-itemselector'), function(it) {
          it.hide();
        });
      },
      scope: me
    });
  }
});
