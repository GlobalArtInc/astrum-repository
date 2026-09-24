
/*global Ext, NX*/

/**
 * @since 3.17
 */
Ext.define('NX.onboarding.app.PluginStrings', {
  '@aggregate_priority': 90,

  singleton: true,
  requires: [
    'NX.I18n'
  ],

  keys: {
    Onboarding_Text: 'Onboarding',
    Onboarding_Description: 'Configuration changes requiring attention',
    Onboarding_Authenticate: 'Your <b>admin</b> user password is located in <br><b>{0}</b> on the server.',
    Onboarding_LoadStepsError: 'Failed to retrieve setup steps from server'
  },

  bundles: {
    'NX.onboarding.view.OnboardingStartScreen': {
      Title: 'Let\'s Get You Set Up',
      Description: '<p>This wizard will guide you through a few quick steps to get your Astrum Repository ready for use.</p>',
    },
    'NX.onboarding.view.OnboardingCompleteScreen': {
      Title: 'Setup Complete',
      Description: '<p>Your instance is now ready to use. Explore GlobalArt Astrum Repository to unlock its full potential.</p>',
      Finish_Button: 'Finish'
    },
    'NX.onboarding.view.ChangeAdminPasswordScreen': {
      Title: 'Please choose a password for the admin user'
    },
    'NX.onboarding.view.ConfigureAnonymousAccessScreen': {
      Title: 'Configure Anonymous Access',
      Description: '<p><b>Enable anonymous access</b> means that by default, users can search, browse and download  ' +
      'components from repositories without credentials. Please <b>consider the security implications for your ' +
      ' organization.</b>' +
      '<br>' +
      '<p><b>Disable anonymous access</b> should be chosen with care, as it <b>will require credentials for all</b> ' +
      'users and/or build tools.'+
      '<br><br>' +
      '<a href="https://links.globalart.dev/products/astrum/anonymous-access/docs" target="_blank" rel="noopener">More information <span class="x-fa fa-external-link"></a></p>',
      Enable_Label: 'Enable anonymous access',
      Disable_Label: 'Disable anonymous access'
    },
    'NX.onboarding.view.ConfigureAnalyticsCollectionScreen': {
      Title: 'Help Us Improve Astrum Repository',
      Description: '<p>Please help us improve the Astrum Repository experience and shape future feature improvements by sharing ' +
          'anonymous statistical metrics and performance information with GlobalArt. The collected information will not contain ' +
          'identifying or proprietary information (e.g. the names of hosts, servers, repositories, or users).</p>',
      Enable_Label: 'Yes, I agree to share anonymous data.',
      Disable_Label: 'No, not interested.'
    },
    'NX.onboarding.view.CommunityEulaScreen': {
      Title: 'Agree End User License Agreement',
      Agree_Button: 'Agree'
    }
  }
}, function(obj) {
  NX.I18n.register(obj);
});
