
/**
 * Configuration specific to apt repositories.
 *
 * @since 3.17
 */
Ext.define('NX.coreui.view.repository.facet.AptHostedFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.nx-aptui-repository-apthosted-facet',
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
        title: NX.I18n.get('Repository_Facet_AptFacet_Title'),
        items: [
          {
            xtype:'textfield',
            name: 'attributes.apt.distribution',
            fieldLabel: NX.I18n.get('Repository_Facet_AptFacet_Distribution_FieldLabel'),
            helpText: NX.I18n.get('Repository_Facet_AptFacet_Distribution_HelpText'),
            allowBlank: false
          }
        ]
      }
    ];

    me.callParent();
  }

});
