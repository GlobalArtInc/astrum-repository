
/*global Ext, AS*/

/**
 * Ssl Certificate Set feature panel.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.ssl.SslCertificateFeature', {
  extend: 'AS.view.drilldown.Drilldown',
  alias: 'widget.as-coreui-sslcertificate-feature',
  requires: [
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      iconName: 'sslcertificate-default',

      masters: [
        { xtype: 'as-coreui-sslcertificate-list' }
      ],

      tabs: { xtype: 'as-coreui-sslcertificate-details' },

      asActions: [
        {
          xtype: 'button',
          text: AS.I18n.get('Ssl_SslCertificateFeature_Delete_Button'),
          iconCls: 'x-fa fa-trash',
          action: 'delete',
          disabled: true
        }
      ]
    });

    this.callParent();
  }
});
