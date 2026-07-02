
/*global Ext, NX*/

/**
 * Select capability type window.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.capability.CapabilitySelectType', {
  extend: 'NX.view.drilldown.Master',
  alias: 'widget.nx-coreui-capability-selecttype',
  requires: [
    'NX.I18n'
  ],

  initComponent: function() {
    var me = this;

    me.store = 'CapabilityType';

    me.cls = 'nx-hr';

    me.columns = [
      {
        xtype: 'nx-iconcolumn',
        width: 36,
        iconVariant: 'x16',
        iconName: function() {
          return 'capability-default';
        }
      },
      { header: NX.I18n.get('Capability_CapabilitySelectType_Type_Header'), dataIndex: 'name', flex: 1 },
      { header: NX.I18n.get('Capability_CapabilitySelectType_Description_Header'), dataIndex: 'about', flex: 2,
        renderer: function(val) {
          var i;
          if (val) {
            i = val.indexOf('.');
            if (i > 0) {
              val = val.substring(0, i);
            }
            // replace HTML
            return val.replace(/(<([^>]+)>)/ig, '');
          }
          return val;
        }
      }
    ];

    me.on('afterrender', function(grid) {
        var view = grid.getView();
        grid.tip = Ext.create('Ext.tip.ToolTip', {
          target: view.el,
          delegate: view.itemSelector,
          trackMouse: true,
          renderTo: Ext.getBody(),
          dismissDelay: 0,
          listeners: {
            beforeshow: function updateTipBody(tip) {
              tip.update(view.getRecord(tip.triggerElement).get('about'));
            }
          }
        });
      }
    );

    me.on('destroy', function(grid) {
      if (grid.tip) {
        grid.tip.destroy();
      }
    });

    me.callParent();
  }

});
