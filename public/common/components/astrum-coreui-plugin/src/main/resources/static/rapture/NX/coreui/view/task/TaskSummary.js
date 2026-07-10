
/*global Ext, NX*/

/**
 * Task "Summary" panel.
 *
 * @since 3.1
 */
Ext.define('NX.coreui.view.task.TaskSummary', {
  extend: 'Ext.Panel',
  alias: 'widget.nx-coreui-task-summary',
  requires: [
    'NX.Conditions',
    'NX.I18n'
  ],
  cls: 'nx-hr',

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.title = NX.I18n.get('TaskFeature_Summary_Title');
    me.autoScroll = true;

    me.layout = {
      type: 'vbox',
      align: 'stretch'
    };

    me.items = {
      xtype: 'panel',
      ui: 'nx-inset',
      items: [
        {
          xtype: 'panel',
          ui: 'nx-subsection',
          itemId: 'nx-coreui-task-summary-subsection',
          frame: true,
          layout: 'column',
          weight: 10,
          items: [
            {
              xtype: 'nx-info',
              columnWidth: 1
            }
          ]
        }
      ]
    };

    me.callParent();
  },

  showInfo: function (info) {
    this.down('nx-info').showInfo(info);
  }
});
