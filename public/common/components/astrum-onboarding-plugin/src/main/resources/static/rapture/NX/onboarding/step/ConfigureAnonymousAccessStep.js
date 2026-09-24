
/*global Ext, NX*/

/**
 * Migration overview step.
 *
 * @since 3.0
 */
Ext.define('NX.onboarding.step.ConfigureAnonymousAccessStep', {
  extend: 'NX.wizard.Step',
  requires: [
    'NX.onboarding.view.ConfigureAnonymousAccessScreen',
    'NX.State'
  ],

  config: {
    screen: 'NX.onboarding.view.ConfigureAnonymousAccessScreen',
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

    NX.direct.coreui_AnonymousSettings.update({enabled: enabled, userId:'anonymous', realmName:'AstrumAuthorizingRealm'}, function(response) {
      if (response && response.success) {
        me.moveNext();
      }
    });
  }
});
