
/*global Ext, NX*/

/**
 * Repository "Settings" form for a Maven Group repository.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.repository.recipe.Maven2Group', {
  extend: 'NX.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.nx-coreui-repository-maven2-group',
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
      {xtype: 'nx-coreui-repository-group-facet', format: 'maven2' }
    ];

    me.callParent();
  }
});
