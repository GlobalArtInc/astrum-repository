
/*global Ext, AS*/

/**
 * Select task type window.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.task.TaskSelectType', {
  extend: 'AS.view.drilldown.Master',
  alias: 'widget.as-coreui-task-selecttype',
  requires: [
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.columns = [
      {
        xtype: 'as-iconcolumn',
        width: 36,
        iconVariant: 'x16',
        iconName: function() {
          return 'task-default';
        }
      },
      { header: AS.I18n.get('Task_TaskSelectType_Name_Header'), dataIndex: 'name', flex: 1 }
    ];

    me.plugins = [
      { ptype: 'gridfilterbox', emptyText: AS.I18n.get('Task_TaskSelectType_Filter_EmptyText') }
    ];

    me.store = Ext.create('AS.coreui.store.TaskType');
    var filters = [{property: 'exposed', value: true}];
    me.filterTasksIfCreated(filters,
        ['blobstore.planReconciliation', 'blobstore.executeReconciliationPlan']);
    me.store.addFilter(filters, false);

    // Add a white background behind the filter, to make it look like part of the header
    me.dockedItems = [
      {
        xtype: 'as-actions'
      }
    ];

    me.callParent();
  },
  filterTasksIfCreated: function(filters, typeIds) {
    var me = this;
    typeIds.forEach(function(typeId) {
      if (me.isTaskCreated(typeId)) {
        filters.push({
          filterFn: function(record) {
            return record.get('id') !== typeId;
          }
        });
      }
    });
  },
  isTaskCreated: function(typeId) {
    var taskListComponent = Ext.ComponentQuery.query('as-coreui-task-list')[0];
    if (taskListComponent) {
      var taskStore = taskListComponent.getStore("Task");
      if (taskStore) {
        var tasks = taskStore.getData().items;
        for (var i = 0; i < tasks.length; i++) {
          if (tasks[i].data.typeId === typeId) {
            return true;
          }
        }
      }
    }
    return false;
  }

});
