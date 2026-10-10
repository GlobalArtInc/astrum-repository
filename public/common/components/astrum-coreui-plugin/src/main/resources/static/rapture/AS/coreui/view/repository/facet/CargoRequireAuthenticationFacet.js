
/*global Ext, AS*/

/**
 * Configuration for availability of Docker V1 api.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.repository.facet.CargoRequireAuthenticationFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.as-coreui-repository-cargo-require-authentication-facet',
  requires: [
    'AS.I18n'
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
        cls: 'as-form-section',
        title: AS.I18n.get('Repository_Facet_CargoRequire_Authentication_Title'),
        items: [
          {
            xtype: 'label',
            text: AS.I18n.get('Repository_Facet_CargoRequire_Authentication_Enabled'),
            style: {
              fontWeight: 'bold',
              display: 'block',
            }
          },
          {
            xtype: 'label',
            text: AS.I18n.get('Repository_Facet_CargoRequire_Authentication_HelpText'),
            style: {
              fontStyle: 'italic'
            }
          },
          {
            xtype: 'checkbox',
            name: 'attributes.cargo.requireAuthentication',
            itemId: 'authrequired',
            helpText: AS.I18n.get('Repository_Facet_CargoRequire_Authentication_Help'),
            value: false
          }
        ]
      }
    ];

    me.callParent();
  }

});
