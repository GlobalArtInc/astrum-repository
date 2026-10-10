
/*global Ext, AS*/

/**
 * Repository "Settings" form for a Yum Group repository.
 *
 * @since 3.11
 */
Ext.define('AS.coreui.view.repository.recipe.YumGroup', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-yum-group',
  requires: [
    'AS.coreui.view.repository.facet.YumSigningFacet',
    'AS.coreui.view.repository.facet.StorageFacet',
    'AS.coreui.view.repository.facet.GroupFacet'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'as-coreui-repository-yum-signing-facet'},
      {xtype: 'as-coreui-repository-storage-facet'},
      {xtype: 'as-coreui-repository-group-facet', format: 'yum'}
    ];

    me.callParent();
  }
});
