
/*global Ext, AS*/

/**
 * Migration overview step.
 *
 * @since 3.0
 */
Ext.define('AS.onboarding.step.ConfigureAnonymousAccessStep', {
  extend: 'AS.wizard.Step',
  requires: [
    'AS.onboarding.view.ConfigureAnonymousAccessScreen',
    'AS.State'
  ],

  config: {
    screen: 'AS.onboarding.view.ConfigureAnonymousAccessScreen',
    enabled: true
  },

  /**
   * @override
   */
  init: function () {
    var me = this;

    me.control({
      'button[action=next]': {
        click: me.configureAnonymous
      },
      'button[action=back]': {
        click: me.moveBack
      }
    });
  },

  configureAnonymous: function(button) {
    var me = this,
        enabled = button.up('form').down('#anonymousEnabled').getValue();

    AS.direct.coreui_AnonymousSettings.update({enabled: enabled, userId:'anonymous', realmName:'AstrumAuthorizingRealm'}, function(response) {
      if (response && response.success) {
        me.moveNext();
      }
    });
  }
});
