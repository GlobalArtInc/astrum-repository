
/*global Ext, NX*/

/**
 * Storage configuration for docker hosted repositories.
 *
 * @since 3.21
 */
Ext.define('NX.coreui.view.repository.facet.DockerStorageFacetHosted', {
  extend: 'NX.coreui.view.repository.facet.StorageFacetHosted',
  alias: 'widget.nx-coreui-repository-docker-storage-hosted-facet',
  requires: [
    'NX.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this, writePolicyComponent, latestCheckbox, writePolicyFieldSet, proprietaryCheckbox;

    me.callParent();
    writePolicyFieldSet = me.down('#writePolicyFieldset');

    // Hide proprietary components checkbox for Docker (format does not support namespace confusion protection)
    proprietaryCheckbox = me.down('[name=attributes.component.proprietaryComponents]');
    if (proprietaryCheckbox) {
      proprietaryCheckbox.setVisible(false);
    }

    writePolicyFieldSet.add({
      xtype: 'checkboxfield',
      name: 'attributes.storage.latestPolicy',
      fieldLabel: NX.I18n.get('Repository_Facet_StorageFacetHosted_Deployment_DisableLatestItem'),
      helpText: NX.I18n.get('Repository_Facet_StorageFacetHosted_Deployment_DisableLatestItemHelpText'),
      checked: false
    });

    writePolicyComponent = me.down('[name=attributes.storage.writePolicy]');
    latestCheckbox = me.down('[name=attributes.storage.latestPolicy]');

    writePolicyComponent.on({
      select: function() {
        setLatestTagRedeployCheckboxVisibility(this.getValue(), latestCheckbox);
      },
      beforeRender: function() {
        setLatestTagRedeployCheckboxVisibility(this.getValue(), latestCheckbox);
      }
    });
  }

});

function setLatestTagRedeployCheckboxVisibility(writePolicyComponentValue, latestCheckbox) {
  latestCheckbox.setVisible(writePolicyComponentValue === 'ALLOW_ONCE');
}
