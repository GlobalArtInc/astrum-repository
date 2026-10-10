
/*global Ext, AS*/

/**
 * Configuration specific to HTTP connections for Maven2 Proxy repositories including preemptive authentication support.
 *
 * @since 3.30
 */
Ext.define('AS.coreui.view.repository.facet.HttpClientFacetWithPreemptiveAuth', {
  extend: 'AS.coreui.view.repository.facet.HttpClientFacet',
  alias: 'widget.as-coreui-repository-httpclient-facet-with-preemptive-auth',
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
