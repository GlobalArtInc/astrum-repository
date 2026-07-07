
/**
 * Repository "Settings" form for an OCI Group repository.
 *
 * @since 3.91
 */
Ext.define('NX.coreui.view.repository.recipe.OciGroup', {
  extend: 'NX.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.nx-coreui-repository-oci-group',
  requires: [
    'NX.coreui.view.repository.facet.StorageFacet',
    'NX.oci.view.repository.facet.OciConnectorFacet',
    'NX.coreui.view.repository.facet.GroupFacet'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'nx-oci-repository-connector-facet'},
      {xtype: 'nx-coreui-repository-storage-facet'},
      {
        xtype: 'nx-coreui-repository-group-facet',
        format: 'oci'
      }
    ];

    me.callParent();
  }
});
