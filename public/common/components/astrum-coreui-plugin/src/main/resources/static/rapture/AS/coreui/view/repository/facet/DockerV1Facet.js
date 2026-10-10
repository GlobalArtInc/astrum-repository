
/*global Ext, AS*/

/**
 * Configuration for availability of Docker V1 api.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.repository.facet.DockerV1Facet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.as-coreui-repository-docker-v1-facet',
  requires: [
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {
        xtype: 'fieldset',
        itemId: 'dockerV1',
        cls: 'as-form-section',
        title: AS.I18n.get('Repository_Facet_DockerHostedFacet_V1_Title'),
        width: 600,
        items: [
          {
            xtype: 'checkbox',
            name: 'attributes.docker.v1Enabled',
            itemId: 'v1enabled',
            fieldLabel: AS.I18n.get('Repository_Facet_DockerHostedFacet_V1_Enabled'),
            helpText: AS.I18n.get('Repository_Facet_DockerHostedFacet_V1_Enabled_Help'),
            value: false
          }
        ]
      }
    ];

    me.callParent();
  }

});

