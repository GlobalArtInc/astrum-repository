
/*global Ext, NX*/

/**
 * Repository "Settings" form for a Conda Group repository
 */
Ext.define('NX.coreui.view.repository.recipe.CondaGroup', {
  extend: 'NX.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.nx-coreui-repository-conda-group',
  requires: [
    'NX.coreui.view.repository.facet.GroupFacet',
    'NX.coreui.view.repository.facet.StorageFacet'
  ],

  /**
   * @override
   */
  initComponent: function () {
    const me = this;

    me.items = [
      {xtype: 'nx-coreui-repository-storage-facet'},
      {xtype: 'nx-coreui-repository-group-facet', format: 'conda'}
    ];

    me.callParent();
  }
});
