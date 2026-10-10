
/**
 * Repository "Settings" form for an Alpine Group repository
 */
Ext.define('AS.coreui.view.repository.recipe.AlpineGroup', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-alpine-group',
  requires: [
    'AS.coreui.view.repository.facet.AlpineSigningFacet',
    'AS.coreui.view.repository.facet.StorageFacet',
    'AS.coreui.view.repository.facet.GroupFacet'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'as-coreui-repository-alpinesigning-facet'},
      {xtype: 'as-coreui-repository-storage-facet'},
      {xtype: 'as-coreui-repository-group-facet', format: 'alpine'}
    ];

    me.callParent();
  }
});
