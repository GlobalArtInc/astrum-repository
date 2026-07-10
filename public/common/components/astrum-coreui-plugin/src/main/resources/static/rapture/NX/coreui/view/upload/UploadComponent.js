
/*global Ext, NX*/

/**
 * Upload Component View
 *
 * @since 3.7
 */
Ext.define('NX.coreui.view.upload.UploadComponent', {
    extend: 'NX.view.AddPanel',
    alias: 'widget.nx-coreui-upload-component',
    requires: [
      'NX.I18n',
      'Ext.util.Cookies',
      'NX.coreui.view.upload.facet.DefaultUploadFacet'
    ],
    uses: [
      'NX.coreui.view.upload.facet.Maven2UploadFacet'
    ],
  /**
   * Facet for describing upload page per format, if not specified DefaultUploadFacet will be used.
   * Also facet class should be added in uses section upper.
   * @alias nx-coreui-upload-facet-{format}
   * @type {DefaultUploadFacet}
   */
  uploadFacet: undefined,

    /**
     * @override
     */
    initComponent: function() {
      var me = this;
      me.store = 'UploadDefinition';

      me.callParent();
    },

    loadRecord: function(uploadDefinition, repository) {
      var me = this;
      var formatXtype = 'nx-coreui-upload-facet-' + repository.get('format');
      var facetNameByAlias = Ext.ClassManager.getNameByAlias('widget.' + formatXtype);
      var uploadFacetAlias = facetNameByAlias ? formatXtype : 'nx-coreui-upload-facet-default';

      me.uploadFacet = Ext.create({
        xtype: uploadFacetAlias,
        uploadDefinition: uploadDefinition,
        repository: repository,
        panel: me
      });
      me.uploadFacet.addWidget();
    },

    addAssetRow: function() {
      var me = this;
      return me.uploadFacet.addAssetRow();
    }
  });
