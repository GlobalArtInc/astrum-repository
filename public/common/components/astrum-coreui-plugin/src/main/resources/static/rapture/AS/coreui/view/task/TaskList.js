
/*global Ext, AS*/

/**
 * Task grid.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.task.TaskList', {
  extend: 'AS.view.drilldown.Master',
  alias: 'widget.as-coreui-task-list',
  requires: [
    'AS.I18n'
  ],

  stateful: true,
  stateId: 'as-coreui-task-list',

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      store: 'Task',

      columns: [
        {
          xtype: 'as-iconcolumn',
          width: 36,
          iconVariant: 'x16',
          iconName: function () {
            return 'task-default';
          }
        },
        { header: AS.I18n.get('Task_TaskList_Name_Header'), dataIndex: 'name', stateId: 'name', flex: 1, renderer: Ext.htmlEncode },
        { header: AS.I18n.get('Task_TaskList_Type_Header'), dataIndex: 'typeName', stateId: 'typeName', flex: 1, renderer: Ext.htmlEncode },
        { header: AS.I18n.get('Task_TaskList_Status_Header'), dataIndex: 'statusProgress', stateId: 'statusDescription', renderer: Ext.htmlEncode },
        { header: AS.I18n.get('Task_TaskList_Schedule_Header'), dataIndex: 'schedule', stateId: 'schedule', renderer: Ext.htmlEncode },
        { header: AS.I18n.get('Task_TaskList_NextRun_Header'), dataIndex: 'nextRun', stateId: 'nextRun', flex: 1, renderer: Ext.htmlEncode },
        { header: AS.I18n.get('Task_TaskList_LastRun_Header'), dataIndex: 'lastRun', stateId: 'lastRun', flex: 1, renderer: Ext.htmlEncode },
        { header: AS.I18n.get('Task_TaskList_LastResult_Header'), dataIndex: 'lastRunResult', stateId: 'lastRunResult', renderer: Ext.htmlEncode }
      ],

      viewConfig: {
        emptyText: AS.I18n.get('Task_TaskList_EmptyState'),
        deferEmptyText: false
      },

      dockedItems: [{
        xtype: 'as-actions',
        items: [
          {
            xtype: 'button',
            text: AS.I18n.get('Task_TaskList_New_Button'),
            iconCls: 'x-fa fa-plus-circle',
            action: 'new',
            disabled: true
          }
        ]
      }],

      plugins: [
        { ptype: 'gridfilterbox', emptyText: AS.I18n.get('Task_TaskList_Filter_EmptyState') }
      ]
    });

    this.callParent();
  }
});
