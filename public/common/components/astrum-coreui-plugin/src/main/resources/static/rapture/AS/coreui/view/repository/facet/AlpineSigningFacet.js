
/**
 * Configuration for Alpine repository signing.
 *
 * @since 3.95
 */
Ext.define('AS.coreui.view.repository.facet.AlpineSigningFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.as-coreui-repository-alpinesigning-facet',
  requires: [
    'AS.I18n'
  ],
  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {
        xtype: 'fieldset',
        cls: 'as-form-section',
        title: AS.I18n.get('Repository_Facet_AlpineSigningFacet_Title'),
        items: [
          {
            xtype:'textareafield',
            name: 'attributes.alpineSigning.keypair',
            fieldLabel: AS.I18n.get('Repository_Facet_AlpineSigningFacet_Keypair_FieldLabel'),
            helpText: AS.I18n.get('Repository_Facet_AlpineSigningFacet_Keypair_HelpText'),
            allowBlank: false,
            grow: true
          },
          {
            xtype:'as-password',
            name: 'attributes.alpineSigning.passphrase',
            fieldLabel: AS.I18n.get('Repository_Facet_AlpineSigningFacet_Passphrase_FieldLabel'),
            allowBlank: true
          }
        ]
      }
    ];

    me.callParent();
  }

});
