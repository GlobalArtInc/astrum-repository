
/*global Ext, AS*/

/**
 * Repository "Settings" form.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.repository.RepositorySettingsForm', {
  extend: 'AS.view.SettingsForm',
  alias: 'widget.as-coreui-repository-settings-form',
  requires: [
    'AS.Conditions',
    'AS.I18n',
    'AS.util.Validator'
  ],

  api: {
    submit: 'AS.direct.coreui_Repository.update'
  },

  initComponent: function() {
    var me = this,
        permittedCondition;

    me.addListener('remotevalidation', me.handleRemoteValidationError);

    me.settingsFormSuccessMessage = function(data) {
      return AS.I18n.get('Repository_RepositorySettingsForm_Update_Success') + data['name'];
    };

    me.editableMarker = AS.I18n.get('Repository_RepositorySettingsForm_Update_Error');

    if (!me.editableCondition) {
      me.editableCondition = AS.Conditions.and(
          permittedCondition = AS.Conditions.isPermitted('astrum:repository-admin:*:*:edit'),
          AS.Conditions.formHasRecord('as-coreui-repository-settings-form', function(model) {
            permittedCondition.setPermission(
                'astrum:repository-admin:' + model.get('format') + ':' + model.get('name') + ':edit'
            );
            return true;
          })
      );
    }

    me.items = me.items || [];
    Ext.Array.insert(me.items, 0, [
      {
        xtype: 'fieldset',
        cls: 'as-form-section as-no-title',

        items: [
          {
            xtype: 'textfield',
            cls: 'as-no-border',
            name: 'name',
            itemId: 'name',
            labelAlign: 'left',
            fieldLabel: AS.I18n.get('Repository_RepositorySettingsForm_Name_FieldLabel'),
            helpText: AS.I18n.get('Repository_RepositorySettingsForm_Name_HelpText'),
            readOnly: true,
            allowBlank: false,
            vtype: 'as-name'
          },
          {
            xtype: 'textfield',
            cls: 'as-no-border',
            name: 'format',
            itemId: 'format',
            labelAlign: 'left',
            fieldLabel: AS.I18n.get('Repository_RepositorySettingsForm_Format_FieldLabel'),
            helpText: AS.I18n.get('Repository_RepositorySettingsForm_Format_HelpText'),
            readOnly: true
          },
          {
            xtype: 'textfield',
            cls: 'as-no-border',
            name: 'type',
            itemId: 'type',
            labelAlign: 'left',
            fieldLabel: AS.I18n.get('Repository_RepositorySettingsForm_Type_FieldLabel'),
            helpText: AS.I18n.get('Repository_RepositorySettingsForm_Type_HelpText'),
            readOnly: true
          },

          {
            xtype: 'textfield',
            cls: 'as-no-border',
            name: 'url',
            itemId: 'url',
            labelAlign: 'left',
            fieldLabel: AS.I18n.get('Repository_RepositorySettingsForm_URL_FieldLabel'),
            helpText: AS.I18n.get('Repository_RepositorySettingsForm_URL_HelpText'),
            readOnly: true
          },
          {
            xtype: 'checkbox',
            name: 'online',
            itemId: 'online',
            labelAlign: 'left',
            fieldLabel: AS.I18n.get('Repository_RepositorySettingsForm_Online_FieldLabel'),
            helpText: AS.I18n.get('Repository_RepositorySettingsForm_Online_HelpText'),
            value: true
          }
        ]
      }
    ]);

    me.callParent();

    //map repository attributes raw map structure to/from a flattened representation
    Ext.override(me.getForm(), {
      getValues: function() {
        return me.doGetValues(this.callParent(arguments));
        },
      setValues: function(values) { 
        me.doSetValues(values);
        this.callParent(arguments);
      }
    });
  },

  doGetValues: function(values) {
    var processed = { attributes: {} };

    Ext.Object.each(values, function(key, value) {
      var segments = key.split('.'),
          parent = processed;

      Ext.each(segments, function(segment, pos) {
        if (pos === segments.length - 1) {
          parent[segment] = value;
        }
        else {
          if (!parent[segment]) {
            parent[segment] = {};
          }
          parent = parent[segment];
        }
      });
    });

    return processed;
  },

  doSetValues: function(values) {
    var process = function(child, prefix) {
      Ext.Object.each(child, function(key, value) {
        var newPrefix = (prefix ? prefix + '.' : '') + key;
        if (Ext.isObject(value)) {
          process(value, newPrefix);
        }
        else {
          values[newPrefix] = value;
        }
      });
    };

    process(values);
  },

  handleRemoteValidationError: function(validationMap) {
    Object.keys(validationMap).forEach(function (key) {
      const errorMsg = validationMap[key];

      if (!key.startsWith("attributes.")) {
        key = "attributes." + key;
      }

      Ext.ComponentQuery.query('form[settingsForm=true] [name=' + key + ']')
        .forEach(function(component) {
          component.markInvalid(errorMsg);
        });
    });
  }
});
