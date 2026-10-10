
/*global Ext, AS*/

/**
 * @since 3.17
 */
Ext.define('AS.onboarding.controller.Onboarding', {
  extend: 'AS.wizard.Controller',
  requires: [
    'AS.Messages',
    'AS.I18n',
    'AS.onboarding.step.OnboardingStartStep',
    'AS.onboarding.step.CommunityEulaStep',
    'AS.onboarding.step.ChangeAdminPasswordStep',
    'AS.onboarding.step.ConfigureAnonymousAccessStep',
    'AS.onboarding.step.OnboardingCompleteStep',
    'AS.State'
  ],
  views: [
    'OnboardingWizard',
    'OnboardingStartScreen',
    'ChangeAdminPasswordScreen',
    'ConfigureAnonymousAccessScreen',
    'OnboardingCompleteScreen',
    'OnboardingModal',
    'CommunityEulaScreen'
  ],
  stores: [
    'Onboarding'
  ],

  /**
   * @override
   */
  init: function() {
    var me = this;

    me.callParent();

    me.listen({
      component: {
        'as-onboarding-wizard': {
          closed: me.reset
        }
      },
      controller: {
        '#State': {
          changed: me.stateChanged,
          userAuthenticated: me.stateChanged
        }
      },
      store: {
        '#Onboarding': {
          load: me.itemsLoaded
        }
      }
    });
  },

  stateChanged: function() {
    // When the React onboarding wizard is enabled, defer to it and skip the ExtJS wizard entirely.
    if (AS.State.getValue('astrum.react.onboarding.enabled')) {
      return;
    }

    var isOnboardingRequired = AS.State.getValue('onboarding.required'),
        user = AS.State.getUser();
    if (isOnboardingRequired && user && user['administrator']) {
      this.loadItems();
    }
  },

  /**
   * @override
   */
  finish: function() {
    AS.State.setValue('onboarding.required', false);
    var results = Ext.ComponentQuery.query('as-onboarding-modal');
    if (results && results.length) {
      results[0].close();
    }
  },

  loadItems: function() {
    var me = this,
    store = me.getStore('Onboarding');

    if (!store.isLoaded() && !store.isLoading()) {
      store.load();
    }
  },

  itemsLoaded: function (store, records, successful) {
    var me = this;
    me.registerStep('AS.onboarding.step.OnboardingStartStep');
    if (successful && Array.isArray(records)) {
      records.forEach(function(record) {
        me.registerStep('AS.onboarding.step.' + record.get('type') + 'Step');
      });
    }
    else {
      AS.Messages.error(AS.I18n.get('Onboarding_LoadStepsError'));
    }
    me.registerStep('AS.onboarding.step.OnboardingCompleteStep');

    Ext.widget('as-onboarding-modal');

    me.load();
  }
});
