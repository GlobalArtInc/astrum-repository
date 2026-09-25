
/*global Ext, NX*/

/**
 * @since 3.77
 */
Ext.define('NX.onboarding.view.CommunityEulaScreen', {
  extend: 'NX.onboarding.view.OnboardingScreen',
  alias: 'widget.nx-onboarding-community-eula-screen',
  requires: [
    'NX.Conditions',
    'NX.I18n',
    'NX.view.SecondaryContainer',
    'NX.util.Url'
  ],

  /**
   * @override
   */
  initComponent: function () {
    const me = this;

    Ext.apply(me, {
      title: NX.I18n.render(me, 'Title'),

      buttons: ['back', '->', {
        text: NX.I18n.render(me, 'Agree_Button'),
        action: 'next',
        enabled: true,
        cls: 'ce-eula-agree-button',
      }],

      fields: [
        {
          xtype: 'uxiframe',
          src: NX.util.Url.licenseUrl(),
          cls: 'nx-onboarding-community-eula-license',
        },
        {
          xtype: 'nx-secondary-container',
          reactView: window.ReactComponents.CommunityEulaOnboarding
        },
      ],
    });
    me.callParent();
  },
});
