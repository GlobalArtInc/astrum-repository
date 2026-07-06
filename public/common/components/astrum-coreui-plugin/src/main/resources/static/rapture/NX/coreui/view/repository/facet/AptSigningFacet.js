
/**
 * Configuration specific to apt repositories.
 *
 * @since 3.17
 */
Ext.define('NX.coreui.view.repository.facet.AptSigningFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.nx-aptui-repository-aptsigning-facet',
  requires: [
    'NX.I18n'
  ],
  /**
   * @cfg {Boolean} allowBlank
   * Whether the keypair field allows blank values. Defaults to false (required for hosted repos).
   */
  allowBlank: false,

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {
        xtype: 'fieldset',
        cls: 'nx-form-section',
        title: NX.I18n.get('Repository_Facet_AptFacet_Title'),
        items: [
          {
            xtype:'textareafield',
            name: 'attributes.aptSigning.keypair',
            fieldLabel: NX.I18n.get('Repository_Facet_AptSigningFacet_Keypair_FieldLabel'),
            helpText: NX.I18n.get('Repository_Facet_AptSigningFacet_Keypair_HelpText'),
            allowBlank: me.allowBlank,
            grow: true
          },
          {
            xtype:'nx-password',
            name: 'attributes.aptSigning.passphrase',
            fieldLabel: NX.I18n.get('Repository_Facet_AptSigningFacet_Passphrase_FieldLabel'),
            allowBlank: true
          }
        ]
      }
    ];

    me.callParent();
  }

});
