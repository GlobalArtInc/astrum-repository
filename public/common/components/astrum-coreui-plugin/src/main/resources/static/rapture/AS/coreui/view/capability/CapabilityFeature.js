
/*global Ext, AS*/

/**
 * Capability feature panel.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.capability.CapabilityFeature', {
  extend: 'AS.view.drilldown.Drilldown',
  alias: 'widget.as-coreui-capability-feature',
  requires: [
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      iconName: 'capability-default',

      masters: [
        { xtype: 'as-coreui-capability-list' }
      ],

      tabs: [
        { xtype: 'as-coreui-capability-summary', weight: 10 },
        { xtype: 'as-coreui-capability-settings', title: AS.I18n.get('Capability_CapabilitySettings_Title'), weight: 20 }
      ],

      asActions: [
        {
          xtype: 'button',
          text: AS.I18n.get('Capability_CapabilityFeature_Delete_Button'),
          action: 'delete',
          disabled: true,
          iconCls: 'x-fa fa-trash'
        },
        '-',
        {
          xtype: 'button',
          text: AS.I18n.get('Capability_CapabilityFeature_Enable_Button'),
          action: 'enable',
          handler: function(button) {
            button.fireEvent('runaction');
          },
          disabled: true,
          iconCls: 'x-fa fa-play'
        },
        {
          xtype: 'button',
          text: AS.I18n.get('Capability_CapabilityFeature_Disable_Button'),
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
