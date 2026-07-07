
/*global Ext, NX*/

/**
 * Repository "Settings" form for a Yum Group repository.
 *
 * @since 3.11
 */
Ext.define('NX.coreui.view.repository.recipe.YumGroup', {
  extend: 'NX.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.nx-coreui-repository-yum-group',
  requires: [
    'NX.coreui.view.repository.facet.YumSigningFacet',
    'NX.coreui.view.repository.facet.StorageFacet',
    'NX.coreui.view.repository.facet.GroupFacet'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'nx-coreui-repository-yum-signing-facet'},
      {xtype: 'nx-coreui-repository-storage-facet'},
      {xtype: 'nx-coreui-repository-group-facet', format: 'yum'}
    ];

    me.callParent();
  }
});
