
/*global Ext, NX*/

/**
 * Configuration for Conan proxy repositories.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.repository.facet.ConanProxyFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.nx-coreui-repository-conanproxy-facet',
  requires: [
    'NX.I18n',
    'NX.State',
    'NX.Icons'
  ],

  /**
   * @override
   */
  initComponent: function() {
    const me = this;

    me.items = [
      {
        xtype: 'fieldset',
        cls: 'nx-form-section',
        title: NX.I18n.get('Repository_Facet_ConanProxyFacet_Title'),
        itemId: 'conan-radio-group-section',
        hidden: !NX.State.isConanV2Enabled(),
        items: [
          {
            xtype: 'label',
            cls: 'description',
            html: NX.I18n.get('Repository_Facet_ConanProxyFacet_HelpLink'),
            style: {
              fontSize: '12px',
            }
          },
          {
            xtype: 'radiogroup',
            name: 'conanVersion',
            fieldLabel: NX.I18n.get('Repository_Facet_ConanProxyFacet_ProtocolVersion'),
            columns: 1,
            vertical: true,
            items: [
              {
                boxLabel: NX.I18n.get('Repository_Facet_ConanProxyFacet_V1'),
                name: 'attributes.conan.conanVersion',
                inputValue: 'V1'
              },
              {
                boxLabel: NX.I18n.get('Repository_Facet_ConanProxyFacet_V2'),
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
                const text = NX.I18n.get(key);
                remoteUrl.setHelpText(text);
                if (remoteUrl.rendered && remoteUrl.el) {
                  const helpEl = remoteUrl.el.down('.nx-boxlabel');
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
            ui: 'nx-drilldown-message',
            cls: 'nx-drilldown-info',
            iconCls: NX.Icons.cls('drilldown-info', 'x16'),
            title: NX.I18n.get('Repository_Facet_ConanProxyFacet_HelpText')
          }
        ]
      },
      {
        xtype: 'fieldset',
        cls: 'nx-form-section',
        title: NX.I18n.get('Repository_Facet_ConanProxyFacet_Title'),
        hidden: !NX.State.isConanV2Enabled(),
        items: [
          {
            xtype: 'textfield',
            name: 'version',
            itemId: 'version',
            labelAlign: 'left',
            fieldLabel: NX.I18n.get('Repository_Facet_ConanProxyFacet_Version'),
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
