
/**
 * Configuration specific to apt repositories.
 *
 * @since 3.17
 */
Ext.define('AS.coreui.view.repository.facet.AptHostedFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.as-aptui-repository-apthosted-facet',
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
        title: AS.I18n.get('Repository_Facet_AptFacet_Title'),
        items: [
          {
            xtype:'textfield',
            name: 'attributes.apt.distribution',
            fieldLabel: AS.I18n.get('Repository_Facet_AptFacet_Distribution_FieldLabel'),
            helpText: AS.I18n.get('Repository_Facet_AptFacet_Distribution_HelpText'),
            allowBlank: false
          }
        ]
      }
    ];

    me.callParent();
  }

});
