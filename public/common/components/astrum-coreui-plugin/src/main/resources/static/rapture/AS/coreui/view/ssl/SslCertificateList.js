
/*global Ext, AS*/

/**
 * Ssl Certificate grid.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.ssl.SslCertificateList', {
  extend: 'AS.view.drilldown.Master',
  alias: 'widget.as-coreui-sslcertificate-list',
  requires: [
    'AS.Icons',
    'AS.I18n'
  ],

  stateful: true,
  stateId: 'as-coreui-sslcertificate-list',

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.store = 'SslCertificate';

    me.columns = [
      {
        xtype: 'as-iconcolumn',
        width: 36,
        iconVariant: 'x16',
        iconName: function () {
          return 'sslcertificate-default';
        }
      },
      {
        header: AS.I18n.get('Ssl_SslCertificateList_Name_Header'),
        dataIndex: 'subjectCommonName',
        stateId: 'subjectCommonName',
        flex: 1,
        renderer: Ext.htmlEncode
      },
      {
        header: AS.I18n.get('Ssl_SslCertificateList_IssuedTo_Header'),
        dataIndex: 'subjectOrganization',
        stateId: 'subjectOrganization',
        flex: 1,
        renderer: Ext.htmlEncode
      },
      {
        header: AS.I18n.get('Ssl_SslCertificateList_IssuedBy_Header'),
        dataIndex: 'issuerOrganization',
        stateId: 'issuerOrganization',
        flex: 1,
        renderer: Ext.htmlEncode
      },
      {
        header: AS.I18n.get('Ssl_SslCertificateList_Fingerprint_Header'),
        dataIndex: 'fingerprint',
        stateId: 'fingerprint',
        flex: 1,
        renderer: Ext.htmlEncode
      }
    ];

    me.viewConfig = {
      emptyText: AS.I18n.get('Ssl_SslCertificateList_EmptyText'),
      deferEmptyText: false
    };

    me.plugins = [
      { ptype: 'gridfilterbox', emptyText: AS.I18n.get('Ssl_SslCertificateList_Filter_EmptyText') }
    ];

    me.dockedItems = [{
      xtype: 'as-actions',
      items: [
        {
          xtype: 'button',
          text: AS.I18n.get('Ssl_SslCertificateList_New_Button'),
          iconCls: 'x-fa fa-plus-circle',
          action: 'new',
          disabled: true,
          menu: [
            {
              text: AS.I18n.get('Ssl_SslCertificateList_Load_Button'),
              action: 'newfromserver',
              iconCls: AS.Icons.cls('sslcertificate-add-by-server', 'x16')
            },
            {
              text: AS.I18n.get('Ssl_SslCertificateList_Paste_Button'),
              action: 'newfrompem',
              iconCls: AS.Icons.cls('sslcertificate-add-by-pem', 'x16')
            }
          ]
        }
      ]
    }];

    me.callParent();
  }
});
