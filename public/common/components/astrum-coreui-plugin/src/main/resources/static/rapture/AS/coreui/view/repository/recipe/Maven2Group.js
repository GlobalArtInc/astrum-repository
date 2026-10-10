
/*global Ext, AS*/

/**
 * Repository "Settings" form for a Maven Group repository.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.repository.recipe.Maven2Group', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-maven2-group',
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
      {xtype: 'as-coreui-repository-group-facet', format: 'maven2' }
    ];

    me.callParent();
  }
});
