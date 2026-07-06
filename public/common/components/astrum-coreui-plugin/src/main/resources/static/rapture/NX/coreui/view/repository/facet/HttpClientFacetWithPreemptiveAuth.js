
/*global Ext, NX*/

/**
 * Configuration specific to HTTP connections for Maven2 Proxy repositories including preemptive authentication support.
 *
 * @since 3.30
 */
Ext.define('NX.coreui.view.repository.facet.HttpClientFacetWithPreemptiveAuth', {
  extend: 'NX.coreui.view.repository.facet.HttpClientFacet',
  alias: 'widget.nx-coreui-repository-httpclient-facet-with-preemptive-auth',
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
