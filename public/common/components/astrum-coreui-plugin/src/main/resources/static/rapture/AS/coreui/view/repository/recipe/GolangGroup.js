
/*global Ext, AS*/

/**
 * Repository "Settings" form for a Go Group repository.
 *
 * @since 3.17
 */
Ext.define('AS.coreui.view.repository.recipe.GolangGroup', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-go-group',
  requires: [
    'AS.coreui.view.repository.facet.StorageFacet',
    'AS.coreui.view.repository.facet.GroupFacet'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'as-coreui-repository-storage-facet'},
      {xtype: 'as-coreui-repository-group-facet', format: 'go'}
    ];

    me.callParent();
  }
});
