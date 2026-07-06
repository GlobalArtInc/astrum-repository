
/*global Ext, NX*/

/**
 * Repository grid.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.repository.RepositoryListTemplate', {
  extend: 'NX.view.drilldown.Master',
  alias: 'widget.nx-coreui-repository-list-template',
  requires: [
    'NX.I18n'
  ],

  hideSize: true,

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.columns = [
      {
        xtype: 'nx-iconcolumn',
        width: 36,
        iconVariant: 'x16',
        iconNamePrefix: 'repository-',
        dataIndex: 'type'
      },
      {
        text: NX.I18n.get('Repository_RepositoryList_Name_Header'),
        dataIndex: 'name',
        stateId: 'name',
        flex: 1,
        renderer: Ext.htmlEncode
      },
      {
        text: NX.I18n.get('Repository_RepositoryList_Size_Header'),
        dataIndex: 'size',
        stateId: 'size',
        renderer: Ext.util.Format.fileSize,
        hidden: me.hideSize,
        hideable: !me.hideSize
      },
      {
        text: NX.I18n.get('Repository_RepositoryList_Type_Header'),
        dataIndex: 'type',
        stateId: 'type',
        renderer: Ext.htmlEncode
      },
      {
        text: NX.I18n.get('Repository_RepositoryList_Format_Header'),
        dataIndex: 'format',
        stateId: 'format',
        renderer: Ext.htmlEncode
      },
      {
        text: NX.I18n.get('Repository_RepositoryList_BlobStore_Header'),
        dataIndex: 'blobStoreName',
        stateId: 'blobStoreName',
        flex: 1,
        renderer: Ext.htmlEncode,
        hidden: me.hideBlobStore,
      },
      {
        header: NX.I18n.get('Repository_RepositoryList_Status_Header'), dataIndex: 'status', stateId: 'status', flex: 1,
        xtype: 'templatecolumn',
        tpl: new Ext.XTemplate(
          '<tpl if="status.online">',
          'Online',
          '<tpl else>',
          'Offline',
          '</tpl>',
          '<tpl if="status.description">',
          ' - {status.description:htmlEncode}',
          '</tpl>',
          '<tpl if="status.reason">',
          '<br/><i>{status.reason:htmlEncode}</i>',
          '</tpl>')
      },
      {
        xtype: 'nx-copylinkcolumn',
        header: NX.I18n.get('Repository_RepositoryList_URL_Header'),
        dataIndex: 'url'
      }
    ];

    me.viewConfig = {
      emptyText: NX.I18n.get('Repository_RepositoryList_EmptyText'),
      deferEmptyText: false,
      markDirty: false
    };

    me.plugins = [
      {
        ptype: 'gridfilterbox',
        emptyText: NX.I18n.get('Repository_RepositoryList_Filter_EmptyText')
      }
    ];

    me.callParent();
  }

});
