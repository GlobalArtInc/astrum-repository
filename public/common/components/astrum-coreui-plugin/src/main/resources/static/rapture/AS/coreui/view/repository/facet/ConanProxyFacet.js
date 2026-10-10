
/*global Ext, AS*/

/**
 * Configuration for Conan proxy repositories.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.repository.facet.ConanProxyFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.as-coreui-repository-conanproxy-facet',
  requires: [
    'AS.I18n',
    'AS.State',
    'AS.Icons'
  ],

  /**
   * @override
   */
  initComponent: function() {
    const me = this;

    me.items = [
      {
        xtype: 'fieldset',
        cls: 'as-form-section',
        title: AS.I18n.get('Repository_Facet_ConanProxyFacet_Title'),
        itemId: 'conan-radio-group-section',
        hidden: !AS.State.isConanV2Enabled(),
        items: [
          {
            xtype: 'label',
            cls: 'description',
            html: AS.I18n.get('Repository_Facet_ConanProxyFacet_HelpLink'),
            style: {
              fontSize: '12px',
            }
          },
          {
            xtype: 'radiogroup',
            name: 'conanVersion',
            fieldLabel: AS.I18n.get('Repository_Facet_ConanProxyFacet_ProtocolVersion'),
            columns: 1,
            vertical: true,
            items: [
              {
                boxLabel: AS.I18n.get('Repository_Facet_ConanProxyFacet_V1'),
                name: 'attributes.conan.conanVersion',
                inputValue: 'V1'
              },
              {
                boxLabel: AS.I18n.get('Repository_Facet_ConanProxyFacet_V2'),
                name: 'attributes.conan.conanVersion',
                inputValue: 'V2'
              }
            ],
            listeners: {
              afterrender: function(radioGroupForm) {
                if (Ext.Object.isEmpty(radioGroupForm.getValue())) {
                  const defaultValue = {
                    'attributes.conan.conanVersion': 'V2'
                  };
                  radioGroupForm.setValue(defaultValue);
                } else {
                  radioGroupForm.up('fieldset').hide();
                }
              },
              change: function(radioGroupForm, newValue) {
                const form = radioGroupForm.up('form');
                const remoteUrl = form && form.down('#remoteUrl');
                if (!remoteUrl) {
                  return;
                }
                const version = newValue['attributes.conan.conanVersion'];
                const key = version === 'V1'
                    ? 'Repository_Facet_ProxyFacet_Conan_V1_Remote_HelpText'
                    : 'Repository_Facet_ProxyFacet_Conan_V2_Remote_HelpText';
                const text = AS.I18n.get(key);
                remoteUrl.setHelpText(text);
                if (remoteUrl.rendered && remoteUrl.el) {
                  const helpEl = remoteUrl.el.down('.as-boxlabel');
                  if (helpEl) {
                    helpEl.dom.innerHTML = text;
                  }
                }
              }
            }
          },
          {
            xtype: 'panel',
            itemId: 'info',
            ui: 'as-drilldown-message',
            cls: 'as-drilldown-info',
            iconCls: AS.Icons.cls('drilldown-info', 'x16'),
            title: AS.I18n.get('Repository_Facet_ConanProxyFacet_HelpText')
          }
        ]
      },
      {
        xtype: 'fieldset',
        cls: 'as-form-section',
        title: AS.I18n.get('Repository_Facet_ConanProxyFacet_Title'),
        hidden: !AS.State.isConanV2Enabled(),
        items: [
          {
            xtype: 'textfield',
            name: 'version',
            itemId: 'version',
            labelAlign: 'left',
            fieldLabel: AS.I18n.get('Repository_Facet_ConanProxyFacet_Version'),
            readOnly: true,
            listeners: {
              afterrender: function() {
                const me = this;
                const form = me.up('form');
                const radioGroupSection = form.down('#conan-radio-group-section');

                if (radioGroupSection.isHidden()) {
                  const conanVersion = Ext.Object.getValues(radioGroupSection.down('radiogroup').getValue())[0];
                  me.setValue(conanVersion);
                  me.resetOriginalValue();
                } else {
                  me.up('fieldset').hide();
                }
              }
            }
          }
        ]
      }
    ];

    me.callParent();
  }
});
