
/*global Ext, AS*/

/**
 * Repository "Settings" form for a Conan Group repository
 *
 * @since 3.28
 */
Ext.define('AS.coreui.view.repository.recipe.ConanGroup', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-conan-group',
  requires: [
    'AS.coreui.view.repository.facet.GroupFacet',
    'AS.coreui.view.repository.facet.StorageFacet',
  ],

  /**
   * @override
   */
  initComponent: function () {
    const me = this;

    me.items = [
      {xtype: 'as-coreui-repository-storage-facet'},
      {xtype: 'as-coreui-repository-group-facet', format: 'conan', supportsGroupWrite: true }
    ];

    me.callParent();
  }
});
