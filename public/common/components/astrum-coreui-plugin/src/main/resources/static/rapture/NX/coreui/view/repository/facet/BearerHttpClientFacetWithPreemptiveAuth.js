
/*global Ext, NX*/

/**
 * Configuration specific to HTTP connections for repositories that need both bearer token
 * support AND preemptive authentication support (e.g. Terraform proxy repositories).
 *
 * Combines the bearer token authentication type from BearerHttpClientFacet with the
 * preemptive authentication checkbox from HttpClientFacetWithPreemptiveAuth.
 *
 * @since 3.95
 */
Ext.define('NX.coreui.view.repository.facet.BearerHttpClientFacetWithPreemptiveAuth', {
  extend: 'NX.coreui.view.repository.facet.BearerHttpClientFacet',
  alias: 'widget.nx-coreui-repository-httpclient-facet-with-bearer-token-and-preemptive-auth',
  requires: [
    'NX.I18n'
  ],

  authFields: function() {
    var me = this,
        parentAuthFields = me.callParent(arguments);

    parentAuthFields.push({
      xtype: 'checkbox',
      itemId: 'attributes_httpclient_authentication_preemptive',
      name: 'attributes.httpclient.authentication.preemptive',
      fieldLabel: NX.I18n.get('System_AuthenticationSettings_Preemptive_FieldLabel'),
      helpText: NX.I18n.get('System_AuthenticationSettings_Preemptive_HelpText'),
      value: false,
      disabled: true
    });

    return parentAuthFields;
  }
});
