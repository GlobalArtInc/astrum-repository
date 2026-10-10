
/*global Ext, AS*/

/**
 * Privilege "Settings" form.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.privilege.PrivilegeSettingsForm', {
  extend: 'AS.view.SettingsForm',
  alias: 'widget.as-coreui-privilege-settings-form',
  requires: [
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    Ext.apply(me, {
          items: [
            {
              xtype: 'hiddenfield',
              name: 'id'
            },
            {
              xtype: 'hiddenfield',
              name: 'version'
            },
            {
              xtype: 'hiddenfield',
              name: 'type'
            },
            {
              xtype: 'textfield',
              fieldLabel: AS.I18n.get('Privilege_PrivilegeSettingsForm_Name_FieldLabel'),
              name: 'name',
              itemId: 'name',
              allowBlank: false,
              readOnly: true
            },
            {
              xtype: 'textfield',
              fieldLabel: AS.I18n.get('Privilege_PrivilegeSettingsForm_Description_FieldLabel'),
              name: 'description',
              allowBlank: true,
              transformRawValue: Ext.htmlDecode
            },
            {
              xtype: 'as-coreui-formfield-settingsfieldset'
            }
          ],

          settingsFormSuccessMessage: me.settingsFormSuccessMessage || function(data) {
            return AS.I18n.format('Privilege_PrivilegeSettingsForm_Update_Success', data['name']);
          },

          editableMarker: me.editableMarker || AS.I18n.get('Privilege_PrivilegeSettingsForm_Update_Error'),

          editableCondition: me.editableCondition || AS.Conditions.and(
              AS.Conditions.isPermitted('astrum:privileges:update'),
              AS.Conditions.formHasRecord('as-coreui-privilege-settings-form', function(model) {
                return !model.get('readOnly');
              })
          )
        }
    );

    me.callParent();
  },

  /**
   * @override
   * Imports privilege into settings field set.
   * @param {AS.coreui.model.Capability} model capability model
   */
  loadRecord: function(model) {
    var me = this,
        store = AS.getApplication().getStore('PrivilegeType'),
        privilegeTypeModel = store.getById(model.get('type')),
        settingsFieldSet = me.down('as-coreui-formfield-settingsfieldset');

    me.model = model;

    if (privilegeTypeModel) {
      settingsFieldSet.importProperties(model.get('properties'), privilegeTypeModel.get('formFields'), me.editableCondition);
    }
    else {
      store.on({
        load: function() {
          privilegeTypeModel = store.getById(model.get('type'));
          if (me.model === model && privilegeTypeModel) {
            settingsFieldSet.importProperties(model.get('properties'), privilegeTypeModel.get('formFields'),
                me.editableCondition);
          }
        },
        single: true
      });
    }
    me.callParent(arguments);
  },

  /**
   * @override
   * Exports privilege from settings field set.
   * @returns {Object} form values
   */
  getValues: function() {
    var me = this,
        values = me.getForm().getFieldValues(),
        privilege = {
          id: values.id,
          name: values.name,
          description: values.description,
          version: values.version,
          type: values.type,
          readOnly: values.readOnly,
          properties: {}
        };

    Ext.apply(privilege.properties, me.down('as-coreui-formfield-settingsfieldset').exportProperties(values));
    return privilege;
  },

  /**
   * Mark fields in this form invalid in bulk.
   * @param {Object/Object[]/Ext.data.Errors} errors
   * Either an array in the form `[{id:'fieldId', msg:'The message'}, ...]`,
   * an object hash of `{id: msg, id2: msg2}`, or a {@link Ext.data.Errors} object.
   */
  markInvalid: function(errors) {
    this.down('as-coreui-formfield-settingsfieldset').markInvalid(errors);
  }

});
