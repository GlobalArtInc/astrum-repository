
/*global Ext, AS*/

/**
 * Repository "Settings" form for a Conda Group repository
 */
Ext.define('AS.coreui.view.repository.recipe.CondaGroup', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-conda-group',
  requires: [
    'AS.coreui.view.repository.facet.GroupFacet',
    'AS.coreui.view.repository.facet.StorageFacet'
  ],

  /**
   * @override
   */
  initComponent: function () {
    const me = this;

    me.items = [
      {xtype: 'as-coreui-repository-storage-facet'},
      {xtype: 'as-coreui-repository-group-facet', format: 'conda'}
    ];

    me.callParent();
  }
});
