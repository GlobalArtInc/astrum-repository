
Ext.define('NX.coreui.view.task.TaskScopeFieldSet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.nx-coreui-task-scopefieldset',

  config: {
    properties: {}
  },

  requires: [
    'NX.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    const me = this;

    me.items = [
      {
        xtype: 'component',
        html: '<b>' + NX.I18n.get('Task_TaskScope_Label') + '</b>'
      },
      {
        xtype: 'label',
        text: NX.I18n.get('Task_TaskScope_HelpText')
      },
      {
        xtype: 'combo',
        name: 'taskScope',
        editable: false,
        required: false,
        queryMode: 'local',

        store: [
          ['dates', NX.I18n.get('Task_TaskScope_StartEndDates_Item')],
          ['duration', NX.I18n.get('Task_TaskScope_Duration_Item')]
        ],

        listeners: {
          change: function(combo, newValue, oldValue) {
            const taskScopeFields = me.down('nx-coreui-task-scope-fields');
            if (taskScopeFields) {
              taskScopeFields.up().remove(taskScopeFields);
            }

            if (newValue && newValue !== oldValue) {
              me.config.properties['taskScope'] = newValue;

              const comboOwner = combo.ownerCt;
              if (comboOwner) {
                const fieldsSetType = 'nx-coreui-task-scope-' + newValue
                comboOwner.add({xtype: fieldsSetType});
                // pass properties through to the internal fields set
                const fieldsSet = me.down(fieldsSetType);
                if (fieldsSet) {
                  fieldsSet.importProperties(me.config.properties);
                }
              }
            }
          },
        }
      }
    ];

    me.callParent();
  },

  importProperties: function (properties) {
    const me = this;

    // Ensure properties is initialized
    me.config.properties = properties || {};

    if(!properties) {
      return;
    }

    const taskScope = me.config.properties && me.config.properties['taskScope'];
    const scopeCombo = me.down('combo[name="taskScope"]');
    const scopeValue = taskScope || 'duration';
    scopeCombo.setValue(scopeValue);
    scopeCombo.resetOriginalValue();

    // Ensure child fieldset always has the latest properties
    const fieldsSet = me.down('nx-coreui-task-scope-' + scopeValue);
    if (fieldsSet) {
      fieldsSet.importProperties(me.config.properties);
    }
  },

  exportProperties: function() {
    const me = this,
        properties = {};

    if (me.config.properties) {
      const taskScope = me.config.properties['taskScope'];
      if (taskScope) {
        properties['taskScope'] = taskScope;
        const fieldsSet = me.down('nx-coreui-task-scope-' + taskScope);
        if (fieldsSet) {
          Object.assign(properties, fieldsSet.exportProperties());
        }
      }
    }
    return properties;
  },
});
