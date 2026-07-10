
/*global Ext, NX*/

/**
 * Upload Component Facet for maven2 format
 *
 * @since 3.29
 */
Ext.define('NX.coreui.view.upload.facet.Maven2UploadFacet', {
  extend: 'NX.coreui.view.upload.facet.DefaultUploadFacet',
  alias: 'widget.nx-coreui-upload-facet-maven2',
  requires: [
    'NX.I18n',
    'Ext.util.Cookies',
    'NX.coreui.view.upload.facet.DefaultUploadFacet'
  ],

  constructor: function(defaultUploadFacet) {
    this.callParent([defaultUploadFacet]);
  },

  /**
   * @override
   */
  addWidget: function() {
    var me = this;
    me.callParent([me.panel]);
    me.panel.down('textfield[name=packaging]').setDisabled(true);
  }
});
