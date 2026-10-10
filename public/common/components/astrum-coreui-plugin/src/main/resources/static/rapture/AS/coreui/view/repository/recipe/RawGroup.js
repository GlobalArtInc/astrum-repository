
/*global Ext, AS*/

/**
 * Repository "Settings" form for a Raw Group repository.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.repository.recipe.RawGroup', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-raw-group',
  requires: [
    'AS.coreui.view.repository.facet.RawFacet',
    'AS.coreui.view.repository.facet.StorageFacet',
    'AS.coreui.view.repository.facet.GroupFacet'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'as-coreui-repository-raw-facet'},
      {xtype: 'as-coreui-repository-storage-facet'},
      {xtype: 'as-coreui-repository-group-facet', format: 'raw' }
    ];

    me.callParent();
  }
});
