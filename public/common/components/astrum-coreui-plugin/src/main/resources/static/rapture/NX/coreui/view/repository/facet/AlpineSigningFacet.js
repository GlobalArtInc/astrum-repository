
/**
 * Configuration for Alpine repository signing.
 *
 * @since 3.95
 */
Ext.define('NX.coreui.view.repository.facet.AlpineSigningFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.nx-coreui-repository-alpinesigning-facet',
  requires: [
    'NX.I18n'
  ],
  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {
        xtype: 'fieldset',
        cls: 'nx-form-section',
        title: NX.I18n.get('Repository_Facet_AlpineSigningFacet_Title'),
        items: [
          {
            xtype:'textareafield',
            name: 'attributes.alpineSigning.keypair',
            fieldLabel: NX.I18n.get('Repository_Facet_AlpineSigningFacet_Keypair_FieldLabel'),
            helpText: NX.I18n.get('Repository_Facet_AlpineSigningFacet_Keypair_HelpText'),
            allowBlank: false,
            grow: true
          },
          {
            xtype:'nx-password',
            name: 'attributes.alpineSigning.passphrase',
            fieldLabel: NX.I18n.get('Repository_Facet_AlpineSigningFacet_Passphrase_FieldLabel'),
            allowBlank: true
          }
        ]
      }
    ];

    me.callParent();
  }

});
