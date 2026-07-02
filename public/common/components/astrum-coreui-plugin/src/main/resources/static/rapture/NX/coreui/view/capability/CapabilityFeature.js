
/*global Ext, NX*/

/**
 * Capability feature panel.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.capability.CapabilityFeature', {
  extend: 'NX.view.drilldown.Drilldown',
  alias: 'widget.nx-coreui-capability-feature',
  requires: [
    'NX.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      iconName: 'capability-default',

      masters: [
        { xtype: 'nx-coreui-capability-list' }
      ],

      tabs: [
        { xtype: 'nx-coreui-capability-summary', weight: 10 },
        { xtype: 'nx-coreui-capability-settings', title: NX.I18n.get('Capability_CapabilitySettings_Title'), weight: 20 }
      ],

      nxActions: [
        {
          xtype: 'button',
          text: NX.I18n.get('Capability_CapabilityFeature_Delete_Button'),
          action: 'delete',
          disabled: true,
          iconCls: 'x-fa fa-trash'
        },
        '-',
        {
          xtype: 'button',
          text: NX.I18n.get('Capability_CapabilityFeature_Enable_Button'),
          action: 'enable',
          handler: function(button) {
            button.fireEvent('runaction');
          },
          disabled: true,
          iconCls: 'x-fa fa-play'
        },
        {
          xtype: 'button',
          text: NX.I18n.get('Capability_CapabilityFeature_Disable_Button'),
          action: 'disable',
          handler: function(button) {
            button.fireEvent('runaction');
          },
          disabled: true,
          iconCls: 'x-fa fa-stop'
        }
      ]
    });

    this.callParent();
  }
});
