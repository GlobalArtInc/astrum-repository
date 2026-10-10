
/*global Ext, AS*/

/**
 * Task Schedule FieldSet.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.task.TaskScheduleFieldSet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.as-coreui-task-schedulefieldset',
  requires: [
    'AS.I18n'
  ],

  autoHeight: false,
  autoScroll: true,
  collapsed: false,
  defaults: {
    allowBlank: false
  },

  initComponent: function() {
    var me = this;

    me.items = [
      {
        xtype: 'combo',
        name: 'schedule',
        itemId: 'schedule',
        fieldLabel: AS.I18n.get('Task_TaskScheduleFieldSet_Recurrence_FieldLabel'),
        helpText: AS.I18n.get('Task_TaskScheduleFieldSet_Recurrence_HelpText'),
        emptyText: AS.I18n.get('Task_TaskScheduleFieldSet_Recurrence_EmptyText'),
        editable: false,
        store: [
          ['manual', AS.I18n.get('Task_TaskScheduleFieldSet_Recurrence_ManualItem')],
          ['once', AS.I18n.get('Task_TaskScheduleFieldSet_Recurrence_OnceItem')],
          ['hourly', AS.I18n.get('Task_TaskScheduleFieldSet_Recurrence_HourlyItem')],
          ['daily', AS.I18n.get('Task_TaskScheduleFieldSet_Recurrence_DailyItem')],
          ['weekly', AS.I18n.get('Task_TaskScheduleFieldSet_Recurrence_WeeklyItem')],
          ['monthly', AS.I18n.get('Task_TaskScheduleFieldSet_Recurrence_MonthlyItem')],
          ['advanced', AS.I18n.get('Task_TaskScheduleFieldSet_Recurrence_AdvancedItem')]
        ],
        queryMode: 'local',
        listeners: {
          change: function (combo, newValue, oldValue) {
            var form = combo.up('form');

            var taskScheduleFields = me.down('as-coreui-task-schedule-fields');
            if (taskScheduleFields) {
              taskScheduleFields.up().remove(taskScheduleFields);
            }

            if (newValue && newValue !== 'internal') {
              combo.ownerCt.add({ xtype: 'as-coreui-task-schedule-' + newValue });
              form.getForm().checkValidity();
              form.isValid();
            }
          }
        }
      }
    ];

    me.callParent();

  },

  /**
   * @public
   * Exports recurring days.
   * @returns {Array} recurring
   */
  getRecurringDays: function () {
    var me = this,
        days = me.query('checkbox[recurringDayValue]'),
        recurringDays = [];

    Ext.Array.each(days, function (day) {
      if (day.value) {
        recurringDays.push(day.recurringDayValue);
      }
    });

    return recurringDays;
  },

  /**
   * @public
   * Returns start date out of start date/time.
   * @returns {Date} start date
   */
  getStartDate: function () {
    var me = this,
        startDate = me.down('#startDate'),
        startTime = me.down('#startTime');

    if (startDate && startTime) {
      startDate = startDate.getValue();
      startTime = startTime.getValue();
      if (startDate && startTime) {
        startDate.setHours(startTime.getHours(), startTime.getMinutes(), 0, 0);
      }
    }
    return startDate;
  },

  /**
   * @public
   * Set values of start date and time
   */
  setStartDate: function(date) {
    var me = this,
      startDate = me.down('#startDate'),
      startTime = me.down('#startTime');

    if (startDate && startTime) {
      startDate.setValue(date);
      startTime.setValue(date);

      // startTime is not part of the form data load and therefore is marked 'dirty' otherwise
      startTime.resetOriginalValue();
    }
  },

  /**
   * @public
   * Set values of recurring days checkboxes
   */
  setRecurringDays: function(recurringDays) {
    var me = this;

    Ext.Array.each(recurringDays, function(day) {
      var checkbox = me.down('checkbox[name=recurringDay-' + day + ']');
      if (checkbox) {
        checkbox.setValue(true);
        //checkboxes are not part of the form data load and therefore is marked 'dirty' otherwise
        checkbox.resetOriginalValue();
      }
    });
  }
});
