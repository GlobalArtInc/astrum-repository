
/**
 * Configuration specific to apt proxy repositories.
 *
 * @since 3.17
 */
Ext.define('NX.coreui.view.repository.facet.AptProxyFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.nx-aptui-repository-aptproxy-facet',
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
            xtype: 'checkbox',
            name: 'attributes.apt.enforceDistribution',
            fieldLabel: NX.I18n.get('Repository_Facet_AptFacet_EnforceDistribution_FieldLabel'),
            helpText: NX.I18n.get('Repository_Facet_AptFacet_EnforceDistribution_HelpText'),
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
            fieldLabel: NX.I18n.get('Repository_Facet_AptFacet_Distribution_FieldLabel'),
            helpText: NX.I18n.get('Repository_Facet_AptFacet_Distribution_HelpText'),
            allowBlank: true
          },
          {
            xtype: 'checkbox',
            name: 'attributes.apt.flat',
            fieldLabel: NX.I18n.get('Repository_Facet_AptFacet_Flat_FieldLabel'),
            helpText: NX.I18n.get('Repository_Facet_AptFacet_Flat_HelpText'),
            value: false
          }
        ]
      }
    ];

    me.callParent();
  }

});
