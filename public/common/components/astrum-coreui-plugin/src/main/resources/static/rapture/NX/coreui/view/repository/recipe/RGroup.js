
/*global Ext, NX*/

/**
 * Repository "Settings" form for a R Group repository.
 *
 * @since 3.28
 */
Ext.define('NX.coreui.view.repository.recipe.RGroup', {
  extend: 'NX.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.nx-coreui-repository-r-group',
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
      {xtype: 'nx-coreui-repository-group-facet', format: 'r'}
    ];

    me.callParent();
  }
});
