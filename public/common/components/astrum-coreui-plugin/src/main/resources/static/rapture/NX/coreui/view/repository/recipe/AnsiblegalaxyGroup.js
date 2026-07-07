
/*global Ext, NX*/

/**
 * Repository "Settings" form for an Ansible Galaxy Group repository
 */
Ext.define('NX.coreui.view.repository.recipe.AnsiblegalaxyGroup', {
  extend: 'NX.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.nx-coreui-repository-ansiblegalaxy-group',
  requires: [
    'NX.coreui.view.repository.facet.GroupFacet',
    'NX.coreui.view.repository.facet.StorageFacet'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'nx-coreui-repository-group-facet', format: 'ansiblegalaxy'},
      {xtype: 'nx-coreui-repository-storage-facet'}
    ];

    me.callParent();
  }
});
