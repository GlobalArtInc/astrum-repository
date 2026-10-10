
/*global Ext, AS*/

/**
 * Configuration specific to Http connections for npm repositories including bearer token support.
 *
 * @since 3.20
 */
Ext.define('AS.coreui.view.repository.facet.BearerHttpClientFacet', {
  extend: 'AS.coreui.view.repository.facet.HttpClientFacet',
  alias: 'widget.as-coreui-repository-httpclient-facet-with-bearer-token',
  requires: [
    'AS.I18n'
  ],

  authFields: function() {
    var me = this;
    var parentAuthFields = me.callParent(arguments);
    parentAuthFields.push(
        {
          xtype: 'textfield',
          inputType: 'password',
          itemId: 'attributes_httpclient_authentication_bearerToken',
          name: 'attributes.httpclient.authentication.bearerTokenId',
          fieldLabel: AS.I18n.get('System_AuthenticationSettings_Bearer_Token_FieldLabel'),
          helpText: AS.I18n.get('System_AuthenticationSettings_Bearer_Token_HelpText'),
          hidden: true,
          disabled: true,
          allowBlank: false
        });
    return parentAuthFields;
  },

  authTypeChanged: function(combo) {
    var form = this.up('form'),
        bearerTokenField = form.down('#attributes_httpclient_authentication_bearerToken'),
        usernameField = form.down('#attributes_httpclient_authentication_username'),
        passwordField = form.down('#attributes_httpclient_authentication_password');

    this.callParent(arguments);

    if (combo.getValue() === 'bearerToken') {
      usernameField.hide();
      usernameField.disable();
      usernameField.allowBlank = true;

      passwordField.hide();
      passwordField.disable();
      passwordField.allowBlank = true;

      bearerTokenField.show();
      bearerTokenField.enable();
      bearerTokenField.allowBlank = false;

      form.isValid();
    }
    else {
      bearerTokenField.hide();
      bearerTokenField.disable();
      bearerTokenField.allowBlank = true;

      usernameField.show();
      usernameField.enable();
      usernameField.allowBlank = false;

      passwordField.show();
      passwordField.enable();
      passwordField.allowBlank = false;

      form.isValid();
    }
  },

  getAuthTypeStore: function() {
    var me = this;
    var parentAuthTypes = me.callParent(arguments);
    parentAuthTypes.push([
        'bearerToken',
        AS.I18n.get('Repository_Facet_HttpClientFacet_AuthenticationType_Bearer_Token')
    ]);
    return parentAuthTypes;
  }

});
