
/*global Ext, AS*/

/**
 * Repository grid.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.repository.RepositoryListTemplate', {
  extend: 'AS.view.drilldown.Master',
  alias: 'widget.as-coreui-repository-list-template',
  requires: [
    'AS.I18n'
  ],

  hideSize: true,

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.columns = [
      {
        xtype: 'as-iconcolumn',
        width: 36,
        iconVariant: 'x16',
        iconNamePrefix: 'repository-',
        dataIndex: 'type'
      },
      {
        text: AS.I18n.get('Repository_RepositoryList_Name_Header'),
        dataIndex: 'name',
        stateId: 'name',
        flex: 1,
        renderer: Ext.htmlEncode
      },
      {
        text: AS.I18n.get('Repository_RepositoryList_Size_Header'),
        dataIndex: 'size',
        stateId: 'size',
        renderer: Ext.util.Format.fileSize,
        hidden: me.hideSize,
        hideable: !me.hideSize
      },
      {
        text: AS.I18n.get('Repository_RepositoryList_Type_Header'),
        dataIndex: 'type',
        stateId: 'type',
        renderer: Ext.htmlEncode
      },
      {
        text: AS.I18n.get('Repository_RepositoryList_Format_Header'),
        dataIndex: 'format',
        stateId: 'format',
        renderer: Ext.htmlEncode
      },
      {
        text: AS.I18n.get('Repository_RepositoryList_BlobStore_Header'),
        dataIndex: 'blobStoreName',
        stateId: 'blobStoreName',
        flex: 1,
        renderer: Ext.htmlEncode,
        hidden: me.hideBlobStore,
      },
      {
        header: AS.I18n.get('Repository_RepositoryList_Status_Header'), dataIndex: 'status', stateId: 'status', flex: 1,
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
        xtype: 'as-copylinkcolumn',
        header: AS.I18n.get('Repository_RepositoryList_URL_Header'),
        dataIndex: 'url'
      }
    ];

    me.viewConfig = {
      emptyText: AS.I18n.get('Repository_RepositoryList_EmptyText'),
      deferEmptyText: false,
      markDirty: false
    };

    me.plugins = [
      {
        ptype: 'gridfilterbox',
        emptyText: AS.I18n.get('Repository_RepositoryList_Filter_EmptyText')
      }
    ];

    me.callParent();
  }

});
