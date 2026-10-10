
/**
 * Configuration specific to apt proxy repositories.
 *
 * @since 3.17
 */
Ext.define('AS.coreui.view.repository.facet.AptProxyFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.as-aptui-repository-aptproxy-facet',
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
            xtype: 'checkbox',
            name: 'attributes.apt.enforceDistribution',
            fieldLabel: AS.I18n.get('Repository_Facet_AptFacet_EnforceDistribution_FieldLabel'),
            helpText: AS.I18n.get('Repository_Facet_AptFacet_EnforceDistribution_HelpText'),
            value: false,
            listeners: {
              change: function(checkbox, newValue) {
                var form = checkbox.up('form');
                if (form) {
                  var distributionField = form.down('#aptDistribution');
                  if (distributionField) {
                    distributionField.allowBlank = !newValue;
                    distributionField.validate();
                  }
                }
              }
            }
          },
          {
            xtype:'textfield',
            name: 'attributes.apt.distribution',
            itemId: 'aptDistribution',
            fieldLabel: AS.I18n.get('Repository_Facet_AptFacet_Distribution_FieldLabel'),
            helpText: AS.I18n.get('Repository_Facet_AptFacet_Distribution_HelpText'),
            allowBlank: true
          },
          {
            xtype: 'checkbox',
            name: 'attributes.apt.flat',
            fieldLabel: AS.I18n.get('Repository_Facet_AptFacet_Flat_FieldLabel'),
            helpText: AS.I18n.get('Repository_Facet_AptFacet_Flat_HelpText'),
            value: false
          }
        ]
      }
    ];

    me.callParent();
  }

});
