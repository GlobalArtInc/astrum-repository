
/*global Ext, NX*/

/**
 * Task grid.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.task.TaskList', {
  extend: 'NX.view.drilldown.Master',
  alias: 'widget.nx-coreui-task-list',
  requires: [
    'NX.I18n'
  ],

  stateful: true,
  stateId: 'nx-coreui-task-list',

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      store: 'Task',

      columns: [
        {
          xtype: 'nx-iconcolumn',
          width: 36,
          iconVariant: 'x16',
          iconName: function () {
            return 'task-default';
          }
        },
        { header: NX.I18n.get('Task_TaskList_Name_Header'), dataIndex: 'name', stateId: 'name', flex: 1, renderer: Ext.htmlEncode },
        { header: NX.I18n.get('Task_TaskList_Type_Header'), dataIndex: 'typeName', stateId: 'typeName', flex: 1, renderer: Ext.htmlEncode },
        { header: NX.I18n.get('Task_TaskList_Status_Header'), dataIndex: 'statusProgress', stateId: 'statusDescription', renderer: Ext.htmlEncode },
        { header: NX.I18n.get('Task_TaskList_Schedule_Header'), dataIndex: 'schedule', stateId: 'schedule', renderer: Ext.htmlEncode },
        { header: NX.I18n.get('Task_TaskList_NextRun_Header'), dataIndex: 'nextRun', stateId: 'nextRun', flex: 1, renderer: Ext.htmlEncode },
        { header: NX.I18n.get('Task_TaskList_LastRun_Header'), dataIndex: 'lastRun', stateId: 'lastRun', flex: 1, renderer: Ext.htmlEncode },
        { header: NX.I18n.get('Task_TaskList_LastResult_Header'), dataIndex: 'lastRunResult', stateId: 'lastRunResult', renderer: Ext.htmlEncode }
      ],

      viewConfig: {
        emptyText: NX.I18n.get('Task_TaskList_EmptyState'),
        deferEmptyText: false
      },

      dockedItems: [{
        xtype: 'nx-actions',
        items: [
          {
            xtype: 'button',
            text: NX.I18n.get('Task_TaskList_New_Button'),
            iconCls: 'x-fa fa-plus-circle',
            action: 'new',
            disabled: true
          }
        ]
      }],

      plugins: [
        { ptype: 'gridfilterbox', emptyText: NX.I18n.get('Task_TaskList_Filter_EmptyState') }
      ]
    });

    this.callParent();
  }
});
