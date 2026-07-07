
/*global Ext, NX*/

/**
 * Repository "Settings" form for a Raw Group repository.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.repository.recipe.RawGroup', {
  extend: 'NX.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.nx-coreui-repository-raw-group',
  requires: [
    'NX.coreui.view.repository.facet.RawFacet',
    'NX.coreui.view.repository.facet.StorageFacet',
    'NX.coreui.view.repository.facet.GroupFacet'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'nx-coreui-repository-raw-facet'},
      {xtype: 'nx-coreui-repository-storage-facet'},
      {xtype: 'nx-coreui-repository-group-facet', format: 'raw' }
    ];

    me.callParent();
  }
});
