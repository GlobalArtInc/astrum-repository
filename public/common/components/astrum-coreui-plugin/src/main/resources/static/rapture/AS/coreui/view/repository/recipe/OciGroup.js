
/**
 * Repository "Settings" form for an OCI Group repository.
 *
 * @since 3.91
 */
Ext.define('AS.coreui.view.repository.recipe.OciGroup', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-oci-group',
  requires: [
    'AS.coreui.view.repository.facet.StorageFacet',
    'AS.oci.view.repository.facet.OciConnectorFacet',
    'AS.coreui.view.repository.facet.GroupFacet'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'as-oci-repository-connector-facet'},
      {xtype: 'as-coreui-repository-storage-facet'},
      {
        xtype: 'as-coreui-repository-group-facet',
        format: 'oci'
      }
    ];

    me.callParent();
  }
});
