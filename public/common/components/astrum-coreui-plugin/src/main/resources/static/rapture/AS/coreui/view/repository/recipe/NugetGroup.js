
/*global Ext, AS*/

/**
 * Repository "Settings" form for a NuGet Group repository.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.repository.recipe.NugetGroup', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-nuget-group',
  requires: [
    'AS.coreui.view.repository.facet.StorageFacet',
    'AS.coreui.view.repository.facet.NugetGroupFacet'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'as-coreui-repository-storage-facet'},
      {xtype: 'as-coreui-repository-nuget-group-facet', format: 'nuget'}
    ];

    me.callParent();
  }
});
