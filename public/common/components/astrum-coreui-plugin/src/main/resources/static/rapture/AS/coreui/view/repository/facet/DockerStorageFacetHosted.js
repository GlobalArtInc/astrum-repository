
/*global Ext, AS*/

/**
 * Storage configuration for docker hosted repositories.
 *
 * @since 3.21
 */
Ext.define('AS.coreui.view.repository.facet.DockerStorageFacetHosted', {
  extend: 'AS.coreui.view.repository.facet.StorageFacetHosted',
  alias: 'widget.as-coreui-repository-docker-storage-hosted-facet',
  requires: [
    'AS.I18n'
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
      fieldLabel: AS.I18n.get('Repository_Facet_StorageFacetHosted_Deployment_DisableLatestItem'),
      helpText: AS.I18n.get('Repository_Facet_StorageFacetHosted_Deployment_DisableLatestItemHelpText'),
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
