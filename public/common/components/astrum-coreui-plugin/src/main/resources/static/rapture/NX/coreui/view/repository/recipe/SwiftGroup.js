
/*global Ext, NX*/

/**
 * Repository "Settings" form for a Swift Group repository.
 */
Ext.define('NX.coreui.view.repository.recipe.SwiftGroup', {
  extend: 'NX.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.nx-coreui-repository-swift-group',
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
      {xtype: 'nx-coreui-repository-group-facet', format: 'swift'}
    ];

    me.callParent();
  }
});