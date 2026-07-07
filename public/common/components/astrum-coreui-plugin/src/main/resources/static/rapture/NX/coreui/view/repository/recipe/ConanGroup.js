
/*global Ext, NX*/

/**
 * Repository "Settings" form for a Conan Group repository
 *
 * @since 3.28
 */
Ext.define('NX.coreui.view.repository.recipe.ConanGroup', {
  extend: 'NX.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.nx-coreui-repository-conan-group',
  requires: [
    'NX.coreui.view.repository.facet.GroupFacet',
    'NX.coreui.view.repository.facet.StorageFacet',
  ],

  /**
   * @override
   */
  initComponent: function () {
    const me = this;

    me.items = [
      {xtype: 'nx-coreui-repository-storage-facet'},
      {xtype: 'nx-coreui-repository-group-facet', format: 'conan', supportsGroupWrite: true }
    ];

    me.callParent();
  }
});
