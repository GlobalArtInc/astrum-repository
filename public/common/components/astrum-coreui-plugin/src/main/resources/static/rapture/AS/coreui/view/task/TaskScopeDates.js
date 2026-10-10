
/*global Ext, AS*/

/**
 * Task scope Start/End dates fields set.
 */
Ext.define('AS.coreui.view.task.TaskScopeDates', {
  extend: 'AS.coreui.view.task.TaskScopeFields',
  alias: 'widget.as-coreui-task-scope-dates',

  requires: [
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function () {
    const me = this;

    me.items = [
      {
        xtype: 'component',
        html: '<b>' + AS.I18n.get('Task_TaskScope_StartEndDates_FieldLabel') + '</b>'
      },
      {
        xtype: 'label',
        text: AS.I18n.get('Task_TaskScope_StartEndDates_FieldHelpText')
      },
      {
        xtype: 'datefield',
        name: 'reconcileStartDate',
        fieldLabel: AS.I18n.get('Task_TaskScopeDates_StartDate_FieldLabel'),
        allowBlank: true,
        submitValue: false,
        format: 'm/d/Y',
        value: new Date(),
        required: false
      },
      {
        xtype: 'datefield',
        name: 'reconcileEndDate',
        fieldLabel: AS.I18n.get('Task_TaskScopeDates_EndDate_FieldLabel'),
        allowBlank: true,
        required: false,
        format: 'm/d/Y',
        value: new Date(),
        submitValue: false
      }
    ];

    me.callParent();
  },

  validate: function () {},

  importProperties: function (properties) {
    this.setDateValueFor('reconcileStartDate', properties['reconcileStartDate']);
    this.setDateValueFor('reconcileEndDate', properties['reconcileEndDate']);
  },

  exportProperties: function () {
    return {
      'reconcileStartDate' : this.getDateValueFor('reconcileStartDate'),
      'reconcileEndDate' : this.getDateValueFor('reconcileEndDate')
    };
  },

  getDateValueFor: function (fieldName) {
    const field = this.down('datefield[name="' + fieldName + '"]');
    if (field) {
      const val = field.getValue();
      const formatted = Ext.Date.format(val, 'm/d/Y');
      field.resetOriginalValue();
      return formatted;
    }
    return null;
  },

  setDateValueFor: function (fieldName, value) {
    const field = this.down('datefield[name="' + fieldName + '"]');
    if (field) {
      if (value) {
        field.setValue(value);
      } else {
        field.setValue(new Date());
      }
      field.resetOriginalValue();
    }
  }
});
