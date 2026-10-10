
/*global Ext, AS*/

/**
 * Configuration specific to HTTP connections for repositories that need both bearer token
 * support AND preemptive authentication support (e.g. Terraform proxy repositories).
 *
 * Combines the bearer token authentication type from BearerHttpClientFacet with the
 * preemptive authentication checkbox from HttpClientFacetWithPreemptiveAuth.
 *
 * @since 3.95
 */
Ext.define('AS.coreui.view.repository.facet.BearerHttpClientFacetWithPreemptiveAuth', {
  extend: 'AS.coreui.view.repository.facet.BearerHttpClientFacet',
  alias: 'widget.as-coreui-repository-httpclient-facet-with-bearer-token-and-preemptive-auth',
  requires: [
    'AS.I18n'
  ],

  authFields: function() {
    var me = this,
        parentAuthFields = me.callParent(arguments);

    parentAuthFields.push({
      xtype: 'checkbox',
      itemId: 'attributes_httpclient_authentication_preemptive',
      name: 'attributes.httpclient.authentication.preemptive',
      fieldLabel: AS.I18n.get('System_AuthenticationSettings_Preemptive_FieldLabel'),
      helpText: AS.I18n.get('System_AuthenticationSettings_Preemptive_HelpText'),
      value: false,
      disabled: true
    });

    return parentAuthFields;
  }
});
