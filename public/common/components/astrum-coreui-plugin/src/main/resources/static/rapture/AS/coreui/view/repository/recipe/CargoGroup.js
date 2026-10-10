
/*global Ext, AS*/

/**
 * Repository "Settings" form for a cargo Group repository.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.repository.recipe.CargoGroup', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-cargo-group',
  requires: [
    'AS.coreui.view.repository.facet.StorageFacet',
    'AS.coreui.view.repository.facet.GroupFacet',
    'AS.coreui.view.repository.facet.CargoRequireAuthenticationFacet'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'as-coreui-repository-cargo-require-authentication-facet'},
      {xtype: 'as-coreui-repository-storage-facet'},
      {xtype: 'as-coreui-repository-group-facet', format: 'cargo'}
    ];

    me.callParent();
  }
});
