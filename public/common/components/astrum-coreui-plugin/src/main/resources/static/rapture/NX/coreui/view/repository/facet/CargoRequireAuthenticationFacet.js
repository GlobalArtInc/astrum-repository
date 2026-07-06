
/*global Ext, NX*/

/**
 * Configuration for availability of Docker V1 api.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.repository.facet.CargoRequireAuthenticationFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.nx-coreui-repository-cargo-require-authentication-facet',
  requires: [
    'NX.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    const me = this;

    me.items = [
      {
        xtype: 'fieldset',
        itemId: 'cargoAuth',
        cls: 'nx-form-section',
        title: NX.I18n.get('Repository_Facet_CargoRequire_Authentication_Title'),
        items: [
          {
            xtype: 'label',
            text: NX.I18n.get('Repository_Facet_CargoRequire_Authentication_Enabled'),
            style: {
              fontWeight: 'bold',
              display: 'block',
            }
          },
          {
            xtype: 'label',
            text: NX.I18n.get('Repository_Facet_CargoRequire_Authentication_HelpText'),
            style: {
              fontStyle: 'italic'
            }
          },
          {
            xtype: 'checkbox',
            name: 'attributes.cargo.requireAuthentication',
            itemId: 'authrequired',
            helpText: NX.I18n.get('Repository_Facet_CargoRequire_Authentication_Help'),
            value: false
          }
        ]
      }
    ];

    me.callParent();
  }

});
