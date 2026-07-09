
/*global Ext, NX*/

/**
 * Ssl Certificate grid.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.ssl.SslCertificateList', {
  extend: 'NX.view.drilldown.Master',
  alias: 'widget.nx-coreui-sslcertificate-list',
  requires: [
    'NX.Icons',
    'NX.I18n'
  ],

  stateful: true,
  stateId: 'nx-coreui-sslcertificate-list',

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.store = 'SslCertificate';

    me.columns = [
      {
        xtype: 'nx-iconcolumn',
        width: 36,
        iconVariant: 'x16',
        iconName: function () {
          return 'sslcertificate-default';
        }
      },
      {
        header: NX.I18n.get('Ssl_SslCertificateList_Name_Header'),
        dataIndex: 'subjectCommonName',
        stateId: 'subjectCommonName',
        flex: 1,
        renderer: Ext.htmlEncode
      },
      {
        header: NX.I18n.get('Ssl_SslCertificateList_IssuedTo_Header'),
        dataIndex: 'subjectOrganization',
        stateId: 'subjectOrganization',
        flex: 1,
        renderer: Ext.htmlEncode
      },
      {
        header: NX.I18n.get('Ssl_SslCertificateList_IssuedBy_Header'),
        dataIndex: 'issuerOrganization',
        stateId: 'issuerOrganization',
        flex: 1,
        renderer: Ext.htmlEncode
      },
      {
        header: NX.I18n.get('Ssl_SslCertificateList_Fingerprint_Header'),
        dataIndex: 'fingerprint',
        stateId: 'fingerprint',
        flex: 1,
        renderer: Ext.htmlEncode
      }
    ];

    me.viewConfig = {
      emptyText: NX.I18n.get('Ssl_SslCertificateList_EmptyText'),
      deferEmptyText: false
    };

    me.plugins = [
      { ptype: 'gridfilterbox', emptyText: NX.I18n.get('Ssl_SslCertificateList_Filter_EmptyText') }
    ];

    me.dockedItems = [{
      xtype: 'nx-actions',
      items: [
        {
          xtype: 'button',
          text: NX.I18n.get('Ssl_SslCertificateList_New_Button'),
          iconCls: 'x-fa fa-plus-circle',
          action: 'new',
          disabled: true,
          menu: [
            {
              text: NX.I18n.get('Ssl_SslCertificateList_Load_Button'),
              action: 'newfromserver',
              iconCls: NX.Icons.cls('sslcertificate-add-by-server', 'x16')
            },
            {
              text: NX.I18n.get('Ssl_SslCertificateList_Paste_Button'),
              action: 'newfrompem',
              iconCls: NX.Icons.cls('sslcertificate-add-by-pem', 'x16')
            }
          ]
        }
      ]
    }];

    me.callParent();
  }
});
