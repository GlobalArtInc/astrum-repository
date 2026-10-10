
/*global Ext, AS*/

/**
 * Ssl use Astrum Repository Truststore checkbox.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.ssl.SslUseTrustStore', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.as-coreui-sslusetruststore',
  requires: [
    'AS.I18n'
  ],

  initComponent: function () {
    var me = this;

    if (!me.fieldLabel && !me.boxLabel) {
      me.fieldLabel = AS.I18n.get('Ssl_SslUseTrustStore_BoxLabel');
    }
    if (me.fieldLabel === true) {
      me.fieldLabel = AS.I18n.get('Ssl_SslUseTrustStore_BoxLabel');
    }
    if (me.boxLabel === true) {
      me.boxLabel = AS.I18n.get('Ssl_SslUseTrustStore_BoxLabel');
    }

    me.items = {
      xtype: 'panel',
      layout: 'hbox',
      items: [
        {
          xtype: 'checkbox',
          name: me.name,
          value: me.value,
          inputValue: true,
          boxLabel: me.boxLabel,
          helpText: AS.I18n.get('Ssl_SslUseTrustStore_Certificate_HelpText')
        },
        {
          xtype: 'button',
          text: AS.I18n.get('Ssl_SslUseTrustStore_Certificate_Button'),
          ui: 'as-plain',
          action: 'showcertificate',
          iconCls: 'x-fa fa-certificate',
          margin: '0 0 0 5'
        }
      ]
    };

    me.callParent();
  }
});
