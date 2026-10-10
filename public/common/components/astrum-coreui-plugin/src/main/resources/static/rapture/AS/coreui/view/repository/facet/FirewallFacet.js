
/*global Ext, AS*/

/**
 * Configuration for GlobalArt Astrum Firewall repository facet.
 */
Ext.define('AS.coreui.view.repository.facet.FirewallFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.as-coreui-repository-firewall-facet',
  requires: [
    'AS.I18n'
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
          ['DISABLED', AS.I18n.get('Repository_Facet_FirewallFacet_Mode_Disabled')],
          ['AUDIT', AS.I18n.get('Repository_Facet_FirewallFacet_Mode_Audit')],
          ['QUARANTINE', AS.I18n.get('Repository_Facet_FirewallFacet_Mode_Quarantine')]
        ];

    if (me.pccsEnabled) {
      modeStore.push(['PCCS', AS.I18n.get('Repository_Facet_FirewallFacet_Mode_PCCS')]);
    }

    me.items = [
      {
        xtype: 'fieldset',
        cls: 'as-form-section',
        title: AS.I18n.get('Repository_Facet_FirewallFacet_Title'),

        items: [
          {
            xtype: 'panel',
            ui: 'as-drilldown-message',
            cls: 'as-drilldown-info',
            iconCls: AS.Icons.cls('drilldown-info', 'x16'),
            title: AS.I18n.get('Repository_Facet_FirewallFacet_Info_Message')
          },
          {
            xtype: 'combo',
            name: 'attributes.firewall.mode',
            itemId: 'firewallMode',
            fieldLabel: AS.I18n.get('Repository_Facet_FirewallFacet_Mode_FieldLabel'),
            helpText: AS.I18n.get('Repository_Facet_FirewallFacet_Mode_HelpText'),
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
