
/*global Ext, NX*/

/**
 * Asset info panel.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.component.AssetInfo', {
  extend: 'NX.view.info.Panel',
  alias: 'widget.nx-coreui-component-assetinfo',
  requires: [
    'NX.I18n',
    'NX.coreui.util.RepositoryUrls'
  ],
  
  mixins: {
    componentUtils: 'NX.coreui.mixin.ComponentUtils'
  },

  /**
   * model to display
   */
  assetModel: null,

  /**
   * @public
   * @param {Object} assetModel the asset to display
   */
  setAssetModel: function (assetModel) {
    var me = this,
        info = {};
    me.assetModel = assetModel;

    // display common data
    var contentType = assetModel.get('contentType');
    var size = assetModel.get('size');
    info[NX.I18n.get('Assets_Info_Path')] = NX.view.info.Entry.rawHtml(
        NX.coreui.util.RepositoryUrls.asRepositoryLink(assetModel, assetModel.get('format')));
    info[NX.I18n.get('Assets_Info_ContentType')] = contentType;
    info[NX.I18n.get('Assets_Info_FileSize')] = Ext.util.Format.fileSize(size);
    info[NX.I18n.get('Assets_Info_Blob_Created')] = assetModel.get('blobCreated');
    info[NX.I18n.get('Assets_Info_Blob_Updated')] = assetModel.get('blobUpdated');

    if (assetModel.get('downloadCount')) {
      info[NX.I18n.get('Assets_Info_Downloaded_Count')] = assetModel.get('downloadCount') + ' ' +
          NX.I18n.get('Assets_Info_Downloaded_Unit');
    }

    info[NX.I18n.get('Assets_Info_Last_Downloaded')] =
        me.mixins.componentUtils.getLastDownloadDateForDisplay(assetModel);
    info[NX.I18n.get('Assets_Info_Locally_Cached')] = contentType !== 'unknown' && size > 0;
    info[NX.I18n.get('Assets_Info_BlobRef')] = assetModel.get('blobRef');
    info[NX.I18n.get('Assets_Info_ContainingRepositoryName')] = assetModel.get('containingRepositoryName');

    if (assetModel.get('createdBy')) {
      info[NX.I18n.get('Assets_Info_UploadedBy')] = assetModel.get('createdBy');
    }
    if (assetModel.get('createdByIp')) {
      info[NX.I18n.get('Assets_Info_UploadedIp')] = assetModel.get('createdByIp');
    }

    me.showInfo(info);
  }

});
