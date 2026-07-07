
/*global Ext, NX*/

/**
 * Repository "Settings" form for a cargo Group repository.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.repository.recipe.CargoGroup', {
  extend: 'NX.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.nx-coreui-repository-cargo-group',
  requires: [
    'NX.coreui.view.repository.facet.StorageFacet',
    'NX.coreui.view.repository.facet.GroupFacet',
    'NX.coreui.view.repository.facet.CargoRequireAuthenticationFacet'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'nx-coreui-repository-cargo-require-authentication-facet'},
      {xtype: 'nx-coreui-repository-storage-facet'},
      {xtype: 'nx-coreui-repository-group-facet', format: 'cargo'}
    ];

    me.callParent();
  }
});
