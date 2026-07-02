
/*global Ext, NX*/

/**
 * Capability grid.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.capability.CapabilityList', {
  extend: 'NX.view.drilldown.Master',
  alias: 'widget.nx-coreui-capability-list',
  requires: [
    'NX.I18n',
    'NX.ext.grid.column.Renderers'
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
        xtype: 'nx-iconcolumn',
        width: 36,
        iconVariant: 'x16',
        iconNamePrefix: 'capability-',
        dataIndex: 'state',
        hideable: false
      },
      {
        text: NX.I18n.get('Capability_CapabilityList_Type_Header'),
        dataIndex: 'typeName',
        flex: 1,
        renderer: Ext.htmlEncode
      },
      {
        text: NX.I18n.get('Capability_CapabilityList_Description_Header'),
        dataIndex: 'description',
        flex: 1,
        groupable: false,
        renderer: NX.ext.grid.column.Renderers.optionalData
      },
      {
        text: NX.I18n.get('Capability_CapabilityList_Notes_Header'),
        dataIndex: 'notes',
        flex: 1,
        renderer: NX.ext.grid.column.Renderers.optionalData
      }
    ];

    me.viewConfig = {
      emptyText: NX.I18n.get('Capability_CapabilityList_EmptyText'),
      deferEmptyText: false,
      getRowClass: function (record) {
        if (record.get('enabled') && !record.get('active')) {
          return 'nx-red-marker';
        }
      }
    };

    me.dockedItems = [{
      xtype: 'nx-actions',
      items: [
        {
          xtype: 'button',
          text: NX.I18n.get('Capability_CapabilityList_New_Button'),
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
      { ptype: 'gridfilterbox', emptyText: NX.I18n.get('Capability_CapabilityList_Filter_EmptyText') }
    ];

    me.originalColumns = me.columns;

    me.callParent();
  }

});
