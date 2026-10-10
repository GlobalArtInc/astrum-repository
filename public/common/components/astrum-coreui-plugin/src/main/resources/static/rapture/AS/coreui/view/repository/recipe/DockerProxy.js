
/*global Ext, AS*/

/**
 * Repository "Settings" form for a Docker Proxy repository.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.repository.recipe.DockerProxy', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-docker-proxy',
  requires: [
    'AS.coreui.view.repository.facet.ProxyFacet',
    'AS.coreui.view.repository.facet.StorageFacet',
    'AS.coreui.view.repository.facet.RoutingRuleFacet',
    'AS.coreui.view.repository.facet.HttpClientFacet',
    'AS.coreui.view.repository.facet.NegativeCacheFacet',
    'AS.coreui.view.repository.facet.DockerConnectorFacet',
    'AS.coreui.view.repository.facet.DockerProxyFacet',
    'AS.coreui.view.repository.facet.DockerV1Facet',
    'AS.coreui.view.repository.facet.CleanupPolicyFacet',
    'AS.coreui.view.repository.facet.FirewallFacet'
  ],

  // Matches an AWS ECR remote URL: <12-digit-accountId>.dkr.ecr.<region>.amazonaws.com
  // (including GovCloud / ISO / China partitions). Kept in sync with the backend EcrUrlParser
  // detection and the React ECR_URL_PATTERN gate.
  ecrUrlPattern: /^https?:\/\/\d{12}\.dkr\.ecr\.[a-z0-9-]+\.amazonaws\.com(\.cn)?\/?/i,

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'as-coreui-repository-docker-connector-facet'},
      {xtype: 'as-coreui-repository-docker-v1-facet'},
      {xtype: 'as-coreui-repository-firewall-facet'},
      {xtype: 'as-coreui-repository-proxy-facet'},
      {xtype: 'as-coreui-repository-storage-facet'},
      {xtype: 'as-coreui-repository-routing-rule-facet'},
      {xtype: 'as-coreui-repository-negativecache-facet'},
      {xtype: 'as-coreui-repository-cleanup-policy-facet'},
      {xtype: 'as-coreui-repository-httpclient-facet'}
    ];

    me.callParent();

    me.down('#remoteUrl').setHelpText(AS.I18n.get('Repository_Facet_ProxyFacet_Docker_Remote_HelpText'));
    me.down('#proxyFieldSet').add(1, {xtype: 'as-coreui-repository-docker-proxy-facet'});

    me.insertEcrSessionTokenField();
  },

  /**
   * Inserts the optional AWS ECR session token field into the shared HTTP authentication
   * fieldset, directly below the Password field, so all three AWS credential parts live
   * together: Access Key ID (username), Secret Access Key (password) and this session token.
   *
   * The field is only visible for AWS ECR remote URLs when the "username" auth type is
   * selected. Visibility tracks the remote URL and auth type changes.
   */
  insertEcrSessionTokenField: function() {
    var me = this,
        passwordField = me.down('#attributes_httpclient_authentication_password');

    if (!passwordField) {
      return;
    }

    var authFieldset = passwordField.up('as-optionalfieldset[checkboxName=authEnabled]');
    if (!authFieldset) {
      return;
    }

    var passwordIdx = authFieldset.items.indexOf(passwordField);
    authFieldset.insert(passwordIdx + 1, {
      xtype: 'as-password',
      itemId: 'ecrSessionToken',
      name: 'attributes.dockerProxy.ecrAuth.sessionToken',
      fieldLabel: AS.I18n.get('Repository_Facet_DockerProxyFacet_EcrSessionToken_FieldLabel'),
      helpText: AS.I18n.get('Repository_Facet_DockerProxyFacet_EcrSessionToken_HelpText'),
      allowBlank: true,
      hidden: true
    });

    var remoteUrlField = me.down('#remoteUrl'),
        authTypeCombo = me.down('[name=attributes.httpclient.authentication.type]');

    if (remoteUrlField) {
      remoteUrlField.on('change', me.updateEcrSessionTokenVisibility, me);
    }
    if (authTypeCombo) {
      authTypeCombo.on('change', me.updateEcrSessionTokenVisibility, me);
    }

    me.on('afterrender', me.updateEcrSessionTokenVisibility, me);
    me.on('boxready', me.updateEcrSessionTokenVisibility, me);
  },

  /**
   * Shows the ECR session token field only for ECR remote URLs with "username" auth type.
   */
  updateEcrSessionTokenVisibility: function() {
    var me = this,
        field = me.down('#ecrSessionToken');

    if (!field) {
      return;
    }

    var remoteUrlField = me.down('#remoteUrl'),
        authTypeCombo = me.down('[name=attributes.httpclient.authentication.type]'),
        remoteUrl = remoteUrlField ? remoteUrlField.getValue() : '',
        authType = authTypeCombo ? authTypeCombo.getValue() : 'username',
        isEcr = me.ecrUrlPattern.test(remoteUrl || '') && authType === 'username';

    field.setVisible(isEcr);
  },

  loadRecord: function(record) {
    var me = this,
        dockerProxyFacet = me.down('as-coreui-repository-docker-proxy-facet'),
        urls = record.get('attributes')['dockerProxy']['foreignLayerUrlWhitelist'];

    dockerProxyFacet.resetWhitelist();
    dockerProxyFacet.formLoad = true;
    me.callParent(arguments);

    if (urls) {
      urls.forEach(function(url) {
        dockerProxyFacet.addWhitelistRow(url);
      });
    }

    dockerProxyFacet.formLoad = false;
  }
});
