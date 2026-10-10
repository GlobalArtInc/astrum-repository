
/*global Ext, AS*/

/**
 * Capability "Settings" form.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.capability.CapabilitySettingsForm', {
  extend: 'AS.view.SettingsForm',
  alias: 'widget.as-coreui-capability-settings-form',
  requires: [
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {
        xtype: 'hiddenfield',
        name: 'id'
      },
      {
        xtype: 'hiddenfield',
        name: 'typeId'
      },
      {
        xtype: 'hiddenfield',
        name: 'notes'
      },
      {
        xtype: 'checkbox',
        name: 'enabled',
        allowBlank: false,
        checked: true,
        inputValue: true,
        hidden: true
      },
      {
        xtype: 'displayfield',
        name: 'enabledLabel',
        value: '',
      },
      {
        xtype: 'as-coreui-formfield-settingsfieldset',
        delimiter: ','
      }
    ];

    me.editableMarker = AS.I18n.get('Capability_CapabilityAdd_Create_Error');

    me.editableCondition = me.editableCondition || AS.Conditions.isPermitted('astrum:capabilities:update');

    me.callParent();
  },

  /**
   * @override
   * Imports capability into settings field set.
   * @param {AS.coreui.model.Capability} model capability model
   */
  loadRecord: function(model) {
    var me = this,
        capabilityTypeModel = AS.getApplication().getStore('CapabilityType').getById(model.get('typeId')),
        settingsFieldSet = me.down('as-coreui-formfield-settingsfieldset');

    me.setEnabledLabel(model);

    me.callParent(arguments);
    if (capabilityTypeModel) {
      settingsFieldSet.importProperties(model.get('properties'), capabilityTypeModel.get('formFields'));
    }
  },

  setEnabledLabel: function(model) {
    var me = this,
        label = me.getForm().findField('enabledLabel'),
        isEnabled = model.get('enabled'),
        isCreate = model.crudState === 'C',
        text

    if (isCreate) {
      label.setVisible(false);
    } else {
      text = isEnabled 
        ? AS.I18n.get('Capability_Settings_Enabled_Label') 
        : AS.I18n.get('Capability_Settings_Disabled_Label');
      me.getForm().findField('enabledLabel').setValue(text);
    }
  },

  /**
   * @override
   * Exports capability from settings field set.
   * @returns {Object} form values
   */
  getValues: function() {
    var me = this,
        values = me.getForm().getValues(),
        capability = {
          id: values.id,
          typeId: values.typeId,
          notes: values.notes,
          enabled: values.enabled,
          properties: {}
        };

    Ext.apply(capability.properties, me.down('as-coreui-formfield-settingsfieldset').exportProperties(values));
    return capability;
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
