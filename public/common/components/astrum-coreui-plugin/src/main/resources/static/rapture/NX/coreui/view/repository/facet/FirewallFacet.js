
/*global Ext, NX*/

/**
 * Configuration for GlobalArt Astrum Firewall repository facet.
 */
Ext.define('NX.coreui.view.repository.facet.FirewallFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.nx-coreui-repository-firewall-facet',
  requires: [
    'NX.I18n'
  ],

  /**
   * Set to true for repository formats that support PCCS mode (npm and pypi).
   */
  pccsEnabled: false,

  /**
   * @override
   */
  initComponent: function() {
    var me = this,
        modeStore = [
          ['DISABLED', NX.I18n.get('Repository_Facet_FirewallFacet_Mode_Disabled')],
          ['AUDIT', NX.I18n.get('Repository_Facet_FirewallFacet_Mode_Audit')],
          ['QUARANTINE', NX.I18n.get('Repository_Facet_FirewallFacet_Mode_Quarantine')]
        ];

    if (me.pccsEnabled) {
      modeStore.push(['PCCS', NX.I18n.get('Repository_Facet_FirewallFacet_Mode_PCCS')]);
    }

    me.items = [
      {
        xtype: 'fieldset',
        cls: 'nx-form-section',
        title: NX.I18n.get('Repository_Facet_FirewallFacet_Title'),

        items: [
          {
            xtype: 'panel',
            ui: 'nx-drilldown-message',
            cls: 'nx-drilldown-info',
            iconCls: NX.Icons.cls('drilldown-info', 'x16'),
            title: NX.I18n.get('Repository_Facet_FirewallFacet_Info_Message')
          },
          {
            xtype: 'combo',
            name: 'attributes.firewall.mode',
            itemId: 'firewallMode',
            fieldLabel: NX.I18n.get('Repository_Facet_FirewallFacet_Mode_FieldLabel'),
            helpText: NX.I18n.get('Repository_Facet_FirewallFacet_Mode_HelpText'),
            editable: false,
            store: modeStore,
            queryMode: 'local',
            allowBlank: true
          }
        ]
      }
    ];

    me.callParent();
  }
});
