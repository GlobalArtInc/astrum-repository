
/*global Ext, AS*/

/**
 * Repository "Settings" form for a Helm Group repository.
 */
Ext.define('AS.coreui.view.repository.recipe.HelmGroup', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-helm-group',
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
      {xtype: 'as-coreui-repository-group-facet', format: 'helm'}
    ];

    me.callParent();
  }
});
