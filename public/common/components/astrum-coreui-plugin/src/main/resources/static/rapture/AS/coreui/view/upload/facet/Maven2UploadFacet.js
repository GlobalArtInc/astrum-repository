
/*global Ext, AS*/

/**
 * Upload Component Facet for maven2 format
 *
 * @since 3.29
 */
Ext.define('AS.coreui.view.upload.facet.Maven2UploadFacet', {
  extend: 'AS.coreui.view.upload.facet.DefaultUploadFacet',
  alias: 'widget.as-coreui-upload-facet-maven2',
  requires: [
    'AS.I18n',
    'Ext.util.Cookies',
    'AS.coreui.view.upload.facet.DefaultUploadFacet'
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
