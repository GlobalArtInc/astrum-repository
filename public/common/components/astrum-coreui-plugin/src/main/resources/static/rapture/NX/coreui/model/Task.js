
/*global Ext, NX*/

/**
 * Task model.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.model.Task', {
  extend: 'Ext.data.Model',
  fields: [
    {name: 'id', type: 'string', sortType: 'asUCText'},
    {name: 'enabled', type: 'boolean'},
    {name: 'name', type: 'string', sortType: 'asUCText'},
    {name: 'typeId', type: 'string', sortType: 'asUCText'},
    {name: 'typeName', type: 'string', sortType: 'asUCText'},
    {name: 'status', type: 'string', sortType: 'asUCText'},
    {name: 'statusDescription', type: 'string', sortType: 'asUCText'},
    {name: 'schedule', type: 'string', sortType: 'asUCText'},
    {name: 'nextRun', type: 'date', dateFormat: 'c' },
    {name: 'lastRun', type: 'date', dateFormat: 'c' },
    {name: 'lastRunResult', type: 'string'},
    {name: 'runnable', type: 'boolean'},
    {name: 'stoppable', type: 'boolean'},
    {name: 'reconcilePlans', type: 'auto'},
    {name: 'alertEmail', type: 'string'},
    {name: 'notificationCondition', type: 'string', defaultValue: 'FAILURE' },
    {name: 'properties', type: 'auto' /*object*/, defaultValue: null },
    {
      name: 'statusProgress',
      type: 'string',
      calculate: function (data) {
        return (data.properties && data.properties['.progress']) || data.statusDescription;
      },
    },
    {name: 'startDate', type: 'date', dateFormat: 'c' },
    {name: 'recurringDays', type: 'auto' /*array*/},
    {name: 'cronExpression', type: 'string'}
  ]
});
