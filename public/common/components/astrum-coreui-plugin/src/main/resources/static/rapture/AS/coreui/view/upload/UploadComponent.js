
/*global Ext, AS*/

/**
 * Upload Component View
 *
 * @since 3.7
 */
Ext.define('AS.coreui.view.upload.UploadComponent', {
    extend: 'AS.view.AddPanel',
    alias: 'widget.as-coreui-upload-component',
    requires: [
      'AS.I18n',
      'Ext.util.Cookies',
      'AS.coreui.view.upload.facet.DefaultUploadFacet'
    ],
    uses: [
      'AS.coreui.view.upload.facet.Maven2UploadFacet'
    ],
  /**
   * Facet for describing upload page per format, if not specified DefaultUploadFacet will be used.
   * Also facet class should be added in uses section upper.
   * @alias as-coreui-upload-facet-{format}
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
      var formatXtype = 'as-coreui-upload-facet-' + repository.get('format');
      var facetNameByAlias = Ext.ClassManager.getNameByAlias('widget.' + formatXtype);
      var uploadFacetAlias = facetNameByAlias ? formatXtype : 'as-coreui-upload-facet-default';

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
