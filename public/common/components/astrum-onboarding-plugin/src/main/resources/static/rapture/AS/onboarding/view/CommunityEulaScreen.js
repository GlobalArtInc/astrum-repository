
/*global Ext, AS*/

/**
 * @since 3.77
 */
Ext.define('AS.onboarding.view.CommunityEulaScreen', {
  extend: 'AS.onboarding.view.OnboardingScreen',
  alias: 'widget.as-onboarding-community-eula-screen',
  requires: [
    'AS.Conditions',
    'AS.I18n',
    'AS.view.SecondaryContainer',
    'AS.util.Url'
  ],

  /**
   * @override
   */
  initComponent: function () {
    const me = this;

    Ext.apply(me, {
      title: AS.I18n.render(me, 'Title'),

      buttons: ['back', '->', {
        text: AS.I18n.render(me, 'Agree_Button'),
        action: 'next',
        enabled: true,
        cls: 'ce-eula-agree-button',
      }],

      fields: [
        {
          xtype: 'uxiframe',
          src: AS.util.Url.licenseUrl(),
          cls: 'as-onboarding-community-eula-license',
        },
        {
          xtype: 'as-secondary-container',
          reactView: window.ReactComponents.CommunityEulaOnboarding
        },
      ],
    });
    me.callParent();
  },
});
