
/*global Ext, NX*/

/**
 * Repository "Settings" form for a PyPI Group repository.
 *
 * @since 3.1
 */
Ext.define('NX.coreui.view.repository.recipe.PyPiGroup', {
  extend: 'NX.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.nx-coreui-repository-pypi-group',
  requires: [
    'NX.coreui.view.repository.facet.StorageFacet',
    'NX.coreui.view.repository.facet.GroupFacet'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'nx-coreui-repository-storage-facet'},
      {xtype: 'nx-coreui-repository-group-facet', format: 'pypi'}
    ];

    me.callParent();
  }
});
