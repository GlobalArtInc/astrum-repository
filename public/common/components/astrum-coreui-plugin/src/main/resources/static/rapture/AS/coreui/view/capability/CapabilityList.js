
/*global Ext, AS*/

/**
 * Capability grid.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.capability.CapabilityList', {
  extend: 'AS.view.drilldown.Master',
  alias: 'widget.as-coreui-capability-list',
  requires: [
    'AS.I18n',
    'AS.ext.grid.column.Renderers'
  ],

  /**
   * Copy of original column configuration, to support adding dynamic tag columns.
   *
   * @cfg
   */
  originalColumns: undefined,

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.store = 'Capability';

    me.columns = [
      {
        xtype: 'as-iconcolumn',
        width: 36,
        iconVariant: 'x16',
        iconNamePrefix: 'capability-',
        dataIndex: 'state',
        hideable: false
      },
      {
        text: AS.I18n.get('Capability_CapabilityList_Type_Header'),
        dataIndex: 'typeName',
        flex: 1,
        renderer: Ext.htmlEncode
      },
      {
        text: AS.I18n.get('Capability_CapabilityList_Description_Header'),
        dataIndex: 'description',
        flex: 1,
        groupable: false,
        renderer: AS.ext.grid.column.Renderers.optionalData
      },
      {
        text: AS.I18n.get('Capability_CapabilityList_Notes_Header'),
        dataIndex: 'notes',
        flex: 1,
        renderer: AS.ext.grid.column.Renderers.optionalData
      }
    ];

    me.viewConfig = {
      emptyText: AS.I18n.get('Capability_CapabilityList_EmptyText'),
      deferEmptyText: false,
      getRowClass: function (record) {
        if (record.get('enabled') && !record.get('active')) {
          return 'as-red-marker';
        }
      }
    };

    me.dockedItems = [{
      xtype: 'as-actions',
      items: [
        {
          xtype: 'button',
          text: AS.I18n.get('Capability_CapabilityList_New_Button'),
          action: 'new',
          disabled: true,
          iconCls: 'x-fa fa-plus-circle'
        }
      ]
    }];

    me.features = [
      {
        ftype: 'grouping',
        groupHeaderTpl: '{[values.name === "" ? "No " + values.columnName : values.name + " " + values.columnName]}'
      }
    ];

    me.plugins = [
      { ptype: 'gridfilterbox', emptyText: AS.I18n.get('Capability_CapabilityList_Filter_EmptyText') }
    ];

    me.originalColumns = me.columns;

    me.callParent();
  }

});
