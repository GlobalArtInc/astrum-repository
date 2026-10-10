
/*global Ext, AS*/

/**
 * Repository "Settings" form for a R Group repository.
 *
 * @since 3.28
 */
Ext.define('AS.coreui.view.repository.recipe.RGroup', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-r-group',
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
      {xtype: 'as-coreui-repository-group-facet', format: 'r'}
    ];

    me.callParent();
  }
});
