
/*global Ext, AS*/

/**
 * Repository "Settings" form for an Ansible Galaxy Group repository
 */
Ext.define('AS.coreui.view.repository.recipe.AnsiblegalaxyGroup', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-ansiblegalaxy-group',
  requires: [
    'AS.coreui.view.repository.facet.GroupFacet',
    'AS.coreui.view.repository.facet.StorageFacet'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'as-coreui-repository-group-facet', format: 'ansiblegalaxy'},
      {xtype: 'as-coreui-repository-storage-facet'}
    ];

    me.callParent();
  }
});
