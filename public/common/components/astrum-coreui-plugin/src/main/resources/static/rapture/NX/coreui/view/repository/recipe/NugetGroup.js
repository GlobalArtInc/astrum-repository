
/*global Ext, NX*/

/**
 * Repository "Settings" form for a NuGet Group repository.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.repository.recipe.NugetGroup', {
  extend: 'NX.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.nx-coreui-repository-nuget-group',
  requires: [
    'NX.coreui.view.repository.facet.StorageFacet',
    'NX.coreui.view.repository.facet.NugetGroupFacet'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'nx-coreui-repository-storage-facet'},
      {xtype: 'nx-coreui-repository-nuget-group-facet', format: 'nuget'}
    ];

    me.callParent();
  }
});
