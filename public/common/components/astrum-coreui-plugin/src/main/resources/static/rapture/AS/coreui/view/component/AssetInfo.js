
/*global Ext, AS*/

/**
 * Asset info panel.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.component.AssetInfo', {
  extend: 'AS.view.info.Panel',
  alias: 'widget.as-coreui-component-assetinfo',
  requires: [
    'AS.I18n',
    'AS.coreui.util.RepositoryUrls'
  ],
  
  mixins: {
    componentUtils: 'AS.coreui.mixin.ComponentUtils'
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
    info[AS.I18n.get('Assets_Info_Path')] = AS.view.info.Entry.rawHtml(
        AS.coreui.util.RepositoryUrls.asRepositoryLink(assetModel, assetModel.get('format')));
    info[AS.I18n.get('Assets_Info_ContentType')] = contentType;
    info[AS.I18n.get('Assets_Info_FileSize')] = Ext.util.Format.fileSize(size);
    info[AS.I18n.get('Assets_Info_Blob_Created')] = assetModel.get('blobCreated');
    info[AS.I18n.get('Assets_Info_Blob_Updated')] = assetModel.get('blobUpdated');

    if (assetModel.get('downloadCount') !== null) {
      info[AS.I18n.get('Assets_Info_Downloaded_Count')] =
          Ext.util.Format.number(assetModel.get('downloadCount'), '0,000');
    }

    info[AS.I18n.get('Assets_Info_Last_Downloaded')] =
        me.mixins.componentUtils.getLastDownloadDateForDisplay(assetModel);
    info[AS.I18n.get('Assets_Info_Locally_Cached')] = contentType !== 'unknown' && size > 0;
    info[AS.I18n.get('Assets_Info_BlobRef')] = assetModel.get('blobRef');
    info[AS.I18n.get('Assets_Info_ContainingRepositoryName')] = assetModel.get('containingRepositoryName');

    if (assetModel.get('createdBy')) {
      info[AS.I18n.get('Assets_Info_UploadedBy')] = assetModel.get('createdBy');
    }
    if (assetModel.get('createdByIp')) {
      info[AS.I18n.get('Assets_Info_UploadedIp')] = assetModel.get('createdByIp');
    }

    me.showInfo(info);
  }

});
